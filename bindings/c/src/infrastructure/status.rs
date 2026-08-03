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
            let _ = Box::from_raw(std::ptr::slice_from_raw_parts_mut(
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

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstanceHandleSeq {
    pub length: i32,
    pub buffer: *mut InstanceHandle_t,
}

impl Default for InstanceHandleSeq {
    fn default() -> Self {
        Self {
            length: 0,
            buffer: std::ptr::null_mut(),
        }
    }
}

impl InstanceHandleSeq {
    pub unsafe fn to_vec(&self) -> Vec<dust_dds::infrastructure::instance::InstanceHandle> {
        if self.buffer.is_null() || self.length <= 0 {
            Vec::new()
        } else {
            let slice = unsafe { std::slice::from_raw_parts(self.buffer, self.length as usize) };
            slice
                .iter()
                .map(|&bytes| dust_dds::infrastructure::instance::InstanceHandle::new(bytes))
                .collect()
        }
    }

    pub fn from_vec(v: &[dust_dds::infrastructure::instance::InstanceHandle]) -> Self {
        if v.is_empty() {
            Self::default()
        } else {
            let mut ptrs: Vec<InstanceHandle_t> = v.iter().map(|&h| <[u8; 16]>::from(h)).collect();
            ptrs.shrink_to_fit();
            let length = ptrs.len() as i32;
            let buffer = ptrs.as_mut_ptr();
            std::mem::forget(ptrs);
            Self { length, buffer }
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_instance_handle_seq_free(seq: InstanceHandleSeq) {
    if !seq.buffer.is_null() && seq.length > 0 {
        unsafe {
            let _ = Vec::from_raw_parts(seq.buffer, seq.length as usize, seq.length as usize);
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuiltInTopicKey {
    pub value: [u8; 16],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParticipantBuiltinTopicData {
    pub key: BuiltInTopicKey,
    pub user_data: crate::infrastructure::qos_policy::UserDataQosPolicy,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_participant_builtin_topic_data_free(
    data: ParticipantBuiltinTopicData,
) {
    unsafe {
        crate::infrastructure::qos_policy::dds_octet_seq_free(data.user_data.value);
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TopicBuiltinTopicData {
    pub key: BuiltInTopicKey,
    pub name: *mut std::os::raw::c_char,
    pub type_name: *mut std::os::raw::c_char,
    pub durability: crate::infrastructure::qos_policy::DurabilityQosPolicy,
    pub deadline: crate::infrastructure::qos_policy::DeadlineQosPolicy,
    pub latency_budget: crate::infrastructure::qos_policy::LatencyBudgetQosPolicy,
    pub liveliness: crate::infrastructure::qos_policy::LivelinessQosPolicy,
    pub reliability: crate::infrastructure::qos_policy::ReliabilityQosPolicy,
    pub transport_priority: crate::infrastructure::qos_policy::TransportPriorityQosPolicy,
    pub lifespan: crate::infrastructure::qos_policy::LifespanQosPolicy,
    pub destination_order: crate::infrastructure::qos_policy::DestinationOrderQosPolicy,
    pub history: crate::infrastructure::qos_policy::HistoryQosPolicy,
    pub resource_limits: crate::infrastructure::qos_policy::ResourceLimitsQosPolicy,
    pub ownership: crate::infrastructure::qos_policy::OwnershipQosPolicy,
    pub topic_data: crate::infrastructure::qos_policy::TopicDataQosPolicy,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_topic_builtin_topic_data_free(data: TopicBuiltinTopicData) {
    if !data.name.is_null() {
        unsafe {
            let _ = std::ffi::CString::from_raw(data.name);
        }
    }
    if !data.type_name.is_null() {
        unsafe {
            let _ = std::ffi::CString::from_raw(data.type_name);
        }
    }
}
