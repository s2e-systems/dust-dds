use super::infrastructure::time::Time;

pub trait Clock {
    fn now(&self) -> Time;
}
