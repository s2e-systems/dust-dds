use core::future::Future;

use super::infrastructure::{error::DdsResult, time::Time};

pub trait Clock {
    fn now(&self) -> Time;
}

pub trait Timer {
    fn delay(&mut self, duration: core::time::Duration) -> impl Future<Output = ()> + Send;
}

pub trait OneshotSend<T> {
    fn send(self, value: T) -> impl Future<Output = ()> + Send;
}

pub trait OneshotReceive<T> {
    fn receive(&mut self) -> impl Future<Output = DdsResult<T>> + Send;
}

pub trait DdsRuntime: Send + 'static {
    type ClockHandle: Clock;
    type TimerHandle: Timer + Send + 'static;
    type OneshotSender<T>: OneshotSend<T> + Send
    where
        T: Send;
    type OneshotReceiver<T>: OneshotReceive<T> + Send
    where
        T: Send;

    fn timer(&mut self) -> Self::TimerHandle;
    fn clock(&mut self) -> Self::ClockHandle;
    fn oneshot<T>() -> (Self::OneshotSender<T>, Self::OneshotReceiver<T>)
    where
        T: Send;
}
