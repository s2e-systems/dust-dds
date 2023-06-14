use std::sync::{Arc, Condvar};

use crate::infrastructure::{error::DdsResult, status::StatusKind};

pub struct StatusConditionImpl {
    enabled_statuses: Vec<StatusKind>,
    status_changes: Vec<StatusKind>,
    cvar_list: Vec<Arc<Condvar>>,
}

impl Default for StatusConditionImpl {
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
            cvar_list: Vec::new(),
        }
    }
}

impl StatusConditionImpl {
    pub fn get_enabled_statuses(&self) -> Vec<StatusKind> {
        self.enabled_statuses.clone()
    }

    pub fn set_enabled_statuses(&mut self, mask: &[StatusKind]) -> DdsResult<()> {
        self.enabled_statuses = mask.to_vec();
        Ok(())
    }

    pub fn get_trigger_value(&self) -> bool {
        for status in &self.status_changes {
            if self.enabled_statuses.contains(status) {
                return true;
            }
        }
        false
    }

    pub fn add_communication_state(&mut self, state: StatusKind) {
        self.status_changes.push(state);

        if self.get_trigger_value() {
            for cvar in self.cvar_list.iter() {
                cvar.notify_all();
            }
        }
    }

    pub fn remove_communication_state(&mut self, state: StatusKind) {
        self.status_changes.retain(|x| x != &state);
    }

    pub fn push_cvar(&mut self, cvar: Arc<Condvar>) {
        self.cvar_list.push(cvar)
    }

    pub fn _get_status_changes(&self) -> Vec<StatusKind> {
        self.status_changes.clone()
    }
}
