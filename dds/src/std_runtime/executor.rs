use std::{
    cell::RefCell,
    collections::{BinaryHeap, HashMap},
    future::Future,
    pin::{Pin, pin},
    sync::{
        Arc, Mutex,
        atomic::{self, AtomicBool, AtomicU8, AtomicUsize},
        mpsc::{Sender, TryRecvError, channel},
    },
    task::{Context, Poll, Wake, Waker},
    thread::{self, JoinHandle, Thread},
    time::{Duration, Instant},
};

use embassy_sync::waitqueue::AtomicWaker;
use mio::{Events, Interest, Poll as MioPoll, Registry, Token, Waker as MioWaker};

use crate::{
    infrastructure::error::{DdsError, DdsResult},
    runtime::{Spawner, TaskHandle, Timer},
};

const WAKER_TOKEN: Token = Token(0);
const SOCKETS_PER_PARTICIPANT: usize = 8;

struct ParticipantIoEntry {
    ready_sockets: Arc<AtomicU8>,
    read_waker: Arc<AtomicWaker>,
}

#[derive(Clone)]
pub struct IoReactorHandle {
    registry: Arc<Registry>,
    entries: Arc<Mutex<HashMap<usize, ParticipantIoEntry>>>,
    next_id: Arc<AtomicUsize>,
    waker: Arc<MioWaker>,
}

thread_local! {
    static CURRENT_REACTOR: RefCell<Option<IoReactorHandle>> = const { RefCell::new(None) };
}

static DEFAULT_REACTOR: Mutex<Option<IoReactorHandle>> = Mutex::new(None);

impl IoReactorHandle {
    fn new(registry: Registry, waker: Arc<MioWaker>) -> Self {
        Self {
            registry: Arc::new(registry),
            entries: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(AtomicUsize::new(1)),
            waker,
        }
    }

    pub fn current() -> Option<Self> {
        CURRENT_REACTOR
            .with(|r| r.borrow().clone())
            .or_else(|| DEFAULT_REACTOR.lock().unwrap().clone())
    }

    pub fn allocate_participant_id(
        &self,
        ready_sockets: Arc<AtomicU8>,
        read_waker: Arc<AtomicWaker>,
    ) -> usize {
        let id = self.next_id.fetch_add(1, atomic::Ordering::Relaxed);
        self.entries.lock().unwrap().insert(
            id,
            ParticipantIoEntry {
                ready_sockets,
                read_waker,
            },
        );
        id
    }

    pub fn register_socket(
        &self,
        participant_id: usize,
        socket_idx: usize,
        socket: &mut mio::net::UdpSocket,
    ) {
        let token = Token(participant_id * SOCKETS_PER_PARTICIPANT + socket_idx);
        self.registry
            .register(socket, token, Interest::READABLE)
            .expect("Failed to register socket");
        self.waker.wake().ok();
    }

    pub fn deregister_participant(&self, participant_id: usize) {
        self.entries.lock().unwrap().remove(&participant_id);
    }

    fn process_events(&self, events: &Events) {
        let entries = self.entries.lock().unwrap();
        for event in events.iter() {
            let token = event.token().0;
            if token >= SOCKETS_PER_PARTICIPANT {
                let participant_id = token / SOCKETS_PER_PARTICIPANT;
                let socket_idx = token % SOCKETS_PER_PARTICIPANT;
                if let Some(entry) = entries.get(&participant_id) {
                    entry
                        .ready_sockets
                        .fetch_or(1 << socket_idx, atomic::Ordering::Release);
                    entry.read_waker.wake();
                }
            }
        }
    }
}

