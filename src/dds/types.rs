pub type ReturnCode<T> = Result<T, ReturnCodes>;

#[derive(Debug)]
pub enum ReturnCodes {
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
pub type StatusMask = u32;

pub const INCONSISTENT_TOPIC_STATUS           : StatusKind = 0x0001 << 0;
pub const OFFERED_DEADLINE_MISSED_STATUS      : StatusKind = 0x0001 << 1;
pub const REQUESTED_DEADLINE_MISSED_STATUS    : StatusKind = 0x0001 << 2;
pub const OFFERED_INCOMPATIBLE_QOS_STATUS     : StatusKind = 0x0001 << 5;
pub const REQUESTED_INCOMPATIBLE_QOS_STATUS   : StatusKind = 0x0001 << 6;
pub const SAMPLE_LOST_STATUS                  : StatusKind = 0x0001 << 7;
pub const SAMPLE_REJECTED_STATUS              : StatusKind = 0x0001 << 8;
pub const DATA_ON_READERS_STATUS              : StatusKind = 0x0001 << 9;
pub const DATA_AVAILABLE_STATUS               : StatusKind = 0x0001 << 10;
pub const LIVELINESS_LOST_STATUS              : StatusKind = 0x0001 << 11;
pub const LIVELINESS_CHANGED_STATUS           : StatusKind = 0x0001 << 12;
pub const PUBLICATION_MATCHED_STATUS          : StatusKind = 0x0001 << 13;
pub const SUBSCRIPTION_MATCHED_STATUS         : StatusKind = 0x0001 << 14;

pub type SampleStateKind = u32;
pub type ViewStateKind = u32;
pub type InstanceStateKind = u32;

pub type DomainId = i32;
pub type InstanceHandle = [u8; 16];
pub type QosPolicyId = i32;

pub trait DDSType {
    fn key(&self) -> InstanceHandle;

    fn data(&self) -> Vec<u8>;
}

pub struct Time {
    pub sec: i32,
    pub nanosec: u32,
}

#[derive(PartialOrd, PartialEq, Debug, Clone)]
pub struct Duration {
    pub sec: i32,
    pub nanosec: u32,
}
pub const LENGTH_UNLIMITED: i32 = -1;   
pub const DURATION_INFINITE: Duration = Duration{sec: 0x7fffffff, nanosec:0x7fffffff};
pub const DURATION_ZERO: Duration = Duration{sec: 0, nanosec:0};
pub const TIME_INVALID: Time = Time{sec: -1, nanosec:0xffffffff};
