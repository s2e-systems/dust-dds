use crate::{
    implementation::runtime::timer::TimerHandle,
    infrastructure::{
        error::{DdsError, DdsResult},
        time::Duration,
    },
};

use super::condition::StatusConditionAsync;

/// Async version of [`Condition`](crate::infrastructure::wait_set::Condition).
#[derive(Clone)]
pub enum ConditionAsync {
    /// Status condition variant
    StatusCondition(StatusConditionAsync),
}

impl ConditionAsync {
    pub(crate) fn timer_handle(&self) -> &TimerHandle {
        match self {
            ConditionAsync::StatusCondition(s) => s.timer_handle(),
        }
    }
}

impl ConditionAsync {
    /// Async version of [`get_trigger_value`](crate::infrastructure::wait_set::Condition::get_trigger_value).
    #[tracing::instrument(skip(self))]
    pub async fn get_trigger_value(&self) -> DdsResult<bool> {
        match self {
            ConditionAsync::StatusCondition(c) => c.get_trigger_value().await,
        }
    }
}

/// Async version of [`WaitSet`](crate::infrastructure::wait_set::WaitSet).
#[derive(Default)]
pub struct WaitSetAsync {
    conditions: Vec<ConditionAsync>,
}

impl WaitSetAsync {
    /// Create a new [`WaitSetAsync`]
    #[tracing::instrument]
    pub fn new() -> Self {
        Self::default()
    }

    /// Async version of [`wait`](crate::infrastructure::wait_set::WaitSet::wait).
    #[tracing::instrument(skip(self))]
    pub async fn wait(&self, timeout: Duration) -> DdsResult<Vec<ConditionAsync>> {
        if self.conditions.is_empty() {
            return Err(DdsError::PreconditionNotMet(
                "WaitSet has no attached conditions".to_string(),
            ));
        };

        let timer_handle = self.conditions[0].timer_handle().clone();
        let start = std::time::Instant::now();
        while std::time::Instant::now().duration_since(start) < timeout.into() {
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
            timer_handle
                .sleep(std::time::Duration::from_millis(20))
                .await;
        }

        Err(DdsError::Timeout)
    }

    /// Async version of [`attach_condition`](crate::infrastructure::wait_set::WaitSet::attach_condition).
    #[tracing::instrument(skip(self, cond))]
    pub async fn attach_condition(&mut self, cond: ConditionAsync) -> DdsResult<()> {
        self.conditions.push(cond);
        Ok(())
    }

    /// Async version of [`detach_condition`](crate::infrastructure::wait_set::WaitSet::detach_condition).
    #[tracing::instrument(skip(self, _cond))]
    pub async fn detach_condition(&self, _cond: ConditionAsync) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`get_conditions`](crate::infrastructure::wait_set::WaitSet::get_conditions).
    #[tracing::instrument(skip(self))]
    pub async fn get_conditions(&self) -> DdsResult<Vec<ConditionAsync>> {
        Ok(self.conditions.clone())
    }
}
