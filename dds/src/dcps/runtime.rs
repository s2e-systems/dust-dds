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
    fn send(&mut self, value: T) -> impl Future<Output = DdsResult<()>> + Send;
}

pub trait ChannelReceive<T> {
    fn receive(&mut self) -> impl Future<Output = Option<T>> + Send;
}

pub trait DdsRuntime: Send + 'static {
    type ClockHandle: Clock + Send + 'static;
    type TimerHandle: Timer + Clone + Send + 'static;
    type SpawnerHandle: Spawner + Clone + Send + Sync + 'static;
    type OneshotSender<T>: OneshotSend<T> + Send
    where
        T: Send;
    type OneshotReceiver<T>: OneshotReceive<T> + Send
    where
        T: Send;
    type ChannelSender<T>: ChannelSend<T> + Send
    where
        T: Send;
    type ChannelReceiver<T>: ChannelReceive<T> + Send
    where
        T: Send;

    fn timer(&self) -> Self::TimerHandle;
    fn clock(&self) -> Self::ClockHandle;
    fn spawner(&self) -> Self::SpawnerHandle;
    fn oneshot<T>() -> (Self::OneshotSender<T>, Self::OneshotReceiver<T>)
    where
        T: Send;
    fn channel<T>() -> (Self::ChannelSender<T>, Self::ChannelReceiver<T>)
    where
        T: Send;
}
