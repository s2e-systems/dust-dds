use std::time::Instant;

use crate::infrastructure::{
    error::{DdsError, DdsResult},
    time::Duration,
};

use super::condition::StatusConditionAsync;

#[derive(Clone)]
pub enum ConditionAsync {
    StatusCondition(StatusConditionAsync),
}
impl ConditionAsync {
    #[tracing::instrument(skip(self))]
    pub async fn get_trigger_value(&self) -> DdsResult<bool> {
        match self {
            ConditionAsync::StatusCondition(c) => c.get_trigger_value().await,
        }
    }
}

#[derive(Default)]
pub struct WaitSetAsync {
    conditions: Vec<ConditionAsync>,
}

impl WaitSetAsync {
    /// Create a new [`WaitSet`]
    #[tracing::instrument]
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
    #[tracing::instrument(skip(self))]
    pub async fn wait(&self, timeout: Duration) -> DdsResult<Vec<ConditionAsync>> {
        let start_time = Instant::now();

        while start_time.elapsed() < std::time::Duration::from(timeout) {
            let mut finished = false;
            let mut trigger_conditions = Vec::new();
            for condition in &self.conditions {
                if condition.get_trigger_value().await? {
                    trigger_conditions.push(condition.clone());
                    finished = true;
                }
            }

            if finished {
                return Ok(trigger_conditions);
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }

        Err(DdsError::Timeout)
    }

    /// Attaches a [`Condition`] to the [`WaitSet`].
    /// It is possible to attach a [`Condition`] on a WaitSet that is currently being waited upon (via the [`WaitSet::wait`] operation). In this case, if the
    /// [`Condition`] has a `trigger_value` of [`true`], then attaching the condition will unblock the [`WaitSet`].
    /// Adding a [`Condition`] that is already attached to the [`WaitSet`] has no effect.
    #[tracing::instrument(skip(self, cond))]
    pub async fn attach_condition(&mut self, cond: ConditionAsync) -> DdsResult<()> {
        self.conditions.push(cond);
        Ok(())
    }

    /// Detaches a [`Condition`] from the [`WaitSet`].
    /// If the [`Condition`] was not attached to the [`WaitSet`], the operation will return [`DdsError::PreconditionNotMet`].
    #[tracing::instrument(skip(self, _cond))]
    pub async fn detach_condition(&self, _cond: ConditionAsync) -> DdsResult<()> {
        todo!()
    }

    /// This operation retrieves the list of attached conditions.
    #[tracing::instrument(skip(self))]
    pub async fn get_conditions(&self) -> DdsResult<Vec<ConditionAsync>> {
        Ok(self.conditions.clone())
    }
}
