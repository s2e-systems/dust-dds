use std::{
    sync::{mpsc::{channel, Sender}, Mutex},
    time::Duration,
};

use super::shared_object::{DdsRwLock, DdsShared};

#[derive(PartialEq, Eq)]
enum TimerMessage {
    Reset,

    #[cfg(test)]
    ProvokeTimeout,
}

pub struct Timer {
    on_deadline: DdsShared<DdsRwLock<Option<Box<dyn FnMut() + Send + Sync>>>>,
    sender: Mutex<Sender<TimerMessage>>,
}

impl Timer {
    pub fn new(duration: Duration) -> Self {
        let (sender, receiver) = channel();

        let timer = Timer {
            on_deadline: DdsShared::new(DdsRwLock::new(None)),
            sender: Mutex::new(sender),
        };

        let on_deadline = timer.on_deadline.clone();

        std::thread::spawn(move || loop {
            match receiver.recv_timeout(duration) {
                Ok(TimerMessage::Reset) => {}
                _ => {
                    if let Some(f) = &mut *on_deadline.write_lock() {
                        f();
                    }

                    while receiver.recv() != Ok(TimerMessage::Reset) {}
                }
            }
        });

        timer
    }

    pub fn reset(&mut self)
    {
        self.sender.lock().unwrap().send(TimerMessage::Reset).unwrap();
        *self.on_deadline.write_lock() = None;
    }

    pub fn on_deadline<F>(&mut self, f: F)
    where
        F: FnMut() + Send + Sync + 'static,
    {
        self.sender.lock().unwrap().send(TimerMessage::Reset).unwrap();
        *self.on_deadline.write_lock() = Some(Box::new(f));
    }

    #[cfg(test)]
    pub fn provoke_timeout(&mut self) {
        self.sender.lock().unwrap().send(TimerMessage::ProvokeTimeout).unwrap();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::shared_object::{DdsRwLock, DdsShared};
    use std::time::{Duration, Instant};

    #[test]
    fn timer_executes_function_in_time() {
        let counter = DdsShared::new(DdsRwLock::new(0));

        let mut timer = Timer::new(Duration::from_millis(20));

        let counter_ref = counter.clone();
        timer.on_deadline(move || {
            *counter_ref.write_lock() += 1;
        });

        let start = Instant::now();
        while *counter.read_lock() == 0 {
            if Instant::now() - start >= Duration::from_millis(50) {
                panic!("(;_;) Too late");
            }
        }

        if Instant::now() - start <= Duration::from_millis(20) {
            panic!("(;_;) Too early!");
        }
    }

    #[test]
    fn timer_doesnt_execute_function_if_reset() {
        let counter = DdsShared::new(DdsRwLock::new(0));

        let mut timer = Timer::new(Duration::from_millis(10));

        let counter_ref = counter.clone();
        timer.on_deadline(move || {
            *counter_ref.write_lock() += 1;
        });

        timer.reset();

        std::thread::sleep(Duration::from_millis(30));
        assert_eq!(0, *counter.read_lock());
    }

    #[test]
    fn timer_executes_new_function_if_reset() {
        let counter = DdsShared::new(DdsRwLock::new(0));

        let mut timer = Timer::new(Duration::from_millis(10));

        let counter_ref = counter.clone();
        timer.on_deadline(move || {
            *counter_ref.write_lock() += 1;
        });

        let counter_ref = counter.clone();
        timer.on_deadline(move || {
            *counter_ref.write_lock() += 2;
        });

        std::thread::sleep(Duration::from_millis(30));
        assert_eq!(2, *counter.read_lock());
    }

    #[test]
    fn provoke_timeout_manually() {
        let counter = DdsShared::new(DdsRwLock::new(0));

        let mut timer = Timer::new(Duration::from_secs(60));

        let counter_ref = counter.clone();
        timer.on_deadline(move || {
            *counter_ref.write_lock() += 1;
        });
        timer.provoke_timeout();

        let t0 = Instant::now();
        while *counter.read_lock() == 0 && (Instant::now() - t0) < Duration::from_millis(50) {}

        assert_eq!(1, *counter.read_lock());
    }
}
