use super::infrastructure::time::Time;
use core::{future::Future, pin::Pin, task::Poll};

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
}

/// Generic either enumeration type
pub(crate) enum Either<A, B> {
    /// Variant A of either
    A(A),
    /// Variant B of either
    B(B),
}
struct Select<A, B> {
    a: A,
    b: B,
}

/// Run two futures in parallel and return the first one to conclude. In case both futures are ready at the same time
/// priority is given to the future A.
pub(crate) async fn select_future<A: Future, B: Future>(
    a: A,
    b: B,
) -> Either<A::Output, B::Output> {
    Select {
        a: core::pin::pin!(a),
        b: core::pin::pin!(b),
    }
    .await
}

impl<A, B> Future for Select<A, B>
where
    A: Future + Unpin,
    B: Future + Unpin,
{
    type Output = Either<A::Output, B::Output>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        if let Poll::Ready(a) = Pin::new(&mut self.a).poll(cx) {
            return Poll::Ready(Either::A(a));
        }
        if let Poll::Ready(b) = Pin::new(&mut self.b).poll(cx) {
            return Poll::Ready(Either::B(b));
        }
        Poll::Pending
    }
}
