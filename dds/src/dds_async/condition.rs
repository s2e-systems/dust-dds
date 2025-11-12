use crate::{
    dcps::{
        actor::ActorAddress, status_condition::DcpsStatusCondition,
        status_condition_mail::DcpsStatusConditionMail,
    },
    infrastructure::{error::DdsResult, status::StatusKind},
    runtime::{DdsRuntime, OneshotReceive},
};
use alloc::vec::Vec;

/// Async version of [`StatusCondition`](crate::infrastructure::condition::StatusCondition).
pub struct StatusConditionAsync<R: DdsRuntime> {
    address: ActorAddress<R, DcpsStatusCondition<R>>,
    timer_handle: R::TimerHandle,
}

impl<R: DdsRuntime> Clone for StatusConditionAsync<R> {
    fn clone(&self) -> Self {
        Self {
            address: self.address.clone(),
            timer_handle: self.timer_handle.clone(),
        }
    }
}

impl<R: DdsRuntime> StatusConditionAsync<R> {
    pub(crate) fn new(
        address: ActorAddress<R, DcpsStatusCondition<R>>,
        timer_handle: R::TimerHandle,
    ) -> Self {
        Self {
            address,
            timer_handle,
        }
    }

    pub(crate) fn timer_handle(&self) -> &R::TimerHandle {
        &self.timer_handle
    }

    pub(crate) async fn register_notification(&self) -> DdsResult<R::ChannelReceiver<()>> {
        let (reply_sender, reply_receiver) = R::oneshot();
        self.address
            .send_actor_mail(DcpsStatusConditionMail::RegisterNotification { reply_sender })
            .await?;
        reply_receiver.receive().await
    }
}

impl<R: DdsRuntime> StatusConditionAsync<R> {
    /// Async version of [`get_enabled_statuses`](crate::infrastructure::condition::StatusCondition::get_enabled_statuses).
    #[tracing::instrument(skip(self))]
    pub async fn get_enabled_statuses(&self) -> DdsResult<Vec<StatusKind>> {
        let (reply_sender, reply_receiver) = R::oneshot();
        self.address
            .send_actor_mail(DcpsStatusConditionMail::GetStatusConditionEnabledStatuses {
                reply_sender,
            })
            .await?;
        reply_receiver.receive().await
    }

    /// Async version of [`set_enabled_statuses`](crate::infrastructure::condition::StatusCondition::set_enabled_statuses).
    #[tracing::instrument(skip(self))]
    pub async fn set_enabled_statuses(&self, mask: &[StatusKind]) -> DdsResult<()> {
        self.address
            .send_actor_mail(DcpsStatusConditionMail::SetStatusConditionEnabledStatuses {
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
        let (reply_sender, reply_receiver) = R::oneshot();
        self.address
            .send_actor_mail(DcpsStatusConditionMail::GetStatusConditionTriggerValue {
                reply_sender,
            })
            .await?;
        reply_receiver.receive().await
    }
}
