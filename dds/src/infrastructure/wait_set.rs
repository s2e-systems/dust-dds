use std::sync::{Arc, Condvar, Mutex};

use crate::infrastructure::{
    error::{DdsError, DdsResult},
    time::Duration,
};

use super::condition::StatusCondition;

/// Enumeration of the different Condition objects that can be associated with a [`WaitSet`].
#[derive(Clone)]
pub enum Condition {
    StatusCondition(StatusCondition),
}
impl Condition {
    pub fn get_trigger_value(&self) -> bool {
        match self {
            Condition::StatusCondition(c) => c.get_trigger_value(),
        }
    }
}

/// A [`WaitSet`] allows an application to wait until one or more of the attached [`Condition`] objects has a `trigger_value` of
/// [`true`] or else until the timeout expires. It is created by calling the [`WaitSet::new`] operation and is not necessarily
/// associated with a single [`DomainParticipant`](crate::domain::domain_participant::DomainParticipant) and could be used to
/// wait on [`Condition`] objects associated with different [`DomainParticipant`](crate::domain::domain_participant::DomainParticipant) objects.
pub struct WaitSet {
    conditions: Vec<Condition>,
    enabled: Mutex<()>,
    cvar: Arc<Condvar>,
}

impl Default for WaitSet {
    fn default() -> Self {
        Self {
            conditions: vec![],
            enabled: Mutex::new(()),
            cvar: Arc::new(Condvar::new()),
        }
    }
}

impl WaitSet {
    /// Create a new [`WaitSet`]
    pub fn new() -> Self {
        Self::default()
    }

    /// This operation allows an application thread to wait for the occurrence of certain conditions. If none of the conditions attached
    /// to the [`WaitSet`] have a `trigger_value` of [`true`], the wait operation will block suspending the calling thread.
    /// The operation returns the list of all the attached conditions that have a `trigger_value` of [`true`] (i.e., the conditions
    /// that unblocked the wait).
    /// This operation takes a `timeout` argument that specifies the maximum duration for the wait. It this duration is exceeded and
    /// none of the attached [`Condition`] objects is [`true`], wait will return with [`DdsError::Timeout`].
    /// It is not allowed for more than one application thread to be waiting on the same [`WaitSet`]. If the wait operation is invoked on a
    /// [`WaitSet`] that already has a thread blocking on it, the operation will return immediately with the value [`DdsError::PreconditionNotMet`].
    pub fn wait(&self, timeout: Duration) -> DdsResult<Vec<Condition>> {
        let enabled = self.enabled.lock().unwrap();
        let std_duration = std::time::Duration::new(timeout.sec() as u64, timeout.nanosec());

        let result = self.cvar.wait_timeout(enabled, std_duration).unwrap();

        if result.1.timed_out() {
            return Err(DdsError::Timeout);
        }

        Ok(self
            .conditions
            .iter()
            .filter(|x| x.get_trigger_value())
            .cloned()
            .collect())
    }

    /// Attaches a [`Condition`] to the [`WaitSet`].
    /// It is possible to attach a [`Condition`] on a WaitSet that is currently being waited upon (via the [`WaitSet::wait`] operation). In this case, if the
    /// [`Condition`] has a `trigger_value` of [`true`], then attaching the condition will unblock the [`WaitSet`].
    /// Adding a [`Condition`] that is already attached to the [`WaitSet`] has no effect.
    pub fn attach_condition(&mut self, cond: Condition) -> DdsResult<()> {
        match &cond {
            Condition::StatusCondition(c) => c.push_cvar(self.cvar.clone()),
        }
        self.conditions.push(cond);
        Ok(())
    }

    /// Detaches a [`Condition`] from the [`WaitSet`].
    /// If the [`Condition`] was not attached to the [`WaitSet`], the operation will return [`DdsError::PreconditionNotMet`].
    pub fn detach_condition(&self, _cond: Condition) -> DdsResult<()> {
        todo!()
    }

    /// This operation retrieves the list of attached conditions.
    pub fn get_conditions(&self) -> DdsResult<Vec<Condition>> {
        Ok(self.conditions.clone())
    }
}
