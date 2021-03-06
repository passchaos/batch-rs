//! A serialized `Task` annotated with metadata.

use std::fmt;
use std::result::Result as StdResult;
use std::time::Duration;

use futures::Future;
use uuid::Uuid;

use client::Client;
use error::{self, Error, Result};
use rabbitmq::Exchange;
use task::Task;
use ser;

/// A `Query` is responsible for publishing jobs to `RabbitMQ`.
pub struct Query<T>
where
    T: Task,
{
    task: T,
    exchange: String,
    routing_key: String,
    timeout: Option<Duration>,
    retries: u32,
}

impl<T> fmt::Debug for Query<T>
where
    T: Task,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> StdResult<(), fmt::Error> {
        write!(
            f,
            "Query {{ exchange: {:?} routing_key: {:?} timeout: {:?} retries: {:?} }}",
            self.exchange, self.routing_key, self.timeout, self.retries
        )
    }
}

impl<T> Query<T>
where
    T: Task,
{
    /// Create a new `Query` from a `Task` instance.
    pub fn new(task: T) -> Self {
        Query {
            task,
            exchange: T::exchange().into(),
            routing_key: T::routing_key().into(),
            timeout: T::timeout(),
            retries: T::retries(),
        }
    }

    /// Set the exchange this task will be published to.
    pub fn exchange(mut self, exchange: &str) -> Self {
        self.exchange = exchange.into();
        self
    }

    /// Set the routing key associated with this task.
    pub fn routing_key(mut self, routing_key: &str) -> Self {
        self.routing_key = routing_key.into();
        self
    }

    /// Set the timeout associated to this task's execution.
    pub fn timeout(mut self, timeout: Option<Duration>) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the number of allowed retries for this task.
    pub fn retries(mut self, retries: u32) -> Self {
        self.retries = retries;
        self
    }

    /// Send the job using the given client.
    pub fn send(self, client: &Client) -> Box<Future<Item = (), Error = Error>> {
        let serialized = ser::to_vec(&self.task)
            .map_err(error::ErrorKind::Serialization)
            .unwrap();
        let job = Job {
            uuid: Uuid::new_v4(),
            name: String::from(T::name()),
            queue: self.routing_key,
            task: serialized,
            timeout: self.timeout,
            retries: self.retries,
        };
        client.send(&job)
    }
}

/// Shorthand to create a new `Query` instance from a `Task`.
pub fn job<T>(task: T) -> Query<T>
where
    T: Task,
{
    Query::new(task)
}

/// A `Job` is a serialized `Task` with metadata about its status & how it should be executed.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Job {
    uuid: Uuid,
    name: String,
    queue: String,
    task: Vec<u8>,
    timeout: Option<Duration>,
    retries: u32,
}

impl Job {
    /// Returns the UUIDv4 of this job.
    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }

    /// Returns the name of the task associated to this job.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the queue this job should be pushed to.
    pub fn queue(&self) -> &str {
        &self.queue
    }

    /// Returns the raw serialized task this job is associated to.
    pub fn task(&self) -> &[u8] {
        &self.task
    }

    /// Returns the timeout associated to this job.
    pub fn timeout(&self) -> Option<Duration> {
        self.timeout
    }

    /// Returns the number of retries this job is allowed.
    pub fn retries(&self) -> u32 {
        self.retries
    }

    /// Returns the `Job` that should be sent if the execution failed.
    ///
    /// If `retries` was 0, the function returns `None` as nothing should be sent to
    /// the broker.
    pub fn failed(self) -> Option<Job> {
        if self.retries() == 0 {
            None
        } else {
            Some(Job {
                retries: self.retries() - 1,
                ..self
            })
        }
    }
}

/// The different states a `Job` can be in.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Status {
    /// The job was created but it wasn't sent/received yet.
    Pending,
    /// The job was received by a worker that started executing it.
    Started,
    /// The job completed successfully.
    Success,
    /// The job didn't complete successfully, see attached `Failure` cause.
    Failed(Failure),
}

/// Stores the reason for a job failure.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Failure {
    /// The task handler returned an error.
    Error,
    /// The task didn't complete in time.
    Timeout,
    /// The task crashed (panic, segfault, etc.) while executing.
    Crash,
}
