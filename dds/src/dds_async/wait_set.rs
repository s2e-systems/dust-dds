use super::condition::StatusConditionAsync;
use crate::{
    dcps::runtime::{Clock, DdsRuntime},
    infrastructure::{
        error::{DdsError, DdsResult},
        time::Duration,
    },
};
use alloc::{string::String, vec::Vec};

/// Async version of [`Condition`](crate::infrastructure::wait_set::Condition).
pub enum ConditionAsync<R: DdsRuntime> {
    /// Status condition variant
    StatusCondition(StatusConditionAsync<R>),
}

impl<R: DdsRuntime> Clone for ConditionAsync<R> {
    fn clone(&self) -> Self {
        match self {
            Self::StatusCondition(arg0) => Self::StatusCondition(arg0.clone()),
        }
    }
}

impl<R: DdsRuntime> ConditionAsync<R> {
    /// Async version of [`get_trigger_value`](crate::infrastructure::wait_set::Condition::get_trigger_value).
    #[tracing::instrument(skip(self))]
    pub async fn get_trigger_value(&self) -> DdsResult<bool> {
        match self {
            ConditionAsync::StatusCondition(c) => c.get_trigger_value().await,
        }
    }
}

/// Async version of [`WaitSet`](crate::infrastructure::wait_set::WaitSet).
pub struct WaitSetAsync<R: DdsRuntime> {
    conditions: Vec<ConditionAsync<R>>,
}

impl<R: DdsRuntime> Default for WaitSetAsync<R> {
    fn default() -> Self {
        Self {
            conditions: Default::default(),
        }
    }
}

impl<R: DdsRuntime> WaitSetAsync<R> {
    /// Create a new [`WaitSetAsync`]
    #[tracing::instrument]
    pub fn new() -> Self {
        Self::default()
    }

    /// Async version of [`wait`](crate::infrastructure::wait_set::WaitSet::wait).
    #[tracing::instrument(skip(self))]
    pub async fn wait(&self, timeout: Duration) -> DdsResult<Vec<ConditionAsync<R>>> {
        if self.conditions.is_empty() {
            return Err(DdsError::PreconditionNotMet(String::from(
                "WaitSet has no attached conditions",
            )));
        };

        let clock_handle = match &self.conditions[0] {
            ConditionAsync::StatusCondition(c) => c.clock_handle().clone(),
        };
        let start = clock_handle.now();
        while clock_handle.now() - start < timeout.into() {
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
            // timer_handle
            //     .sleep(std::time::Duration::from_millis(20))
            //     .await;
        }

        Err(DdsError::Timeout)
    }

    /// Async version of [`attach_condition`](crate::infrastructure::wait_set::WaitSet::attach_condition).
    #[tracing::instrument(skip(self, cond))]
    pub async fn attach_condition(&mut self, cond: ConditionAsync<R>) -> DdsResult<()> {
        self.conditions.push(cond);
        Ok(())
    }

    /// Async version of [`detach_condition`](crate::infrastructure::wait_set::WaitSet::detach_condition).
    #[tracing::instrument(skip(self, _cond))]
    pub async fn detach_condition(&self, _cond: ConditionAsync<R>) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`get_conditions`](crate::infrastructure::wait_set::WaitSet::get_conditions).
    #[tracing::instrument(skip(self))]
    pub async fn get_conditions(&self) -> DdsResult<Vec<ConditionAsync<R>>> {
        Ok(self.conditions.clone())
    }
}
