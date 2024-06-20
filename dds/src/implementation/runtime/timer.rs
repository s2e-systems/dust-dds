use std::{
    collections::BinaryHeap,
    future::{poll_fn, Future},
    pin::{pin, Pin},
    sync::{mpsc::RecvTimeoutError, Arc, Mutex},
    task::{Context, Poll, Waker},
    thread::JoinHandle,
    time::{Duration, Instant},
};

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

pub struct Sleep {
    id: usize,
    deadline: Option<Instant>,
    duration: Duration,
    periodic_task_sender: std::sync::mpsc::Sender<TimerWake>,
}

impl Sleep {
    pub fn is_elapsed(&self) -> bool {
        if let Some(d) = self.deadline {
            Instant::now() > d
        } else {
            false
        }
    }

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

#[derive(Debug)]
pub enum TimeoutError {
    Timeout,
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
            t.waker.wake();
        }
    }
}

struct HandleInner {
    sleep_task_id: usize,
    periodic_task_sender: std::sync::mpsc::Sender<TimerWake>,
}

#[derive(Clone)]
pub struct TimerHandle {
    inner: Arc<Mutex<HandleInner>>,
}

impl TimerHandle {
    pub fn sleep(&self, duration: Duration) -> Sleep {
        let mut inner_lock = self.inner.lock().expect("Mutex should not be poisoned");
        let id = inner_lock.sleep_task_id;
        inner_lock.sleep_task_id += 1;
        Sleep {
            id,
            deadline: None,
            duration,
            periodic_task_sender: inner_lock.periodic_task_sender.clone(),
        }
    }

    pub fn timeout<T>(
        &self,
        duration: Duration,
        mut future: Pin<Box<dyn Future<Output = T> + Send>>,
    ) -> impl Future<Output = Result<T, TimeoutError>> {
        let mut timeout = self.sleep(duration);

        poll_fn(move |cx| {
            if let Poll::Ready(t) = pin!(&mut future).poll(cx) {
                return Poll::Ready(Ok(t));
            }
            if pin!(&mut timeout).poll(cx).is_ready() {
                return Poll::Ready(Err(TimeoutError::Timeout));
            }

            Poll::Pending
        })
    }
}

pub struct TimerDriver {
    inner: Arc<Mutex<HandleInner>>,
    _timer_thread_join_handle: JoinHandle<()>,
}

impl TimerDriver {
    pub fn new() -> Self {
        let (periodic_task_sender, periodic_task_receiver) =
            std::sync::mpsc::channel::<TimerWake>();
        let timer_thread_join_handle = std::thread::spawn(move || {
            let mut timer_heap = TimerHeap::new();
            loop {
                // Check if there are any elapsed tasks and wake them
                while timer_heap.is_next_timer_elapsed() {
                    timer_heap.notify_next_timer();
                }

                // Wait for a new timer wake to come. Sleep forever
                // if there are no timer tasks on the queue otherwise
                // sleep until the next deadline so that the tasks can be
                // notified at the correct time
                let new_timer = match timer_heap.duration_until_next_timer() {
                    Some(d) => periodic_task_receiver.recv_timeout(d),
                    None => periodic_task_receiver
                        .recv()
                        .map_err(|_| RecvTimeoutError::Disconnected),
                };

                match new_timer {
                    Ok(t) => timer_heap.push(t),
                    Err(RecvTimeoutError::Timeout) => (),
                    Err(RecvTimeoutError::Disconnected) => break,
                }
            }
        });
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
