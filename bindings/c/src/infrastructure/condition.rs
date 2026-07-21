use crate::infrastructure::error::{RETCODE_BAD_PARAMETER, RETCODE_OK, ReturnCode};
use std::ptr::NonNull;

/// cbindgen:opaque
pub struct DustDdsStatusCondition(pub(crate) dust_dds::condition::StatusCondition);

impl DustDdsStatusCondition {
    pub fn new(condition: dust_dds::condition::StatusCondition) -> Self {
        Self(condition)
    }

    pub fn inner(&self) -> &dust_dds::condition::StatusCondition {
        &self.0
    }
}

pub type DustDdsStatusMask = u32;

pub const DUST_DDS_STATUS_INCONSISTENT_TOPIC_STATUS: DustDdsStatusMask = 0x00000001;
pub const DUST_DDS_STATUS_OFFERED_DEADLINE_MISSED_STATUS: DustDdsStatusMask = 0x00000002;
pub const DUST_DDS_STATUS_REQUESTED_DEADLINE_MISSED_STATUS: DustDdsStatusMask = 0x00000004;
pub const DUST_DDS_STATUS_OFFERED_INCOMPATIBLE_QOS_STATUS: DustDdsStatusMask = 0x00000008;
pub const DUST_DDS_STATUS_REQUESTED_INCOMPATIBLE_QOS_STATUS: DustDdsStatusMask = 0x00000010;
pub const DUST_DDS_STATUS_SAMPLE_LOST_STATUS: DustDdsStatusMask = 0x00000020;
pub const DUST_DDS_STATUS_SAMPLE_REJECTED_STATUS: DustDdsStatusMask = 0x00000040;
pub const DUST_DDS_STATUS_DATA_ON_READERS_STATUS: DustDdsStatusMask = 0x00000080;
pub const DUST_DDS_STATUS_DATA_AVAILABLE_STATUS: DustDdsStatusMask = 0x00000100;
pub const DUST_DDS_STATUS_LIVELINESS_LOST_STATUS: DustDdsStatusMask = 0x00000200;
pub const DUST_DDS_STATUS_LIVELINESS_CHANGED_STATUS: DustDdsStatusMask = 0x00000400;
pub const DUST_DDS_STATUS_PUBLICATION_MATCHED_STATUS: DustDdsStatusMask = 0x00000800;
pub const DUST_DDS_STATUS_SUBSCRIPTION_MATCHED_STATUS: DustDdsStatusMask = 0x00001000;

fn mask_to_status_kinds(mask: DustDdsStatusMask) -> Vec<dust_dds::infrastructure::status::StatusKind> {
    let mut kinds = Vec::new();
    if mask & DUST_DDS_STATUS_INCONSISTENT_TOPIC_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::InconsistentTopic);
    }
    if mask & DUST_DDS_STATUS_OFFERED_DEADLINE_MISSED_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::OfferedDeadlineMissed);
    }
    if mask & DUST_DDS_STATUS_REQUESTED_DEADLINE_MISSED_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::RequestedDeadlineMissed);
    }
    if mask & DUST_DDS_STATUS_OFFERED_INCOMPATIBLE_QOS_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::OfferedIncompatibleQos);
    }
    if mask & DUST_DDS_STATUS_REQUESTED_INCOMPATIBLE_QOS_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::RequestedIncompatibleQos);
    }
    if mask & DUST_DDS_STATUS_SAMPLE_LOST_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::SampleLost);
    }
    if mask & DUST_DDS_STATUS_SAMPLE_REJECTED_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::SampleRejected);
    }
    if mask & DUST_DDS_STATUS_DATA_ON_READERS_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::DataOnReaders);
    }
    if mask & DUST_DDS_STATUS_DATA_AVAILABLE_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::DataAvailable);
    }
    if mask & DUST_DDS_STATUS_LIVELINESS_LOST_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::LivelinessLost);
    }
    if mask & DUST_DDS_STATUS_LIVELINESS_CHANGED_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::LivelinessChanged);
    }
    if mask & DUST_DDS_STATUS_PUBLICATION_MATCHED_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::PublicationMatched);
    }
    if mask & DUST_DDS_STATUS_SUBSCRIPTION_MATCHED_STATUS != 0 {
        kinds.push(dust_dds::infrastructure::status::StatusKind::SubscriptionMatched);
    }
    kinds
}

/// Defines the list of communication statuses that are taken into account to determine the trigger_value of the StatusCondition.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_status_condition_set_enabled_statuses(
    condition: Option<NonNull<DustDdsStatusCondition>>,
    mask: DustDdsStatusMask,
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_status_condition_free(
    condition: Option<NonNull<DustDdsStatusCondition>>,
) -> ReturnCode {
    if let Some(condition) = condition {
        unsafe {
            drop(Box::from_raw(condition.as_ptr()));
        }
    }
    RETCODE_OK
}
