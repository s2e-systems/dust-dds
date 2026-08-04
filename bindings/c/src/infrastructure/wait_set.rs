use crate::infrastructure::condition::StatusCondition;
use crate::infrastructure::error::{RETCODE_BAD_PARAMETER, RETCODE_OK, ReturnCode};
use std::ptr::NonNull;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Duration {
    pub sec: i32,
    pub nanosec: u32,
}

impl From<Duration> for dust_dds::infrastructure::time::Duration {
    fn from(value: Duration) -> Self {
        dust_dds::infrastructure::time::Duration::new(value.sec, value.nanosec)
    }
}

impl From<dust_dds::infrastructure::time::Duration> for Duration {
    fn from(value: dust_dds::infrastructure::time::Duration) -> Self {
        Duration {
            sec: value.sec(),
            nanosec: value.nanosec(),
        }
    }
}

/// cbindgen:opaque
pub struct WaitSet(pub(crate) dust_dds::wait_set::WaitSet);

impl WaitSet {
    pub fn new(wait_set: dust_dds::wait_set::WaitSet) -> Self {
        Self(wait_set)
    }

    pub fn inner(&self) -> &dust_dds::wait_set::WaitSet {
        &self.0
    }
}

/// Creates a new WaitSet.
///
/// # Safety
///
/// There are no special safety invariants to be observed when calling this function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_WaitSet_new() -> Option<NonNull<WaitSet>> {
    NonNull::new(Box::into_raw(Box::new(WaitSet(
        dust_dds::wait_set::WaitSet::new(),
    ))))
}

/// Frees a WaitSet object.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `wait_set` must point to a valid, initialized `WaitSet` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_WaitSet_free(wait_set: Option<NonNull<WaitSet>>) -> ReturnCode {
    if let Some(wait_set) = wait_set {
        unsafe {
            drop(Box::from_raw(wait_set.as_ptr()));
        }
    }
    RETCODE_OK
}

/// Attaches a Condition to the WaitSet.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `wait_set` must point to a valid, initialized `WaitSet` instance.
/// - `condition` must point to a valid, initialized `StatusCondition` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_WaitSet_attach_condition(
    wait_set: Option<NonNull<WaitSet>>,
    condition: Option<NonNull<StatusCondition>>,
) -> ReturnCode {
    let Some(mut wait_set) = wait_set else {
        return RETCODE_BAD_PARAMETER;
    };
    let Some(condition) = condition else {
        return RETCODE_BAD_PARAMETER;
    };

    let wait_set_ref = unsafe { wait_set.as_mut() };
    let condition_ref = unsafe { condition.as_ref() };

    let rust_cond = dust_dds::wait_set::Condition::StatusCondition(condition_ref.inner().clone());

    match wait_set_ref.0.attach_condition(rust_cond) {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Allows an application thread to wait for the occurrence of certain conditions.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `wait_set` must point to a valid, initialized `WaitSet` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_WaitSet_wait(
    wait_set: Option<NonNull<WaitSet>>,
    timeout: Duration,
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
