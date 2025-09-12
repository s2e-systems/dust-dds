use super::infrastructure::{error::DdsResult, time::Time};
use core::future::Future;

/// Represents a clock that provides the current time.
pub trait Clock {
    /// Returns the current time.
    fn now(&self) -> Time;
}

/// Provides delay/sleep functionality in an asynchronous context.
pub trait Timer {
    /// Creates a future that completes after the specified duration has elapsed.
    ///
    /// # Arguments
    /// * `duration` - The amount of time to delay for
    fn delay(&mut self, duration: core::time::Duration) -> impl Future<Output = ()> + Send;
}

/// Provides task spawning capabilities for asynchronous execution.
pub trait Spawner {
    /// Spawns a new asynchronous task.
    ///
    /// # Arguments
    /// * `f` - Future to be spawned as a new task
    fn spawn(&self, f: impl Future<Output = ()> + Send + 'static);
}

/// Represents the sending half of a one-shot channel.
pub trait OneshotSend<T> {
    /// Sends a value through the one-shot channel.
    ///
    /// # Arguments
    /// * `value` - Value to be sent
    fn send(self, value: T);
}

/// Represents the receiving half of a one-shot channel.
pub trait OneshotReceive<T> {
    /// Receives a value from the one-shot channel.
    ///
    /// Returns a future that resolves to the received value or an error.
    fn receive(self) -> impl Future<Output = DdsResult<T>> + Send;
}

/// Represents the sending half of a multiple-producer channel.
pub trait ChannelSend<T> {
    /// Sends a value through the channel.
    ///
    /// # Arguments
    /// * `value` - Value to be sent
    ///
    /// Returns a future that resolves when the send operation completes.
    fn send(&self, value: T) -> impl Future<Output = DdsResult<()>> + Send;
}

/// Represents the receiving half of a multiple-producer channel.
pub trait ChannelReceive<T> {
    /// Receives a value from the channel.
    ///
    /// Returns a future that resolves to Some(value) if a value was received,
    /// or None if the channel is closed.
    fn receive(&mut self) -> impl Future<Output = Option<T>> + Send;
}

/// Main runtime trait for the DDS system, providing access to various async primitives
/// and runtime capabilities.
///
/// This trait combines clock, timer, spawner, and channel functionality into a cohesive runtime.
pub trait DdsRuntime: Send + 'static {
    /// Type representing a clock implementation for this runtime
    type ClockHandle: Clock + Clone + Send + Sync + 'static;
    /// Type representing a timer implementation for this runtime
    type TimerHandle: Timer + Clone + Send + Sync + 'static;
    /// Type representing a spawner implementation for this runtime
    type SpawnerHandle: Spawner + Clone + Send + Sync + 'static;
    /// Type representing a one-shot sender for this runtime
    type OneshotSender<T: Send>: OneshotSend<T> + Send;
    /// Type representing a one-shot receiver for this runtime
    type OneshotReceiver<T: Send>: OneshotReceive<T> + Send;
    /// Type representing a channel sender for this runtime
    type ChannelSender<T: Send>: ChannelSend<T> + Clone + Send + Sync;
    /// Type representing a channel receiver for this runtime
    type ChannelReceiver<T: Send + 'static>: ChannelReceive<T> + Send + 'static;

    /// Returns a timer handle for this runtime
    fn timer(&self) -> Self::TimerHandle;
    /// Returns a clock handle for this runtime
    fn clock(&self) -> Self::ClockHandle;
    /// Returns a spawner handle for this runtime
    fn spawner(&self) -> Self::SpawnerHandle;
    /// Creates a new one-shot channel, returning the sender and receiver handles
    fn oneshot<T: Send>() -> (Self::OneshotSender<T>, Self::OneshotReceiver<T>);
    /// Creates a new multiple-producer channel, returning the sender and receiver handles
    fn channel<T: Send>() -> (Self::ChannelSender<T>, Self::ChannelReceiver<T>);
    /// Runs a future to completion on the current thread
    ///
    /// # Arguments
    /// * `f` - Future to be executed
    fn block_on<T>(f: impl Future<Output = T>) -> T;
}
