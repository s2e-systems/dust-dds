use crate::{
    dcps::{
        actor::ActorAddress,
        channels::{mpsc::MpscReceiver, oneshot::oneshot},
        status_condition::DcpsStatusCondition,
        status_condition_mail::DcpsStatusConditionMail,
    },
    infrastructure::{error::DdsResult, status::StatusKind},
};
use alloc::vec::Vec;

/// Async version of [`StatusCondition`](crate::infrastructure::condition::StatusCondition).
pub struct StatusConditionAsync {
    address: ActorAddress<DcpsStatusCondition>,
}

impl Clone for StatusConditionAsync {
    fn clone(&self) -> Self {
        Self {
            address: self.address.clone(),
        }
    }
}

impl StatusConditionAsync {
    pub(crate) fn new(address: ActorAddress<DcpsStatusCondition>) -> Self {
        Self { address }
    }

    pub(crate) async fn register_notification(&self) -> DdsResult<MpscReceiver<()>> {
        let (reply_sender, reply_receiver) = oneshot();
        self.address
            .send_actor_mail(DcpsStatusConditionMail::RegisterNotification { reply_sender })
            .await?;
        reply_receiver.await
    }
}

impl StatusConditionAsync {
    /// Async version of [`get_enabled_statuses`](crate::infrastructure::condition::StatusCondition::get_enabled_statuses).
    #[tracing::instrument(skip(self))]
    pub async fn get_enabled_statuses(&self) -> DdsResult<Vec<StatusKind>> {
        let (reply_sender, reply_receiver) = oneshot();
        self.address
            .send_actor_mail(DcpsStatusConditionMail::GetStatusConditionEnabledStatuses {
                reply_sender,
            })
            .await?;
        reply_receiver.await
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

impl StatusConditionAsync {
    /// Async version of [`get_trigger_value`](crate::infrastructure::condition::StatusCondition::get_trigger_value).
    #[tracing::instrument(skip(self))]
    pub async fn get_trigger_value(&self) -> DdsResult<bool> {
        let (reply_sender, reply_receiver) = oneshot();
        self.address
            .send_actor_mail(DcpsStatusConditionMail::GetStatusConditionTriggerValue {
                reply_sender,
            })
            .await?;
        reply_receiver.await
    }
}
