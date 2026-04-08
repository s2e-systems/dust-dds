use crate::{
    dcps::channels::mpsc::{MpscReceiver, MpscSender, mpsc_channel},
    infrastructure::{instance::InstanceHandle, status::StatusKind},
};
use alloc::{vec, vec::Vec};

#[derive(Clone)]
pub enum StatusConditionEntity {
    Subscriber {
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
    },
    Topic {
        participant_handle: InstanceHandle,
        topic_handle: InstanceHandle,
    },
    DataWriter {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        writer_handle: InstanceHandle,
    },
    DataReader {
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        reader_handle: InstanceHandle,
    },
}

#[derive(Debug)]
pub struct DcpsStatusCondition {
    enabled_statuses: Vec<StatusKind>,
    status_changes: Vec<StatusKind>,
    registered_notifications: Vec<MpscSender<()>>,
}

impl Default for DcpsStatusCondition {
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
            registered_notifications: Vec::new(),
        }
    }
}

impl DcpsStatusCondition {
    pub async fn add_communication_state(&mut self, state: StatusKind) {
        self.status_changes.push(state);
        if self.get_trigger_value() {
            for w in self.registered_notifications.drain(..) {
                // Do not care if there is no channel waiting for response
                w.send(()).await.ok();
            }
        }
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

    pub async fn register_notification(&mut self) -> MpscReceiver<()> {
        let (sender, receiver) = mpsc_channel();
        if self.get_trigger_value() {
            sender.send(()).await.ok();
        } else {
            self.registered_notifications.push(sender);
        }
        receiver
    }
}
