use crate::infrastructure::condition::DustDdsStatusCondition;
use crate::infrastructure::error::{RETCODE_BAD_PARAMETER, RETCODE_OK, ReturnCode};
use std::ptr::NonNull;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DustDdsDuration {
    pub sec: i32,
    pub nanosec: u32,
}

impl From<DustDdsDuration> for dust_dds::infrastructure::time::Duration {
    fn from(value: DustDdsDuration) -> Self {
        dust_dds::infrastructure::time::Duration::new(value.sec, value.nanosec)
    }
}

impl From<dust_dds::infrastructure::time::Duration> for DustDdsDuration {
    fn from(value: dust_dds::infrastructure::time::Duration) -> Self {
        DustDdsDuration {
            sec: value.sec(),
            nanosec: value.nanosec(),
        }
    }
}

/// cbindgen:opaque
pub struct DustDdsWaitSet(pub(crate) dust_dds::wait_set::WaitSet);

impl DustDdsWaitSet {
    pub fn new(wait_set: dust_dds::wait_set::WaitSet) -> Self {
        Self(wait_set)
    }

    pub fn inner(&self) -> &dust_dds::wait_set::WaitSet {
        &self.0
    }
}

/// Creates a new WaitSet.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_wait_set_new() -> Option<NonNull<DustDdsWaitSet>> {
    NonNull::new(Box::into_raw(Box::new(DustDdsWaitSet(
        dust_dds::wait_set::WaitSet::new(),
    ))))
}

/// Frees a WaitSet object.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_wait_set_free(
    wait_set: Option<NonNull<DustDdsWaitSet>>,
) -> ReturnCode {
    if let Some(wait_set) = wait_set {
        unsafe {
            drop(Box::from_raw(wait_set.as_ptr()));
        }
    }
    RETCODE_OK
}

/// Attaches a Condition to the WaitSet.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_wait_set_attach_condition(
    wait_set: Option<NonNull<DustDdsWaitSet>>,
    condition: Option<NonNull<DustDdsStatusCondition>>,
) -> ReturnCode {
    let Some(mut wait_set) = wait_set else {
        return RETCODE_BAD_PARAMETER;
    };
    let Some(condition) = condition else {
        return RETCODE_BAD_PARAMETER;
    };

    let wait_set_ref = unsafe { wait_set.as_mut() };
    let condition_ref = unsafe { condition.as_ref() };

    let rust_cond =
        dust_dds::wait_set::Condition::StatusCondition(condition_ref.inner().clone());

    match wait_set_ref.0.attach_condition(rust_cond) {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Allows an application thread to wait for the occurrence of certain conditions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_wait_set_wait(
    wait_set: Option<NonNull<DustDdsWaitSet>>,
    timeout: DustDdsDuration,
) -> ReturnCode {
    let Some(wait_set) = wait_set else {
        return RETCODE_BAD_PARAMETER;
    };

    let wait_set_ref = unsafe { wait_set.as_ref() };

    match wait_set_ref.0.wait(timeout.into()) {
        Ok(_) => RETCODE_OK,
        Err(e) => e.into(),
    }
}
