use std::{
    sync::{
        mpsc::{channel, Sender},
        Mutex,
    },
    time::Duration,
};

use super::shared_object::{DdsRwLock, DdsShared};

pub trait Timer {
    fn new(duration: Duration) -> Self;
    fn reset(&mut self);
    fn on_deadline<F>(&mut self, f: F)
    where
        F: FnMut() + Send + Sync + 'static;
}

#[derive(PartialEq, Eq)]
enum TimerMessage {
    Reset,
    Stop,
}

type AsyncCallback = Box<dyn FnMut() + Send + Sync>;

pub struct ThreadTimer {
    on_deadline: DdsShared<DdsRwLock<Option<AsyncCallback>>>,
    sender: Mutex<Sender<TimerMessage>>,
}

impl Timer for ThreadTimer {
    fn new(duration: Duration) -> Self {
        let (sender, receiver) = channel();

        let timer = ThreadTimer {
            on_deadline: DdsShared::new(DdsRwLock::new(None)),
            sender: Mutex::new(sender),
        };

        let on_deadline = timer.on_deadline.clone();

        std::thread::spawn(move || 'timer: loop {
            match receiver.recv_timeout(duration) {
                Ok(TimerMessage::Reset) => {}
                Ok(TimerMessage::Stop) => break 'timer,
                Err(_) => {
                    if let Some(f) = &mut *on_deadline.write_lock() {
                        f();
                    }

                    let msg = receiver
                        .recv()
                        .expect("(;_;) Connection with sender closed unexpectedly");
                    match msg {
                        TimerMessage::Reset => (),
                        TimerMessage::Stop => break 'timer,
                    }
                }
            }
        });

        timer
    }

    fn reset(&mut self) {
        self.sender
            .lock()
            .unwrap()
            .send(TimerMessage::Reset)
            .unwrap();
        *self.on_deadline.write_lock() = None;
    }

    fn on_deadline<F>(&mut self, f: F)
    where
        F: FnMut() + Send + Sync + 'static,
    {
        self.sender
            .lock()
            .unwrap()
            .send(TimerMessage::Reset)
            .unwrap();
        *self.on_deadline.write_lock() = Some(Box::new(f));
    }
}

impl Drop for ThreadTimer {
    fn drop(&mut self) {
        self.sender
            .lock()
            .unwrap()
            .send(TimerMessage::Stop)
            .unwrap();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::implementation::utils::shared_object::{DdsRwLock, DdsShared};
    use std::time::Duration;

    #[test]
    fn timer_doesnt_execute_function_if_reset() {
        let counter = DdsShared::new(DdsRwLock::new(0));

        let mut timer = ThreadTimer::new(Duration::from_millis(10));

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

        let mut timer = ThreadTimer::new(Duration::from_millis(10));

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
    fn timer_thread_exits_quietly_when_dropped() {
        let timer = ThreadTimer::new(Duration::from_millis(10));
        drop(timer);
        std::thread::sleep(Duration::from_millis(50));
    }
}
