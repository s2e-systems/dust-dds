use std::sync::{Arc, Condvar};

use crate::infrastructure::{error::DdsResult, status::StatusKind};

pub struct StatusConditionImpl {
    enabled_statuses: Vec<StatusKind>,
    communication_status: Vec<StatusKind>,
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
            communication_status: Vec::new(),
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
        for status in &self.communication_status {
            if self.enabled_statuses.contains(status) {
                return true;
            }
        }
        false
    }

    pub fn add_communication_state(&mut self, state: StatusKind) {
        self.communication_status.push(state);

        if self.get_trigger_value() {
            self.communication_status.retain(|x| x != &state);
            for cvar in self.cvar_list.iter() {
                cvar.notify_all();
            }
        }
    }

    pub fn remove_communication_state(&mut self, state: StatusKind) {
        self.communication_status.retain(|x| x != &state);
    }

    pub fn push_cvar(&mut self, cvar: Arc<Condvar>) {
        self.cvar_list.push(cvar)
    }
}
