pub trait ThreadManager {
    fn spawn<F>(&mut self, f: F) -> std::io::Result<()>
    where
        F: FnOnce(),
        F: Send + 'static,
        Self: Sized;
}

pub struct StdThread;

impl ThreadManager for StdThread {
    fn spawn<F>(&mut self, f: F) -> std::io::Result<()>
    where
        F: FnOnce(),
        F: Send + 'static,
    {
        std::thread::spawn(f);
        Ok(())
    }
}
