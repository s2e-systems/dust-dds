use super::infrastructure::time::Time;
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

    /// Returns a timer handle for this runtime
    fn timer(&self) -> Self::TimerHandle;
    /// Returns a clock handle for this runtime
    fn clock(&self) -> Self::ClockHandle;
    /// Returns a spawner handle for this runtime
    fn spawner(&self) -> Self::SpawnerHandle;

    /// Runs a future to completion on the current thread
    ///
    /// # Arguments
    /// * `f` - Future to be executed
    fn block_on<T>(f: impl Future<Output = T>) -> T;
}
