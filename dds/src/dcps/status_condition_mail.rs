use crate::{
    dcps::{actor::MailHandler, status_condition::StatusConditionActor},
    infrastructure::status::StatusKind,
    runtime::{DdsRuntime, OneshotSend},
};

pub enum StatusConditionMail<R: DdsRuntime> {
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

impl<R: DdsRuntime> MailHandler for StatusConditionActor<R> {
    type Mail = StatusConditionMail<R>;
    async fn handle(&mut self, message: StatusConditionMail<R>) {
        match message {
            StatusConditionMail::GetStatusConditionEnabledStatuses { reply_sender } => {
                reply_sender.send(self.get_enabled_statuses())
            }
            StatusConditionMail::SetStatusConditionEnabledStatuses { status_mask } => {
                self.set_enabled_statuses(status_mask)
            }
            StatusConditionMail::GetStatusConditionTriggerValue { reply_sender } => {
                reply_sender.send(self.get_trigger_value())
            }
            StatusConditionMail::AddCommunicationState { state } => {
                self.add_communication_state(state)
            }
            StatusConditionMail::RemoveCommunicationState { state } => {
                self.remove_communication_state(state)
            }
        }
    }
}
