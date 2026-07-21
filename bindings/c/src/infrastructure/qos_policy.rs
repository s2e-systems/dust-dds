use std::ptr::NonNull;

/// cbindgen:opaque
pub struct DustDdsUserDataQosPolicy(
    pub(crate) dust_dds::infrastructure::qos_policy::UserDataQosPolicy,
);

pub type UserDataQosPolicy = DustDdsUserDataQosPolicy;

impl DustDdsUserDataQosPolicy {
    pub fn new(policy: dust_dds::infrastructure::qos_policy::UserDataQosPolicy) -> Self {
        Self(policy)
    }

    pub fn inner(&self) -> &dust_dds::infrastructure::qos_policy::UserDataQosPolicy {
        &self.0
    }
}

/// Creates a default UserDataQosPolicy object.
/// Returns a raw pointer to DustDdsUserDataQosPolicy on success.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_user_data_qos_policy_default(
) -> Option<NonNull<DustDdsUserDataQosPolicy>> {
    NonNull::new(Box::into_raw(Box::new(DustDdsUserDataQosPolicy(
        dust_dds::infrastructure::qos_policy::UserDataQosPolicy::default(),
    ))))
}

/// Creates a UserDataQosPolicy object with a given byte array value.
/// Returns a raw pointer to DustDdsUserDataQosPolicy on success, or NULL if data is NULL and len > 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_user_data_qos_policy_new(
    data: *const u8,
    len: usize,
) -> Option<NonNull<DustDdsUserDataQosPolicy>> {
    let vec_data = if len == 0 {
        Vec::new()
    } else {
        if data.is_null() {
            return None;
        }
        unsafe { std::slice::from_raw_parts(data, len) }.to_vec()
    };

    NonNull::new(Box::into_raw(Box::new(DustDdsUserDataQosPolicy(
        dust_dds::infrastructure::qos_policy::UserDataQosPolicy { value: vec_data },
    ))))
}

/// Sets the byte array value of a UserDataQosPolicy object.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_user_data_qos_policy_set_value(
    policy: Option<NonNull<DustDdsUserDataQosPolicy>>,
    data: *const u8,
    len: usize,
) {
    let Some(mut policy) = policy else {
        return;
    };
    let vec_data = if len == 0 || data.is_null() {
        Vec::new()
    } else {
        unsafe { std::slice::from_raw_parts(data, len) }.to_vec()
    };
    unsafe { policy.as_mut() }.0.value = vec_data;
}

/// Gets the length of the byte array value of a UserDataQosPolicy object.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_user_data_qos_policy_get_value_length(
    policy: Option<NonNull<DustDdsUserDataQosPolicy>>,
) -> usize {
    let Some(policy) = policy else {
        return 0;
    };
    unsafe { policy.as_ref() }.0.value.len()
}

/// Copies the byte array value of a UserDataQosPolicy object into a buffer.
/// Returns the number of bytes copied.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_user_data_qos_policy_get_value(
    policy: Option<NonNull<DustDdsUserDataQosPolicy>>,
    buffer: *mut u8,
    max_len: usize,
) -> usize {
    let (Some(policy), false) = (policy, buffer.is_null()) else {
        return 0;
    };
    let val = &unsafe { policy.as_ref() }.0.value;
    let copy_len = val.len().min(max_len);
    unsafe {
        std::ptr::copy_nonoverlapping(val.as_ptr(), buffer, copy_len);
    }
    copy_len
}

/// Frees a UserDataQosPolicy object.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_user_data_qos_policy_free(
    policy: Option<NonNull<DustDdsUserDataQosPolicy>>,
) {
    if let Some(policy) = policy {
        unsafe {
            drop(Box::from_raw(policy.as_ptr()));
        }
    }
}
