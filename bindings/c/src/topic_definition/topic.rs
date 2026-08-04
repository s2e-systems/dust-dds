use crate::infrastructure::error::{RETCODE_BAD_PARAMETER, RETCODE_OK, ReturnCode};

/// cbindgen:opaque
pub struct DustDdsTopic(pub(crate) dust_dds::topic_definition::topic::Topic);

impl DustDdsTopic {
    pub fn new(topic: dust_dds::topic_definition::topic::Topic) -> Self {
        Self(topic)
    }

    pub fn inner(&self) -> &dust_dds::topic_definition::topic::Topic {
        &self.0
    }
}
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `topic` must point to a valid, initialized `DustDdsTopic` instance.
/// - `handle` must be a valid pointer to a `InstanceHandle_t` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_topic_get_instance_handle(
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
