use crate::{
    domain::domain_participant::DustDdsDomainParticipant,
    infrastructure::error::{RETCODE_BAD_PARAMETER, RETCODE_ERROR, RETCODE_OK, ReturnCode},
};
use std::ptr::NonNull;

/// cbindgen:opaque
pub struct DustDdsTopicDescription(
    pub(crate) Box<dyn dust_dds::topic_definition::topic_description::TopicDescription>,
);

pub type TopicDescription = DustDdsTopicDescription;

impl DustDdsTopicDescription {
    pub fn new(
        topic_description: Box<dyn dust_dds::topic_definition::topic_description::TopicDescription>,
    ) -> Self {
        Self(topic_description)
    }

    pub fn inner(&self) -> &dyn dust_dds::topic_definition::topic_description::TopicDescription {
        self.0.as_ref()
    }
}

/// Gets the participant to which the TopicDescription belongs.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `topic_desc` must point to a valid, initialized `DustDdsTopicDescription` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_topic_description_get_participant(
    topic_desc: Option<NonNull<DustDdsTopicDescription>>,
) -> Option<NonNull<DustDdsDomainParticipant>> {
    let topic_desc = topic_desc?;
    let participant = unsafe { topic_desc.as_ref() }.inner().get_participant();
    NonNull::new(Box::into_raw(Box::new(DustDdsDomainParticipant::new(
        participant,
    ))))
}

/// Gets the type name.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `topic_desc` must point to a valid, initialized `DustDdsTopicDescription` instance.
/// - `value` must be a valid pointer to a `c_char` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_topic_description_get_type_name(
    topic_desc: Option<NonNull<DustDdsTopicDescription>>,
    value: *mut *mut std::os::raw::c_char,
) -> ReturnCode {
    let Some(topic_desc) = topic_desc else {
        return RETCODE_BAD_PARAMETER;
    };
    if value.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    let type_name = unsafe { topic_desc.as_ref() }.inner().get_type_name();
    let c_string = match std::ffi::CString::new(type_name) {
        Ok(cs) => cs,
        Err(_) => return RETCODE_ERROR,
    };
    unsafe { *value = c_string.into_raw() };
    RETCODE_OK
}

/// Gets the name.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `topic_desc` must point to a valid, initialized `DustDdsTopicDescription` instance.
/// - `value` must be a valid pointer to a `c_char` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_topic_description_get_name(
    topic_desc: Option<NonNull<DustDdsTopicDescription>>,
    value: *mut *mut std::os::raw::c_char,
) -> ReturnCode {
    let Some(topic_desc) = topic_desc else {
        return RETCODE_BAD_PARAMETER;
    };
    if value.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    let name = unsafe { topic_desc.as_ref() }.inner().get_name();
    let c_string = match std::ffi::CString::new(name) {
        Ok(cs) => cs,
        Err(_) => return RETCODE_ERROR,
    };
    unsafe { *value = c_string.into_raw() };
    RETCODE_OK
}

/// Frees a TopicDescription object.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `topic_desc` must point to a valid, initialized `DustDdsTopicDescription` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_topic_description_free(
    topic_desc: Option<NonNull<DustDdsTopicDescription>>,
) -> ReturnCode {
    if let Some(topic_desc) = topic_desc {
        unsafe {
            drop(Box::from_raw(topic_desc.as_ptr()));
        }
    }
    RETCODE_OK
}
