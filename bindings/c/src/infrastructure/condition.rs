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


pub(crate) fn mask_to_status_kinds(
    mask: StatusMask,
) -> Vec<dust_dds::infrastructure::status::StatusKind> {
    let mut kinds = Vec::new();
    if mask & crate::infrastructure::status::INCONSISTENT_TOPIC_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::InconsistentTopic);
    }
    if mask & crate::infrastructure::status::OFFERED_DEADLINE_MISSED_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::OfferedDeadlineMissed);
    }
    if mask & crate::infrastructure::status::REQUESTED_DEADLINE_MISSED_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::RequestedDeadlineMissed);
    }
    if mask & crate::infrastructure::status::OFFERED_INCOMPATIBLE_QOS_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::OfferedIncompatibleQos);
    }
    if mask & crate::infrastructure::status::REQUESTED_INCOMPATIBLE_QOS_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::RequestedIncompatibleQos);
    }
    if mask & crate::infrastructure::status::SAMPLE_LOST_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::SampleLost);
    }
    if mask & crate::infrastructure::status::SAMPLE_REJECTED_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::SampleRejected);
    }
    if mask & crate::infrastructure::status::DATA_ON_READERS_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::DataOnReaders);
    }
    if mask & crate::infrastructure::status::DATA_AVAILABLE_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::DataAvailable);
    }
    if mask & crate::infrastructure::status::LIVELINESS_LOST_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::LivelinessLost);
    }
    if mask & crate::infrastructure::status::LIVELINESS_CHANGED_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::LivelinessChanged);
    }
    if mask & crate::infrastructure::status::PUBLICATION_MATCHED_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::PublicationMatched);
    }
    if mask & crate::infrastructure::status::SUBSCRIPTION_MATCHED_STATUS != 0 {
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
