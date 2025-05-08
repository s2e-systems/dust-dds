use crate::{
    dcps::{
        actor::ActorAddress,
        runtime::{DdsRuntime, OneshotReceive},
        status_condition_actor::{StatusConditionActor, StatusConditionMail},
    },
    infrastructure::{error::DdsResult, status::StatusKind},
};

/// Async version of [`StatusCondition`](crate::infrastructure::condition::StatusCondition).
pub struct StatusConditionAsync<R: DdsRuntime> {
    address: ActorAddress<R, StatusConditionActor<R>>,
}

impl<R: DdsRuntime> Clone for StatusConditionAsync<R> {
    fn clone(&self) -> Self {
        Self {
            address: self.address.clone(),
        }
    }
}

impl<R: DdsRuntime> StatusConditionAsync<R> {
    pub(crate) fn new(address: ActorAddress<R, StatusConditionActor<R>>) -> Self {
        Self { address }
    }
}

impl<R: DdsRuntime> StatusConditionAsync<R> {
    /// Async version of [`get_enabled_statuses`](crate::infrastructure::condition::StatusCondition::get_enabled_statuses).
    #[tracing::instrument(skip(self))]
    pub async fn get_enabled_statuses(&self) -> DdsResult<Vec<StatusKind>> {
        let (reply_sender, mut reply_receiver) = R::oneshot();
        self.address
            .send_actor_mail(StatusConditionMail::GetStatusConditionEnabledStatuses {
                reply_sender,
            })
            .await?;
        Ok(reply_receiver.receive().await?)
    }

    /// Async version of [`set_enabled_statuses`](crate::infrastructure::condition::StatusCondition::set_enabled_statuses).
    #[tracing::instrument(skip(self))]
    pub async fn set_enabled_statuses(&self, mask: &[StatusKind]) -> DdsResult<()> {
        self.address
            .send_actor_mail(StatusConditionMail::SetStatusConditionEnabledStatuses {
                status_mask: mask.to_vec(),
            })
            .await?;
        Ok(())
    }

    /// Async version of [`get_entity`](crate::infrastructure::condition::StatusCondition::get_entity).
    #[tracing::instrument(skip(self))]
    pub async fn get_entity(&self) {
        todo!()
    }
}

impl<R: DdsRuntime> StatusConditionAsync<R> {
    /// Async version of [`get_trigger_value`](crate::infrastructure::condition::StatusCondition::get_trigger_value).
    #[tracing::instrument(skip(self))]
    pub async fn get_trigger_value(&self) -> DdsResult<bool> {
        let (reply_sender, mut reply_receiver) = R::oneshot();
        self.address
            .send_actor_mail(StatusConditionMail::GetStatusConditionTriggerValue { reply_sender })
            .await?;
        Ok(reply_receiver.receive().await?)
    }
}
