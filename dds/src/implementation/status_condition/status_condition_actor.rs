use crate::{
    infrastructure::status::StatusKind,
    runtime::{actor::MailHandler, oneshot::OneshotSender},
};

#[derive(Debug)]
pub struct StatusConditionActor {
    enabled_statuses: Vec<StatusKind>,
    status_changes: Vec<StatusKind>,
}

impl Default for StatusConditionActor {
    fn default() -> Self {
        Self {
            enabled_statuses: vec![
                StatusKind::InconsistentTopic,
                StatusKind::OfferedDeadlineMissed,
                StatusKind::RequestedDeadlineMissed,
                StatusKind::OfferedIncompatibleQos,
                StatusKind::RequestedIncompatibleQos,
                StatusKind::SampleLost,
                StatusKind::SampleRejected,
                StatusKind::DataOnReaders,
                StatusKind::DataAvailable,
                StatusKind::LivelinessLost,
                StatusKind::LivelinessChanged,
                StatusKind::PublicationMatched,
                StatusKind::SubscriptionMatched,
            ],
            status_changes: Vec::new(),
        }
    }
}

impl StatusConditionActor {
    pub fn add_communication_state(&mut self, state: StatusKind) {
        self.status_changes.push(state);
    }

    pub fn remove_communication_state(&mut self, state: StatusKind) {
        self.status_changes.retain(|x| x != &state);
    }

    pub fn get_enabled_statuses(&self) -> Vec<StatusKind> {
        self.enabled_statuses.clone()
    }

    pub fn set_enabled_statuses(&mut self, mask: Vec<StatusKind>) {
        self.enabled_statuses = mask;
    }

    pub fn get_trigger_value(&self) -> bool {
        for status in &self.status_changes {
            if self.enabled_statuses.contains(status) {
                return true;
            }
        }
        false
    }
}

pub struct GetStatusConditionEnabledStatuses {
    pub reply_sender: OneshotSender<Vec<StatusKind>>,
}
impl MailHandler<GetStatusConditionEnabledStatuses> for StatusConditionActor {
    fn handle(&mut self, message: GetStatusConditionEnabledStatuses) {
        let status = self.get_enabled_statuses();
        message.reply_sender.send(status);
    }
}

pub struct SetStatusConditionEnabledStatuses {
    pub status_mask: Vec<StatusKind>,
}
impl MailHandler<SetStatusConditionEnabledStatuses> for StatusConditionActor {
    fn handle(&mut self, message: SetStatusConditionEnabledStatuses) {
        self.set_enabled_statuses(message.status_mask);
    }
}

pub struct GetStatusConditionTriggerValue {
    pub reply_sender: OneshotSender<bool>,
}
impl MailHandler<GetStatusConditionTriggerValue> for StatusConditionActor {
    fn handle(&mut self, message: GetStatusConditionTriggerValue) {
        let value = self.get_trigger_value();
        message.reply_sender.send(value);
    }
}

pub struct AddCommunicationState {
    pub state: StatusKind,
}
impl MailHandler<AddCommunicationState> for StatusConditionActor {
    fn handle(&mut self, message: AddCommunicationState) {
        self.add_communication_state(message.state);
    }
}

pub struct RemoveCommunicationState {
    pub state: StatusKind,
}
impl MailHandler<RemoveCommunicationState> for StatusConditionActor {
    fn handle(&mut self, message: RemoveCommunicationState) {
        self.remove_communication_state(message.state);
    }
}
