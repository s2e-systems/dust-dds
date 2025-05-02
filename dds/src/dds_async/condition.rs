use crate::{
    implementation::status_condition::status_condition_actor::{
        StatusConditionActor, StatusConditionMail,
    },
    infrastructure::{error::DdsResult, status::StatusKind},
    runtime::{actor::ActorAddress, oneshot::oneshot},
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
        let (reply_sender, reply_receiver) = oneshot();
        self.address
            .send_actor_mail(StatusConditionMail::GetStatusConditionEnabledStatuses {
                reply_sender,
            })?;
        Ok(reply_receiver.await?)
    }

    /// Async version of [`set_enabled_statuses`](crate::infrastructure::condition::StatusCondition::set_enabled_statuses).
    #[tracing::instrument(skip(self))]
    pub async fn set_enabled_statuses(&self, mask: &[StatusKind]) -> DdsResult<()> {
        self.address
            .send_actor_mail(StatusConditionMail::SetStatusConditionEnabledStatuses {
                status_mask: mask.to_vec(),
            })?;
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
        let (reply_sender, reply_receiver) = oneshot();
        self.address
            .send_actor_mail(StatusConditionMail::GetStatusConditionTriggerValue {
                reply_sender,
            })?;
        Ok(reply_receiver.await?)
    }
}
