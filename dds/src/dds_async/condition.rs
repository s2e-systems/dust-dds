use crate::{
    dcps::{
        channels::{
            mpsc::{MpscReceiver, MpscSender},
            oneshot::oneshot,
        },
        dcps_mail::{DcpsMail, StatusConditionMail},
        status_condition::StatusConditionEntity,
    },
    infrastructure::{error::DdsResult, status::StatusKind},
};
use alloc::vec::Vec;

/// Async version of [`StatusCondition`](crate::infrastructure::condition::StatusCondition).
pub struct StatusConditionAsync {
    dcps_sender: MpscSender<DcpsMail>,
    entity: StatusConditionEntity,
}

impl Clone for StatusConditionAsync {
    fn clone(&self) -> Self {
        Self {
            dcps_sender: self.dcps_sender.clone(),
            entity: self.entity.clone(),
        }
    }
}

impl StatusConditionAsync {
    pub(crate) fn new(dcps_sender: MpscSender<DcpsMail>, entity: StatusConditionEntity) -> Self {
        Self {
            dcps_sender,
            entity,
        }
    }

    pub(crate) async fn register_notification(&self) -> DdsResult<MpscReceiver<()>> {
        let (reply_sender, reply_receiver) = oneshot();
        self.dcps_sender
            .send(DcpsMail::StatusCondition(
                StatusConditionMail::RegisterNotification {
                    entity: self.entity.clone(),
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.await?
    }
}

impl StatusConditionAsync {
    /// Async version of [`get_enabled_statuses`](crate::infrastructure::condition::StatusCondition::get_enabled_statuses).
    #[tracing::instrument(skip(self))]
    pub async fn get_enabled_statuses(&self) -> DdsResult<Vec<StatusKind>> {
        let (reply_sender, reply_receiver) = oneshot();
        self.dcps_sender
            .send(DcpsMail::StatusCondition(
                StatusConditionMail::GetStatusConditionEnabledStatuses {
                    entity: self.entity.clone(),
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.await?
    }

    /// Async version of [`set_enabled_statuses`](crate::infrastructure::condition::StatusCondition::set_enabled_statuses).
    #[tracing::instrument(skip(self))]
    pub async fn set_enabled_statuses(&self, mask: &[StatusKind]) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        self.dcps_sender
            .send(DcpsMail::StatusCondition(
                StatusConditionMail::SetStatusConditionEnabledStatuses {
                    entity: self.entity.clone(),
                    status_mask: mask.to_vec(),
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.await?
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
        self.dcps_sender
            .send(DcpsMail::StatusCondition(
                StatusConditionMail::GetStatusConditionTriggerValue {
                    entity: self.entity.clone(),
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.await?
    }
}