pub fn block_timeout<T>(
    duration: core::time::Duration,
    future: impl Future<Output = T>,
) -> DdsResult<T> {
    struct ChannelWake(std::sync::mpsc::SyncSender<()>);
    impl Wake for ChannelWake {
        fn wake(self: std::sync::Arc<Self>) {
            self.wake_by_ref();
        }

        fn wake_by_ref(self: &Arc<Self>) {
            self.0.send(()).ok();
        }
    }
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let waker = Waker::from(Arc::new(ChannelWake(sender)));
    let mut cx = Context::from_waker(&waker);
    let mut pinned_fut = pin!(future);
    let start_instant = std::time::Instant::now();
    loop {
        match pinned_fut.as_mut().poll(&mut cx) {
            Poll::Ready(t) => return Ok(t),
            Poll::Pending => {
                if let Some(timeout) =
                    duration.checked_sub(std::time::Instant::now().duration_since(start_instant))
                {
                    match receiver.recv_timeout(timeout) {
                        Ok(_) => (),
                        Err(_) => return Err(DdsError::Timeout),
                    }
                } else {
                    return Err(DdsError::Timeout);
                }
            }
        }
    }
}

pub fn block_on<T>(f: impl Future<Output = T>) -> T {
    struct ThreadWake(Thread);
    impl Wake for ThreadWake {
        fn wake(self: std::sync::Arc<Self>) {
            self.wake_by_ref()
        }

        fn wake_by_ref(self: &Arc<Self>) {
            self.0.unpark()
        }
    }
    let waker = Waker::from(Arc::new(ThreadWake(thread::current())));
    let mut cx = Context::from_waker(&waker);
    let mut pinned_fut = pin!(f);
    loop {
        match pinned_fut.as_mut().poll(&mut cx) {
            Poll::Ready(t) => return t,
            Poll::Pending => thread::park(),
        }
    }
}

pub struct Task {
    future: Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>,
    task_sender: Sender<Arc<Task>>,
    waker: Arc<MioWaker>,
    finished: AtomicBool,
    is_queued: AtomicBool,
    join_waker: Mutex<Option<Waker>>,
}

pub struct ExecutorTaskHandle {
    task: Arc<Task>,
}

impl TaskHandle for ExecutorTaskHandle {
    fn join(&self) {
        struct JoinFuture<'a> {
            task: &'a Arc<Task>,
        }

        impl<'a> Future for JoinFuture<'a> {
            type Output = ();

            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                if self.task.is_finished() {
                    Poll::Ready(())
                } else {
                    *self.task.join_waker.lock().unwrap() = Some(cx.waker().clone());
                    if self.task.is_finished() {
                        Poll::Ready(())
                    } else {
                        Poll::Pending
                    }
                }
            }
        }

        block_on(JoinFuture { task: &self.task });
    }
}

impl Task {
    fn is_finished(&self) -> bool {
        self.finished.load(atomic::Ordering::Acquire)
    }
}

impl Wake for Task {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref()
    }

    fn wake_by_ref(self: &Arc<Self>) {
        if !self.is_finished() && !self.is_queued.swap(true, atomic::Ordering::AcqRel) {
            self.task_sender.send(self.clone()).unwrap();
            self.waker.wake().ok();
        }
    }
}

#[derive(Clone)]
pub struct ExecutorHandle {
    task_sender: Sender<Arc<Task>>,
    waker: Arc<MioWaker>,
}

impl ExecutorHandle {
    pub fn spawn(&self, f: impl Future<Output = ()> + Send + 'static) -> ExecutorTaskHandle {
        let future = Box::pin(f);
        let task = Arc::new(Task {
            future: Mutex::new(future),
            task_sender: self.task_sender.clone(),
            waker: self.waker.clone(),
            finished: AtomicBool::new(false),
            is_queued: AtomicBool::new(true),
            join_waker: Mutex::new(None),
        });
        self.task_sender
            .send(task.clone())
            .expect("Should never fail to send");
        self.waker.wake().ok();
        ExecutorTaskHandle { task }
    }
}

impl Spawner for ExecutorHandle {
    type TaskHandle = ExecutorTaskHandle;

    fn spawn(&self, f: impl Future<Output = ()> + Send + 'static) -> Self::TaskHandle {
        self.spawn(f)
    }
}

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
        other.deadline.cmp(&self.deadline)
    }
}

