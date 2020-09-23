pub use rust_dds_interface::types::{InstanceHandle, Data, Duration, DURATION_INFINITE, DURATION_ZERO, Time, TIME_INVALID, LENGTH_UNLIMITED, ReturnCode, ReturnCodes};

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
pub type QosPolicyId = i32;

pub trait DDSType {
    fn instance_handle(&self) -> InstanceHandle;

    fn serialize(&self) -> Data;

    fn deserialize(data: Data) -> Self;
}

 
