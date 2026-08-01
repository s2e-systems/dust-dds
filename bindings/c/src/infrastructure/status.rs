#![allow(non_camel_case_types)]

use crate::infrastructure::qos_policy::QosPolicyId_t;

pub type StatusKind = u32;
pub type StatusMask = u32;
pub type InstanceHandle_t = [u8; 16];

pub const INCONSISTENT_TOPIC_STATUS: StatusKind = 0x0001 << 0;
pub const OFFERED_DEADLINE_MISSED_STATUS: StatusKind = 0x0001 << 1;
pub const REQUESTED_DEADLINE_MISSED_STATUS: StatusKind = 0x0001 << 2;
pub const OFFERED_INCOMPATIBLE_QOS_STATUS: StatusKind = 0x0001 << 5;
pub const REQUESTED_INCOMPATIBLE_QOS_STATUS: StatusKind = 0x0001 << 6;
pub const SAMPLE_LOST_STATUS: StatusKind = 0x0001 << 7;
pub const SAMPLE_REJECTED_STATUS: StatusKind = 0x0001 << 8;
pub const DATA_ON_READERS_STATUS: StatusKind = 0x0001 << 9;
pub const DATA_AVAILABLE_STATUS: StatusKind = 0x0001 << 10;
pub const LIVELINESS_LOST_STATUS: StatusKind = 0x0001 << 11;
pub const LIVELINESS_CHANGED_STATUS: StatusKind = 0x0001 << 12;
pub const PUBLICATION_MATCHED_STATUS: StatusKind = 0x0001 << 13;
pub const SUBSCRIPTION_MATCHED_STATUS: StatusKind = 0x0001 << 14;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InconsistentTopicStatus {
    pub total_count: i32,
    pub total_count_change: i32,
}

