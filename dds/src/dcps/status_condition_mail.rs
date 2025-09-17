use crate::{
    dcps::{actor::MailHandler, status_condition::DcpsStatusCondition},
    infrastructure::status::StatusKind,
    runtime::{DdsRuntime, OneshotSend},
};
use alloc::vec::Vec;

pub enum DcpsStatusConditionMail<R: DdsRuntime> {
    GetStatusConditionEnabledStatuses {
        reply_sender: R::OneshotSender<Vec<StatusKind>>,
    },
    SetStatusConditionEnabledStatuses {
        status_mask: Vec<StatusKind>,
    },
    GetStatusConditionTriggerValue {
        reply_sender: R::OneshotSender<bool>,
    },
    AddCommunicationState {
        state: StatusKind,
    },
    RemoveCommunicationState {
        state: StatusKind,
    },
}

impl<R: DdsRuntime> MailHandler for DcpsStatusCondition<R> {
    type Mail = DcpsStatusConditionMail<R>;
    async fn handle(&mut self, message: DcpsStatusConditionMail<R>) {
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
                self.add_communication_state(state)
            }
            DcpsStatusConditionMail::RemoveCommunicationState { state } => {
                self.remove_communication_state(state)
            }
        }
    }
}
