pub trait Timer {
    fn start() -> Self;
    fn elapsed(&self) -> std::time::Duration;
}

pub struct StdTimer(std::time::Instant);

impl Timer for StdTimer {
    fn start() -> Self {
        Self(std::time::Instant::now())
    }

    fn elapsed(&self) -> std::time::Duration {
        self.0.elapsed()
    }
}