impl From<dust_dds::infrastructure::status::InconsistentTopicStatus> for InconsistentTopicStatus {
    fn from(status: dust_dds::infrastructure::status::InconsistentTopicStatus) -> Self {
        Self {
            total_count: status.total_count,
            total_count_change: status.total_count_change,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SampleLostStatus {
    pub total_count: i32,
    pub total_count_change: i32,
}

impl From<dust_dds::infrastructure::status::SampleLostStatus> for SampleLostStatus {
    fn from(status: dust_dds::infrastructure::status::SampleLostStatus) -> Self {
        Self {
            total_count: status.total_count,
            total_count_change: status.total_count_change,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SampleRejectedStatusKind {
    NOT_REJECTED,
    REJECTED_BY_INSTANCES_LIMIT,
    REJECTED_BY_SAMPLES_LIMIT,
    REJECTED_BY_SAMPLES_PER_INSTANCE_LIMIT,
}

impl From<dust_dds::infrastructure::status::SampleRejectedStatusKind> for SampleRejectedStatusKind {
    fn from(kind: dust_dds::infrastructure::status::SampleRejectedStatusKind) -> Self {
        match kind {
            dust_dds::infrastructure::status::SampleRejectedStatusKind::NotRejected => Self::NOT_REJECTED,
            dust_dds::infrastructure::status::SampleRejectedStatusKind::RejectedByInstancesLimit => Self::REJECTED_BY_INSTANCES_LIMIT,
            dust_dds::infrastructure::status::SampleRejectedStatusKind::RejectedBySamplesLimit => Self::REJECTED_BY_SAMPLES_LIMIT,
            dust_dds::infrastructure::status::SampleRejectedStatusKind::RejectedBySamplesPerInstanceLimit => Self::REJECTED_BY_SAMPLES_PER_INSTANCE_LIMIT,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SampleRejectedStatus {
    pub total_count: i32,
    pub total_count_change: i32,
    pub last_reason: SampleRejectedStatusKind,
    pub last_instance_handle: InstanceHandle_t,
}

impl From<dust_dds::infrastructure::status::SampleRejectedStatus> for SampleRejectedStatus {
    fn from(status: dust_dds::infrastructure::status::SampleRejectedStatus) -> Self {
        Self {
            total_count: status.total_count,
            total_count_change: status.total_count_change,
            last_reason: status.last_reason.into(),
            last_instance_handle: status.last_instance_handle.into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LivelinessLostStatus {
    pub total_count: i32,
    pub total_count_change: i32,
}

impl From<dust_dds::infrastructure::status::LivelinessLostStatus> for LivelinessLostStatus {
    fn from(status: dust_dds::infrastructure::status::LivelinessLostStatus) -> Self {
        Self {
            total_count: status.total_count,
            total_count_change: status.total_count_change,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LivelinessChangedStatus {
    pub alive_count: i32,
    pub not_alive_count: i32,
    pub alive_count_change: i32,
    pub not_alive_count_change: i32,
    pub last_publication_handle: InstanceHandle_t,
}

impl From<dust_dds::infrastructure::status::LivelinessChangedStatus> for LivelinessChangedStatus {
    fn from(status: dust_dds::infrastructure::status::LivelinessChangedStatus) -> Self {
        Self {
            alive_count: status.alive_count,
            not_alive_count: status.not_alive_count,
            alive_count_change: status.alive_count_change,
            not_alive_count_change: status.not_alive_count_change,
            last_publication_handle: status.last_publication_handle.into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OfferedDeadlineMissedStatus {
    pub total_count: i32,
    pub total_count_change: i32,
    pub last_instance_handle: InstanceHandle_t,
}

impl From<dust_dds::infrastructure::status::OfferedDeadlineMissedStatus>
    for OfferedDeadlineMissedStatus
{
    fn from(status: dust_dds::infrastructure::status::OfferedDeadlineMissedStatus) -> Self {
        Self {
            total_count: status.total_count,
            total_count_change: status.total_count_change,
            last_instance_handle: status.last_instance_handle.into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequestedDeadlineMissedStatus {
    pub total_count: i32,
    pub total_count_change: i32,
    pub last_instance_handle: InstanceHandle_t,
}

impl From<dust_dds::infrastructure::status::RequestedDeadlineMissedStatus>
    for RequestedDeadlineMissedStatus
{
    fn from(status: dust_dds::infrastructure::status::RequestedDeadlineMissedStatus) -> Self {
        Self {
            total_count: status.total_count,
            total_count_change: status.total_count_change,
            last_instance_handle: status.last_instance_handle.into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QosPolicyCount {
    pub policy_id: QosPolicyId_t,
    pub count: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QosPolicyCountSeq {
    pub length: i32,
    pub buffer: *mut QosPolicyCount,
}

impl Default for QosPolicyCountSeq {
    fn default() -> Self {
        Self {
            length: 0,
            buffer: std::ptr::null_mut(),
        }
    }
}

impl From<Vec<dust_dds::infrastructure::status::QosPolicyCount>> for QosPolicyCountSeq {
    fn from(v: Vec<dust_dds::infrastructure::status::QosPolicyCount>) -> Self {
        if v.is_empty() {
            Self::default()
        } else {
            let mut counts: Vec<QosPolicyCount> = v
                .into_iter()
                .map(|q| QosPolicyCount {
                    policy_id: q.policy_id,
                    count: q.count,
                })
                .collect();
            counts.shrink_to_fit();
            let length = counts.len() as i32;
            let buffer = counts.as_mut_ptr();
            std::mem::forget(counts);
            Self { length, buffer }
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_qos_policy_count_seq_free(seq: QosPolicyCountSeq) {
    if !seq.buffer.is_null() && seq.length > 0 {
        unsafe {
            let _ = Box::from_raw(std::slice::from_raw_parts_mut(
                seq.buffer,
                seq.length as usize,
            ));
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OfferedIncompatibleQosStatus {
    pub total_count: i32,
    pub total_count_change: i32,
    pub last_policy_id: QosPolicyId_t,
    pub policies: QosPolicyCountSeq,
}

impl From<dust_dds::infrastructure::status::OfferedIncompatibleQosStatus>
    for OfferedIncompatibleQosStatus
{
    fn from(status: dust_dds::infrastructure::status::OfferedIncompatibleQosStatus) -> Self {
        Self {
            total_count: status.total_count,
            total_count_change: status.total_count_change,
            last_policy_id: status.last_policy_id,
            policies: status.policies.into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequestedIncompatibleQosStatus {
    pub total_count: i32,
    pub total_count_change: i32,
    pub last_policy_id: QosPolicyId_t,
    pub policies: QosPolicyCountSeq,
}

impl From<dust_dds::infrastructure::status::RequestedIncompatibleQosStatus>
    for RequestedIncompatibleQosStatus
{
    fn from(status: dust_dds::infrastructure::status::RequestedIncompatibleQosStatus) -> Self {
        Self {
            total_count: status.total_count,
            total_count_change: status.total_count_change,
            last_policy_id: status.last_policy_id,
            policies: status.policies.into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PublicationMatchedStatus {
    pub total_count: i32,
    pub total_count_change: i32,
    pub current_count: i32,
    pub current_count_change: i32,
    pub last_subscription_handle: InstanceHandle_t,
}

impl From<dust_dds::infrastructure::status::PublicationMatchedStatus> for PublicationMatchedStatus {
    fn from(status: dust_dds::infrastructure::status::PublicationMatchedStatus) -> Self {
        Self {
            total_count: status.total_count,
            total_count_change: status.total_count_change,
            current_count: status.current_count,
            current_count_change: status.current_count_change,
            last_subscription_handle: status.last_subscription_handle.into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SubscriptionMatchedStatus {
    pub total_count: i32,
    pub total_count_change: i32,
    pub current_count: i32,
    pub current_count_change: i32,
    pub last_publication_handle: InstanceHandle_t,
}

impl From<dust_dds::infrastructure::status::SubscriptionMatchedStatus>
    for SubscriptionMatchedStatus
{
    fn from(status: dust_dds::infrastructure::status::SubscriptionMatchedStatus) -> Self {
        Self {
            total_count: status.total_count,
            total_count_change: status.total_count_change,
            current_count: status.current_count,
            current_count_change: status.current_count_change,
            last_publication_handle: status.last_publication_handle.into(),
        }
    }
}
