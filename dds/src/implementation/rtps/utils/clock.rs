pub trait TimerConstructor {
    fn new() -> Self;
}

pub trait Timer {
    fn reset(&mut self);
    fn elapsed(&self) -> std::time::Duration;
}

#[derive(Debug, PartialEq, Eq)]
pub struct StdTimer(std::time::Instant);

impl TimerConstructor for StdTimer {
    fn new() -> Self {
        Self(std::time::Instant::now())
    }
}

impl Timer for StdTimer {
    fn reset(&mut self) {
        self.0 = std::time::Instant::now();
    }

    fn elapsed(&self) -> std::time::Duration {
        self.0.elapsed()
    }
}
