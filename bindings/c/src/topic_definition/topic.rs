#![allow(improper_ctypes_definitions)]

use crate::infrastructure::error::{RETCODE_BAD_PARAMETER, RETCODE_OK, ReturnCode};

/// cbindgen:opaque
pub struct DustDdsTopic(pub(crate) dust_dds::topic_definition::topic::Topic);

pub type Topic = DustDdsTopic;

impl DustDdsTopic {
    pub fn new(topic: dust_dds::topic_definition::topic::Topic) -> Self {
        Self(topic)
    }

    pub fn inner(&self) -> &dust_dds::topic_definition::topic::Topic {
        &self.0
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_topic_get_instance_handle(
    topic: Option<std::ptr::NonNull<DustDdsTopic>>,
    handle: *mut crate::infrastructure::status::InstanceHandle_t,
) -> ReturnCode {
    let Some(topic) = topic else {
        return RETCODE_BAD_PARAMETER;
    };
    if handle.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    unsafe {
        *handle = topic.as_ref().inner().get_instance_handle().into();
    }
    RETCODE_OK
}
