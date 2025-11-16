use crate::{
    dcps::{
        actor::MailHandler,
        channels::{mpsc::MpscReceiver, oneshot::OneshotSender},
        status_condition::DcpsStatusCondition,
    },
    infrastructure::status::StatusKind,
};
use alloc::vec::Vec;

pub enum DcpsStatusConditionMail {
    GetStatusConditionEnabledStatuses {
        reply_sender: OneshotSender<Vec<StatusKind>>,
    },
    SetStatusConditionEnabledStatuses {
        status_mask: Vec<StatusKind>,
    },
    GetStatusConditionTriggerValue {
        reply_sender: OneshotSender<bool>,
    },
    AddCommunicationState {
        state: StatusKind,
    },
    RemoveCommunicationState {
        state: StatusKind,
    },
    RegisterNotification {
        reply_sender: OneshotSender<MpscReceiver<()>>,
    },
}

impl MailHandler for DcpsStatusCondition {
    type Mail = DcpsStatusConditionMail;
    async fn handle(&mut self, message: DcpsStatusConditionMail) {
        match message {
            DcpsStatusConditionMail::GetStatusConditionEnabledStatuses { reply_sender } => {
                reply_sender.send(self.get_enabled_statuses())
            }
            DcpsStatusConditionMail::SetStatusConditionEnabledStatuses { status_mask } => {
                self.set_enabled_statuses(status_mask)
            }
            DcpsStatusConditionMail::GetStatusConditionTriggerValue { reply_sender } => {
                reply_sender.send(self.get_trigger_value())
            }
            DcpsStatusConditionMail::AddCommunicationState { state } => {
                self.add_communication_state(state).await
            }
            DcpsStatusConditionMail::RemoveCommunicationState { state } => {
                self.remove_communication_state(state)
            }
            DcpsStatusConditionMail::RegisterNotification { reply_sender } => {
                reply_sender.send(self.register_notification().await)
            }
        }
    }
}
