pub type InstanceHandle = [u8; 16];
pub type Data = Vec<u8>;


//// From DDS
#[derive(PartialOrd, PartialEq, Debug, Clone)]
pub struct Duration {
    pub sec: i32,
    pub nanosec: u32,
}

pub const DURATION_INFINITE: Duration = Duration{sec: 0x7fffffff, nanosec:0x7fffffff};
pub const DURATION_ZERO: Duration = Duration{sec: 0, nanosec:0};


pub struct Time {
    pub sec: i32,
    pub nanosec: u32,
}

pub const TIME_INVALID: Time = Time{sec: -1, nanosec:0xffffffff};

pub type Length = i32;
pub const LENGTH_UNLIMITED: i32 = -1;  

pub struct ResourceLimits {
    pub max_samples: Length,
    pub max_instances: Length,
    pub max_samples_per_instance: Length,
}

pub enum HistoryKind {
    KeepAll,
    KeepLast(i32),
}

//// From RTPS
#[derive(PartialEq)]
pub enum ReliabilityKind {
    BestEffort,
    Reliable,
}

pub enum TopicKind {
    NoKey,
    WithKey,
}
