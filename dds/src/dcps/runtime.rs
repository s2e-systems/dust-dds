use core::future::Future;

use super::infrastructure::{error::DdsResult, time::Time};

pub trait Clock {
    fn now(&self) -> Time;
}

pub trait Timer {
    fn delay(&mut self, duration: core::time::Duration) -> impl Future<Output = ()> + Send;
}

pub trait Spawner {
    fn spawn(&self, f: impl Future<Output = ()> + Send + 'static);
}

pub trait OneshotSend<T> {
    fn send(self, value: T) -> impl Future<Output = ()> + Send;
}

pub trait OneshotReceive<T> {
    fn receive(&mut self) -> impl Future<Output = DdsResult<T>> + Send;
}

pub trait ChannelSend<T> {
    fn send(&self, value: T) -> impl Future<Output = DdsResult<()>> + Send;
}

pub trait ChannelReceive<T> {
    fn receive(&mut self) -> impl Future<Output = Option<T>> + Send;
}

pub trait DdsRuntime: Send + 'static {
    type ClockHandle: Clock + Send + 'static;
    type TimerHandle: Timer + Clone + Send + 'static;
    type SpawnerHandle: Spawner + Clone + Send + Sync + 'static;
    type OneshotSender<T: Send>: OneshotSend<T> + Send;
    type OneshotReceiver<T: Send>: OneshotReceive<T> + Send;
    type ChannelSender<T: Send>: ChannelSend<T> + Clone + Send + Sync;
    type ChannelReceiver<T: Send + 'static>: ChannelReceive<T> + Send + 'static;

    fn timer(&self) -> Self::TimerHandle;
    fn clock(&self) -> Self::ClockHandle;
    fn spawner(&self) -> Self::SpawnerHandle;
    fn oneshot<T: Send>() -> (Self::OneshotSender<T>, Self::OneshotReceiver<T>);
    fn channel<T: Send>() -> (Self::ChannelSender<T>, Self::ChannelReceiver<T>);
    fn block_on<T>(f: impl Future<Output = T>) -> T;
}
