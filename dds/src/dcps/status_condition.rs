use crate::{
    infrastructure::status::StatusKind,
    runtime::{ChannelSend, DdsRuntime},
};
use alloc::{vec, vec::Vec};

#[derive(Debug)]
pub struct DcpsStatusCondition<R: DdsRuntime> {
    enabled_statuses: Vec<StatusKind>,
    status_changes: Vec<StatusKind>,
    registered_notifications: Vec<R::ChannelSender<()>>,
}

impl<R: DdsRuntime> Default for DcpsStatusCondition<R> {
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

impl<R: DdsRuntime> DcpsStatusCondition<R> {
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

    pub async fn register_notification(&mut self) -> R::ChannelReceiver<()> {
        let (sender, receiver) = R::channel();
        if self.get_trigger_value() {
            sender.send(()).await.ok();
        } else {
            self.registered_notifications.push(sender);
        }
        receiver
    }
}
