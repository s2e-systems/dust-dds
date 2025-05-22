use core::marker::PhantomData;

use crate::{
    dcps::actor::{Actor, MailHandler},
    infrastructure::status::StatusKind,
    runtime::{DdsRuntime, OneshotSend},
};
use alloc::{vec, vec::Vec};

#[derive(Debug)]
pub struct StatusConditionActor<R: DdsRuntime> {
    enabled_statuses: Vec<StatusKind>,
    status_changes: Vec<StatusKind>,
    phantom: PhantomData<R>,
}

impl<R: DdsRuntime> Default for StatusConditionActor<R> {
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
            phantom: PhantomData,
        }
    }
}

impl<R: DdsRuntime> StatusConditionActor<R> {
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

impl<R: DdsRuntime> crate::dcps::status_condition::StatusCondition
    for Actor<R, StatusConditionActor<R>>
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
