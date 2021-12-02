pub trait Timer {
    fn reset(&mut self);

    fn elapsed(&self) -> std::time::Duration;
}

pub struct StdTimer(std::time::Instant);

impl StdTimer {
    pub fn new() -> Self {
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
