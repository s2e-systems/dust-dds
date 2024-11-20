use crate::{
    infrastructure::status::StatusKind,
    runtime::actor::{Mail, MailHandler},
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

pub struct GetStatusConditionEnabledStatuses;
impl Mail for GetStatusConditionEnabledStatuses {
    type Result = Vec<StatusKind>;
}
impl MailHandler<GetStatusConditionEnabledStatuses> for StatusConditionActor {
    fn handle(
        &mut self,
        _: GetStatusConditionEnabledStatuses,
    ) -> <GetStatusConditionEnabledStatuses as Mail>::Result {
        self.enabled_statuses.clone()
    }
}

pub struct SetStatusConditionEnabledStatuses {
    pub status_mask: Vec<StatusKind>,
}
impl Mail for SetStatusConditionEnabledStatuses {
    type Result = ();
}
impl MailHandler<SetStatusConditionEnabledStatuses> for StatusConditionActor {
    fn handle(
        &mut self,
        message: SetStatusConditionEnabledStatuses,
    ) -> <SetStatusConditionEnabledStatuses as Mail>::Result {
        self.enabled_statuses = message.status_mask;
    }
}

pub struct GetStatusConditionTriggerValue;
impl Mail for GetStatusConditionTriggerValue {
    type Result = bool;
}
impl MailHandler<GetStatusConditionTriggerValue> for StatusConditionActor {
    fn handle(
        &mut self,
        _: GetStatusConditionTriggerValue,
    ) -> <GetStatusConditionTriggerValue as Mail>::Result {
        self.get_trigger_value()
    }
}

pub struct AddCommunicationState {
    pub state: StatusKind,
}
impl Mail for AddCommunicationState {
    type Result = ();
}
impl MailHandler<AddCommunicationState> for StatusConditionActor {
    fn handle(
        &mut self,
        message: AddCommunicationState,
    ) -> <AddCommunicationState as Mail>::Result {
        self.add_communication_state(message.state);
    }
}

pub struct RemoveCommunicationState {
    pub state: StatusKind,
}
impl Mail for RemoveCommunicationState {
    type Result = ();
}
impl MailHandler<RemoveCommunicationState> for StatusConditionActor {
    fn handle(
        &mut self,
        message: RemoveCommunicationState,
    ) -> <RemoveCommunicationState as Mail>::Result {
        self.remove_communication_state(message.state);
    }
}
