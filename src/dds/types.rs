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

pub struct Duration {
    sec: i32,
    nanosec: u32,
}