use core::time::Duration;

pub trait Clock {
    fn now(&self) -> Duration;
}