#[derive(Debug)]
pub struct TimerState {
    next_id: AtomicUsize,
    entries: Mutex<BinaryHeap<TimerEntry>>,
    executor_waker: Arc<MioWaker>,
}

impl TimerState {
    pub fn new(executor_waker: Arc<MioWaker>) -> Self {
        Self {
            next_id: AtomicUsize::new(1),
            entries: Mutex::new(BinaryHeap::new()),
            executor_waker,
        }
    }

    pub fn allocate_id(&self) -> usize {
        self.next_id.fetch_add(1, atomic::Ordering::Relaxed)
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
            self.executor_waker.wake().ok();
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

    pub fn wake_elapsed(&self) -> bool {
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
        let woke_any = !to_wake.is_empty();
        for waker in to_wake {
            waker.wake();
        }
        woke_any
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

pub struct Executor {
    task_sender: Sender<Arc<Task>>,
    _executor_thread_handle: JoinHandle<()>,
    timer_state: Arc<TimerState>,
    waker: Arc<MioWaker>,
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

impl Executor {
    pub fn new() -> Self {
        let (task_sender, task_receiver) = channel::<Arc<Task>>();
        let mut poll = MioPoll::new().expect("Failed to create mio Poll");
        let waker = Arc::new(
            MioWaker::new(poll.registry(), WAKER_TOKEN).expect("Failed to create mio Waker"),
        );
        let registry = poll
            .registry()
            .try_clone()
            .expect("Failed to clone mio Registry");
        let reactor_handle = IoReactorHandle::new(registry, waker.clone());
        *DEFAULT_REACTOR.lock().unwrap() = Some(reactor_handle.clone());

        let timer_state = Arc::new(TimerState::new(waker.clone()));
        let timer_state_clone = timer_state.clone();
        let reactor_handle_clone = reactor_handle.clone();

        let executor_thread_handle = std::thread::Builder::new()
            .name("Dust DDS Executor".to_string())
            .spawn(move || {
                CURRENT_REACTOR.with(|r| {
                    *r.borrow_mut() = Some(reactor_handle_clone.clone());
                });
                let mut events = Events::with_capacity(128);

                loop {
                    match task_receiver.try_recv() {
                        Ok(task) => {
                            task.is_queued.store(false, atomic::Ordering::Release);
                            if !task.is_finished() {
                                let waker = Waker::from(task.clone());
                                let mut cx = Context::from_waker(&waker);
                                let poll_result = task
                                    .future
                                    .try_lock()
                                    .expect("Only ever locked here")
                                    .as_mut()
                                    .poll(&mut cx);
                                if matches!(poll_result, Poll::Ready(_)) {
                                    task.finished.store(true, atomic::Ordering::Release);
                                    if let Some(waker) = task.join_waker.lock().unwrap().take() {
                                        waker.wake();
                                    }
                                }
                            }
                        }
                        Err(TryRecvError::Empty) => {
                            let woke_timer = timer_state_clone.wake_elapsed();
                            let timeout = if woke_timer {
                                Some(Duration::ZERO)
                            } else {
                                timer_state_clone.duration_until_next_deadline()
                            };

                            match poll.poll(&mut events, timeout) {
                                Ok(_) => {}
                                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                                Err(_) => break,
                            }

                            reactor_handle_clone.process_events(&events);
                            timer_state_clone.wake_elapsed();
                        }
                        Err(TryRecvError::Disconnected) => break,
                    }
                }
            })
            .expect("failed to spawn thread");

        Self {
            task_sender,
            _executor_thread_handle: executor_thread_handle,
            timer_state,
            waker,
        }
    }

    pub fn handle(&self) -> ExecutorHandle {
        ExecutorHandle {
            task_sender: self.task_sender.clone(),
            waker: self.waker.clone(),
        }
    }

    pub fn timer_handle(&self) -> TimerHandle {
        TimerHandle {
            state: self.timer_state.clone(),
        }
    }
}
