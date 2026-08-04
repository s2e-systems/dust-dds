pub type StatusMask = u32;
use crate::infrastructure::error::{RETCODE_BAD_PARAMETER, RETCODE_OK, ReturnCode};
use std::ptr::NonNull;

/// cbindgen:opaque
pub struct StatusCondition(pub(crate) dust_dds::condition::StatusCondition);

impl StatusCondition {
    pub fn new(condition: dust_dds::condition::StatusCondition) -> Self {
        Self(condition)
    }

    pub fn inner(&self) -> &dust_dds::condition::StatusCondition {
        &self.0
    }
}

pub const DDS_INCONSISTENT_TOPIC_STATUS: StatusMask = 0x00000001;
pub const DDS_OFFERED_DEADLINE_MISSED_STATUS: StatusMask = 0x00000002;
pub const DDS_REQUESTED_DEADLINE_MISSED_STATUS: StatusMask = 0x00000004;
pub const DDS_OFFERED_INCOMPATIBLE_QOS_STATUS: StatusMask = 0x00000008;
pub const DDS_REQUESTED_INCOMPATIBLE_QOS_STATUS: StatusMask = 0x00000010;
pub const DDS_SAMPLE_LOST_STATUS: StatusMask = 0x00000020;
pub const DDS_SAMPLE_REJECTED_STATUS: StatusMask = 0x00000040;
pub const DDS_DATA_ON_READERS_STATUS: StatusMask = 0x00000080;
pub const DDS_DATA_AVAILABLE_STATUS: StatusMask = 0x00000100;
pub const DDS_LIVELINESS_LOST_STATUS: StatusMask = 0x00000200;
pub const DDS_LIVELINESS_CHANGED_STATUS: StatusMask = 0x00000400;
pub const DDS_PUBLICATION_MATCHED_STATUS: StatusMask = 0x00000800;
pub const DDS_SUBSCRIPTION_MATCHED_STATUS: StatusMask = 0x00001000;

pub(crate) fn mask_to_status_kinds(
    mask: StatusMask,
) -> Vec<dust_dds::infrastructure::status::StatusKind> {
    let mut kinds = Vec::new();
    if mask & DDS_INCONSISTENT_TOPIC_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::InconsistentTopic);
    }
    if mask & DDS_OFFERED_DEADLINE_MISSED_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::OfferedDeadlineMissed);
    }
    if mask & DDS_REQUESTED_DEADLINE_MISSED_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::RequestedDeadlineMissed);
    }
    if mask & DDS_OFFERED_INCOMPATIBLE_QOS_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::OfferedIncompatibleQos);
    }
    if mask & DDS_REQUESTED_INCOMPATIBLE_QOS_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::RequestedIncompatibleQos);
    }
    if mask & DDS_SAMPLE_LOST_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::SampleLost);
    }
    if mask & DDS_SAMPLE_REJECTED_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::SampleRejected);
    }
    if mask & DDS_DATA_ON_READERS_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::DataOnReaders);
    }
    if mask & DDS_DATA_AVAILABLE_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::DataAvailable);
    }
    if mask & DDS_LIVELINESS_LOST_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::LivelinessLost);
    }
    if mask & DDS_LIVELINESS_CHANGED_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::LivelinessChanged);
    }
    if mask & DDS_PUBLICATION_MATCHED_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::PublicationMatched);
    }
    if mask & DDS_SUBSCRIPTION_MATCHED_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::SubscriptionMatched);
    }
    kinds
}

/// Defines the list of communication statuses that are taken into account to determine the trigger_value of the StatusCondition.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `condition` must point to a valid, initialized `StatusCondition` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_StatusCondition_set_enabled_statuses(
    condition: Option<NonNull<StatusCondition>>,
    mask: StatusMask,
) -> ReturnCode {
    let Some(condition) = condition else {
        return RETCODE_BAD_PARAMETER;
    };

    let condition_ref = unsafe { condition.as_ref() };
    let rust_mask = mask_to_status_kinds(mask);

    match condition_ref.0.set_enabled_statuses(&rust_mask) {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Frees a StatusCondition object.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `condition` must point to a valid, initialized `StatusCondition` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_StatusCondition_free(
    condition: Option<NonNull<StatusCondition>>,
) -> ReturnCode {
    if let Some(condition) = condition {
        unsafe {
            drop(Box::from_raw(condition.as_ptr()));
        }
    }
    RETCODE_OK
}
