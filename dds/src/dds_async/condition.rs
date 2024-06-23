use crate::{
    implementation::{
        actor::ActorAddress,
        actors::status_condition_actor::{self, StatusConditionActor},
        runtime::{executor::ExecutorHandle, timer::TimerHandle},
    },
    infrastructure::{error::DdsResult, status::StatusKind},
};

/// Async version of [`StatusCondition`](crate::infrastructure::condition::StatusCondition).
#[derive(Clone)]
pub struct StatusConditionAsync {
    address: ActorAddress<StatusConditionActor>,
    _executor_handle: ExecutorHandle,
    timer_handle: TimerHandle,
}

impl StatusConditionAsync {
    pub(crate) fn new(
        address: ActorAddress<StatusConditionActor>,
        executor_handle: ExecutorHandle,
        timer_handle: TimerHandle,
    ) -> Self {
        Self {
            address,
            _executor_handle: executor_handle,
            timer_handle,
        }
    }

    pub(crate) fn address(&self) -> &ActorAddress<StatusConditionActor> {
        &self.address
    }

    #[allow(dead_code)]
    pub(crate) fn executor_handle(&self) -> &ExecutorHandle {
        &self._executor_handle
    }

    pub(crate) fn timer_handle(&self) -> &TimerHandle {
        &self.timer_handle
    }
}

impl StatusConditionAsync {
    /// Async version of [`get_enabled_statuses`](crate::infrastructure::condition::StatusCondition::get_enabled_statuses).
    #[tracing::instrument(skip(self))]
    pub async fn get_enabled_statuses(&self) -> DdsResult<Vec<StatusKind>> {
        Ok(self
            .address
            .send_actor_mail(status_condition_actor::GetEnabledStatuses)?
            .receive_reply()
            .await)
    }

    /// Async version of [`set_enabled_statuses`](crate::infrastructure::condition::StatusCondition::set_enabled_statuses).
    #[tracing::instrument(skip(self))]
    pub async fn set_enabled_statuses(&self, mask: &[StatusKind]) -> DdsResult<()> {
        self.address
            .send_actor_mail(status_condition_actor::SetEnabledStatuses {
                mask: mask.to_vec(),
            })?
            .receive_reply()
            .await;
        Ok(())
    }

    /// Async version of [`get_entity`](crate::infrastructure::condition::StatusCondition::get_entity).
    #[tracing::instrument(skip(self))]
    pub async fn get_entity(&self) {
        todo!()
    }
}

impl StatusConditionAsync {
    /// Async version of [`get_trigger_value`](crate::infrastructure::condition::StatusCondition::get_trigger_value).
    #[tracing::instrument(skip(self))]
    pub async fn get_trigger_value(&self) -> DdsResult<bool> {
        Ok(self
            .address
            .send_actor_mail(status_condition_actor::GetTriggerValue)?
            .receive_reply()
            .await)
    }
}
