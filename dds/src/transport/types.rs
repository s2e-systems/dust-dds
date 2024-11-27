/// TopicKind_t
/// Enumeration used to distinguish whether a Topic has defined some fields within to be used as the 'key' that identifies data-instances within the Topic. See the DDS specification for more details on keys.
/// The following values are reserved by the protocol: NO_KEY, WITH_KEY
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TopicKind {
    NoKey,
    WithKey,
}

/// ChangeKind_t
/// Enumeration used to distinguish the kind of change that was made to a data-object. Includes changes to the data or the instance state of the data-object.
/// It can take the values:
/// ALIVE, ALIVE_FILTERED, NOT_ALIVE_DISPOSED, NOT_ALIVE_UNREGISTERED
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(dead_code)]
pub enum ChangeKind {
    Alive,
    AliveFiltered,
    NotAliveDisposed,
    NotAliveUnregistered,
    NotAliveDisposedUnregistered,
}

/// ReliabilityKind_t
/// Enumeration used to indicate the level of the reliability used for communications.
/// It can take the values: BEST_EFFORT, RELIABLE.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReliabilityKind {
    BestEffort,
    Reliable,
}

#[derive(Clone, PartialEq, Debug, Copy, PartialOrd, Eq, Ord)]
pub struct Time {
    sec: i32,
    nanosec: u32,
}

impl Time {
    /// Create a new [`Time`] with a number of seconds and nanoseconds
    pub const fn new(sec: i32, nanosec: u32) -> Self {
        let sec = sec + (nanosec / 1_000_000_000) as i32;
        let nanosec = nanosec % 1_000_000_000;
        Self { sec, nanosec }
    }

    /// Get the number of seconds contained by this time
    pub const fn sec(&self) -> i32 {
        self.sec
    }

    /// Get the number of nanoseconds contained by this time
    pub const fn nanosec(&self) -> u32 {
        self.nanosec
    }
}
