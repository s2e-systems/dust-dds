use std::future::Future;

use crate::{
    implementation::actor::{Mail, MailHandler},
    infrastructure::status::StatusKind,
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
    ) -> impl Future<Output = <AddCommunicationState as Mail>::Result> + Send {
        async move {
            self.status_changes.push(message.state);
        }
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
    ) -> impl Future<Output = <RemoveCommunicationState as Mail>::Result> + Send {
        async move {
            self.status_changes.retain(|x| x != &message.state);
        }
    }
}

pub struct GetEnabledStatuses;
impl Mail for GetEnabledStatuses {
    type Result = Vec<StatusKind>;
}
impl MailHandler<GetEnabledStatuses> for StatusConditionActor {
    fn handle(
        &mut self,
        _: GetEnabledStatuses,
    ) -> impl Future<Output = <GetEnabledStatuses as Mail>::Result> + Send {
        async move { self.enabled_statuses.clone() }
    }
}

pub struct SetEnabledStatuses {
    pub mask: Vec<StatusKind>,
}
impl Mail for SetEnabledStatuses {
    type Result = ();
}
impl MailHandler<SetEnabledStatuses> for StatusConditionActor {
    fn handle(
        &mut self,
        message: SetEnabledStatuses,
    ) -> impl Future<Output = <SetEnabledStatuses as Mail>::Result> + Send {
        async move {
            self.enabled_statuses = message.mask;
        }
    }
}

pub struct GetTriggerValue;
impl Mail for GetTriggerValue {
    type Result = bool;
}
impl MailHandler<GetTriggerValue> for StatusConditionActor {
    fn handle(
        &mut self,
        _message: GetTriggerValue,
    ) -> impl Future<Output = <GetTriggerValue as Mail>::Result> + Send {
        async move {
            for status in &self.status_changes {
                if self.enabled_statuses.contains(status) {
                    return true;
                }
            }
            false
        }
    }
}
