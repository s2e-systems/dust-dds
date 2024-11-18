use crate::{
    implementation::{
        actor::ActorAddress,
        status_condition::status_condition_actor::{self, StatusConditionActor},
    },
    infrastructure::{error::DdsResult, status::StatusKind},
};

/// Async version of [`StatusCondition`](crate::infrastructure::condition::StatusCondition).
#[derive(Clone)]
pub struct StatusConditionAsync {
    address: ActorAddress<StatusConditionActor>,
}

impl StatusConditionAsync {
    pub(crate) fn new(address: ActorAddress<StatusConditionActor>) -> Self {
        Self { address }
    }
}

impl StatusConditionAsync {
    /// Async version of [`get_enabled_statuses`](crate::infrastructure::condition::StatusCondition::get_enabled_statuses).
    #[tracing::instrument(skip(self))]
    pub async fn get_enabled_statuses(&self) -> DdsResult<Vec<StatusKind>> {
        Ok(self
            .address
            .send_actor_mail(status_condition_actor::GetStatusConditionEnabledStatuses)?
            .receive_reply()
            .await)
    }

    /// Async version of [`set_enabled_statuses`](crate::infrastructure::condition::StatusCondition::set_enabled_statuses).
    #[tracing::instrument(skip(self))]
    pub async fn set_enabled_statuses(&self, mask: &[StatusKind]) -> DdsResult<()> {
        self.address
            .send_actor_mail(status_condition_actor::SetStatusConditionEnabledStatuses {
                status_mask: mask.to_vec(),
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
            .send_actor_mail(status_condition_actor::GetStatusConditionTriggerValue)?
            .receive_reply()
            .await)
    }
}
