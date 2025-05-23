use std::{
    collections::BinaryHeap,
    future::Future,
    pin::Pin,
    sync::{mpsc::RecvTimeoutError, Arc, Mutex},
    task::{Context, Poll, Waker},
    thread::JoinHandle,
    time::{Duration, Instant},
};

use tracing::trace;

use crate::runtime::Timer;

pub struct TimerWake {
    id: usize,
    deadline: Instant,
    waker: Waker,
}

impl PartialEq for TimerWake {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.deadline == other.deadline
    }
}

impl Eq for TimerWake {}

impl PartialOrd for TimerWake {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TimerWake {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse order compared to usual implementation
        // since the binary heap is a max tree
        other.deadline.cmp(&self.deadline)
    }
}

#[derive(Debug)]
pub struct Sleep {
    id: usize,
    deadline: Option<Instant>,
    duration: Duration,
    periodic_task_sender: std::sync::mpsc::Sender<TimerWake>,
}

impl Sleep {
    #[tracing::instrument]
    pub fn is_elapsed(&self) -> bool {
        if let Some(d) = self.deadline {
            Instant::now() > d
        } else {
            false
        }
    }

    #[tracing::instrument]
    pub fn reset(&mut self) {
        self.deadline = Some(Instant::now() + self.duration);
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if this.is_elapsed() {
            Poll::Ready(())
        } else {
            if this.deadline.is_none() {
                // First time being polled starts the sleep count
                this.reset();
            }
            let deadline = this
                .deadline
                .expect("Must have deadline set after check above");
            let timer_wake = TimerWake {
                id: this.id,
                deadline,
                waker: cx.waker().clone(),
            };
            this.periodic_task_sender
                .send(timer_wake)
                .expect("Shouldn't fail to send");
            Poll::Pending
        }
    }
}

struct TimerHeap {
    heap: BinaryHeap<TimerWake>,
}

impl TimerHeap {
    fn new() -> Self {
        Self {
            heap: BinaryHeap::with_capacity(10),
        }
    }

    #[inline(always)]
    fn push(&mut self, item: TimerWake) {
        self.heap.push(item)
    }

    #[inline(always)]
    fn duration_until_next_timer(&self) -> Option<Duration> {
        if let Some(t) = self.heap.peek() {
            let d = t.deadline.duration_since(Instant::now());
            Some(d)
        } else {
            None
        }
    }

    #[inline(always)]
    fn is_next_timer_elapsed(&self) -> bool {
        matches!(self.heap.peek(), Some(t) if t.deadline < Instant::now())
    }

    #[inline(always)]
    fn notify_next_timer(&mut self) {
        if let Some(t) = self.heap.pop() {
            debug_assert!(t.deadline < Instant::now());
            trace!("Notify timer with id {}", t.id);
            t.waker.wake();
        }
    }
}

#[derive(Debug)]
struct HandleInner {
    sleep_task_id: usize,
    periodic_task_sender: std::sync::mpsc::Sender<TimerWake>,
}

#[derive(Clone, Debug)]
pub struct TimerHandle {
    inner: Arc<Mutex<HandleInner>>,
}

impl TimerHandle {
    #[tracing::instrument]
    pub fn sleep(&self, duration: Duration) -> Sleep {
        let mut inner_lock = self.inner.lock().expect("Mutex should not be poisoned");
        let id = inner_lock.sleep_task_id;
        inner_lock.sleep_task_id += 1;
        trace!("Create Sleep with id {}", inner_lock.sleep_task_id);
        Sleep {
            id,
            deadline: None,
            duration,
            periodic_task_sender: inner_lock.periodic_task_sender.clone(),
        }
    }
}

impl Timer for TimerHandle {
    #[tracing::instrument]
    fn delay(&mut self, duration: core::time::Duration) -> impl Future<Output = ()> + Send {
        self.sleep(duration)
    }
}

pub struct TimerDriver {
    inner: Arc<Mutex<HandleInner>>,
    _timer_thread_join_handle: JoinHandle<()>,
}

impl Default for TimerDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl TimerDriver {
    pub fn new() -> Self {
        let (periodic_task_sender, periodic_task_receiver) =
            std::sync::mpsc::channel::<TimerWake>();
        let timer_thread_join_handle = std::thread::Builder::new()
            .name("Dust DDS Timer".to_string())
            .spawn(move || {
                let mut timer_heap = TimerHeap::new();
                loop {
                    // Check if there are any elapsed tasks and wake them
                    while timer_heap.is_next_timer_elapsed() {
                        trace!("Notifying next timer");
                        timer_heap.notify_next_timer();
                    }

                    // Wait for a new timer wake to come. Sleep forever
                    // if there are no timer tasks on the queue otherwise
                    // sleep until the next deadline so that the tasks can be
                    // notified at the correct time
                    let new_timer = match timer_heap.duration_until_next_timer() {
                        Some(d) => {
                            trace!("Waiting for new waker value for fixed duration {:?}", d);
                            periodic_task_receiver.recv_timeout(d)
                        }
                        None => {
                            trace!("Waiting for new waker value indefinitely");
                            periodic_task_receiver
                                .recv()
                                .map_err(|_| RecvTimeoutError::Disconnected)
                        }
                    };

                    match new_timer {
                        Ok(t) => timer_heap.push(t),
                        Err(RecvTimeoutError::Timeout) => (),
                        Err(RecvTimeoutError::Disconnected) => break,
                    }
                }
            })
            .expect("failed to spawn thread");
        let inner = Arc::new(Mutex::new(HandleInner {
            sleep_task_id: 0,
            periodic_task_sender,
        }));
        Self {
            inner,
            _timer_thread_join_handle: timer_thread_join_handle,
        }
    }

    pub fn handle(&self) -> TimerHandle {
        TimerHandle {
            inner: self.inner.clone(),
        }
    }
}
