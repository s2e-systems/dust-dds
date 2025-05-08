use crate::{
    dcps::runtime::DdsRuntime,
    infrastructure::status::StatusKind,
    runtime::{
        actor::{Actor, MailHandler},
        oneshot::OneshotSender,
    },
};
use alloc::vec::Vec;

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

pub enum StatusConditionMail {
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
}

impl MailHandler for StatusConditionActor {
    type Mail = StatusConditionMail;
    async fn handle(&mut self, message: StatusConditionMail) {
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

impl<R: DdsRuntime> crate::dcps::status_condition::StatusCondition
    for Actor<R, StatusConditionActor>
{
    async fn add_state(&mut self, state: StatusKind) {
        self.send_actor_mail(StatusConditionMail::AddCommunicationState { state })
            .await;
    }

    async fn remove_state(&mut self, state: StatusKind) {
        self.send_actor_mail(StatusConditionMail::RemoveCommunicationState { state })
            .await;
    }
}
