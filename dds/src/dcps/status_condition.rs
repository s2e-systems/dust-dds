use core::marker::PhantomData;

use crate::{infrastructure::status::StatusKind, runtime::DdsRuntime};
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
