use std::{
    collections::BinaryHeap,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex, atomic::AtomicUsize},
    task::{Context, Poll, Waker},
    thread::Thread,
    time::{Duration, Instant},
};

use crate::runtime::Timer;

#[derive(Debug)]
struct TimerEntry {
    id: usize,
    deadline: Instant,
    waker: Waker,
}

impl PartialEq for TimerEntry {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.deadline == other.deadline
    }
}

impl Eq for TimerEntry {}

impl PartialOrd for TimerEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TimerEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse order compared to usual implementation
        // since the binary heap is a max tree and we want a min-heap
        other.deadline.cmp(&self.deadline)
    }
}

#[derive(Default, Debug)]
pub struct TimerState {
    next_id: AtomicUsize,
    entries: Mutex<BinaryHeap<TimerEntry>>,
    executor_thread: Mutex<Option<Thread>>,
}

impl TimerState {
    pub fn new() -> Self {
        Self {
            next_id: AtomicUsize::new(1),
            entries: Mutex::new(BinaryHeap::new()),
            executor_thread: Mutex::new(None),
        }
    }

    pub fn set_executor_thread(&self, thread: Thread) {
        *self.executor_thread.lock().unwrap() = Some(thread);
    }

    pub fn allocate_id(&self) -> usize {
        self.next_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    pub fn register(&self, id: usize, deadline: Instant, waker: Waker) {
        let mut entries = self.entries.lock().unwrap();
        let is_earliest = entries
            .peek()
            .is_none_or(|earliest| deadline < earliest.deadline);
        entries.push(TimerEntry {
            id,
            deadline,
            waker,
        });
        drop(entries);

        if is_earliest {
            if let Some(thread) = self.executor_thread.lock().unwrap().as_ref() {
                thread.unpark();
            }
        }
    }

    pub fn remove(&self, id: usize) {
        let mut entries = self.entries.lock().unwrap();
        entries.retain(|e| e.id != id);
    }

    pub fn duration_until_next_deadline(&self) -> Option<Duration> {
        let entries = self.entries.lock().unwrap();
        entries.peek().map(|earliest| {
            let now = Instant::now();
            if earliest.deadline > now {
                earliest.deadline.duration_since(now)
            } else {
                Duration::ZERO
            }
        })
    }

    pub fn wake_elapsed(&self) {
        let now = Instant::now();
        let mut to_wake = Vec::new();
        {
            let mut entries = self.entries.lock().unwrap();
            while let Some(earliest) = entries.peek() {
                if earliest.deadline <= now {
                    if let Some(entry) = entries.pop() {
                        to_wake.push(entry.waker);
                    }
                } else {
                    break;
                }
            }
        }
        for waker in to_wake {
            waker.wake();
        }
    }
}

#[derive(Debug)]
pub struct Sleep {
    id: usize,
    deadline: Option<Instant>,
    duration: Duration,
    timer_state: Arc<TimerState>,
}

impl Drop for Sleep {
    fn drop(&mut self) {
        self.timer_state.remove(self.id);
    }
}

impl Sleep {
    pub fn is_elapsed(&self) -> bool {
        if let Some(d) = self.deadline {
            Instant::now() >= d
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.deadline = Some(
            Instant::now()
                .checked_add(self.duration)
                .unwrap_or_else(|| {
                    // Fallback to a day sleep if duration is extremely large
                    Instant::now()
                        .checked_add(Duration::from_secs(24 * 60 * 60))
                        .unwrap_or_else(Instant::now)
                }),
        );
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
                this.reset();
                let deadline = this
                    .deadline
                    .expect("Must have deadline set after check above");
                this.timer_state
                    .register(this.id, deadline, cx.waker().clone());
            }
            Poll::Pending
        }
    }
}

#[derive(Clone, Debug)]
pub struct TimerHandle {
    state: Arc<TimerState>,
}

impl TimerHandle {
    pub fn sleep(&self, duration: Duration) -> Sleep {
        let id = self.state.allocate_id();
        Sleep {
            id,
            deadline: None,
            duration,
            timer_state: self.state.clone(),
        }
    }
}

impl Timer for TimerHandle {
    fn delay(&mut self, duration: core::time::Duration) -> impl Future<Output = ()> + Send {
        self.sleep(duration)
    }
}

pub struct TimerDriver {
    state: Arc<TimerState>,
}

impl Default for TimerDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl TimerDriver {
    pub fn new() -> Self {
        Self {
            state: Arc::new(TimerState::new()),
        }
    }

    pub fn state(&self) -> Arc<TimerState> {
        self.state.clone()
    }

    pub fn handle(&self) -> TimerHandle {
        TimerHandle {
            state: self.state.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reset_overflow() {
        let timer_state = Arc::new(TimerState::new());
        let mut sleep = Sleep {
            id: 1,
            deadline: None,
            duration: Duration::MAX,
            timer_state,
        };
        // This should not panic
        sleep.reset();
        assert!(sleep.deadline.is_some());
    }

    #[test]
    fn test_timer_delay() {
        let timer_driver = TimerDriver::new();
        let mut timer = timer_driver.handle();
        let start = Instant::now();
        let sleep_duration = Duration::from_millis(50);

        let fut = timer.delay(sleep_duration);
        let timer_state = timer_driver.state();
        let waker = std::task::Waker::noop();
        let mut cx = Context::from_waker(&waker);
        let mut pinned = std::pin::pin!(fut);

        assert!(matches!(pinned.as_mut().poll(&mut cx), Poll::Pending));
        assert!(timer_state.duration_until_next_deadline().is_some());

        std::thread::sleep(sleep_duration);
        timer_state.wake_elapsed();
        assert!(matches!(pinned.as_mut().poll(&mut cx), Poll::Ready(())));
        assert!(start.elapsed() >= sleep_duration);
    }
}
