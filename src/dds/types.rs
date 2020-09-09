pub enum ReturnCode {
    Ok,
    Error,
    Unsupported,
    BadParameter,
    PreconditionNotMet,
    OutOfResources,
    NotEnabled,
    ImmutablePolicy,
    InconsistentPolicy,
    AlreadyDeleted,
    Timeout,
    NoData,
    IllegalOperation,
}

pub type StatusKind = u32;
pub type SampleStateKind = u32;
pub type ViewStateKind = u32;
pub type InstanceStateKind = u32;

pub type DomainId = i32;
pub type InstanceHandle = [u8; 16];
pub type QosPolicyId = i32;

pub struct Time {
    sec: i32,
    nanosec: u32,
}

#[derive(PartialOrd, PartialEq)]
pub struct Duration {
    pub sec: i32,
    pub nanosec: u32,
}
pub const LENGTH_UNLIMITED: i32 = -1;   
pub const DURATION_INFINITE: Duration = Duration{sec: 0x7fffffff, nanosec:0x7fffffff};
pub const DURATION_ZERO: Duration = Duration{sec: 0, nanosec:0};
pub const TIME_INVALID: Time = Time{sec: -1, nanosec:0xffffffff};
