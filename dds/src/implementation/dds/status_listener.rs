use crate::{
    implementation::utils::shared_object::{DdsRwLock, DdsShared},
    infrastructure::{condition::StatusCondition, status::StatusKind},
};

use super::{
    nodes::{DataReaderNode, DataWriterNode, TopicNode},
    status_condition_impl::StatusConditionImpl,
};

pub struct StatusListener<T: ?Sized> {
    listener: Option<Box<T>>,
    status_kind: Vec<StatusKind>,
    status_condition: DdsShared<DdsRwLock<StatusConditionImpl>>,
}

impl<T: ?Sized> StatusListener<T> {
    pub fn new(listener: Option<Box<T>>, status_kind: &[StatusKind]) -> Self {
        Self {
            listener,
            status_kind: status_kind.to_vec(),
            status_condition: DdsShared::new(DdsRwLock::new(StatusConditionImpl::default())),
        }
    }

    pub fn _is_enabled(&self, status_kind: &StatusKind) -> bool {
        self.listener.is_some() && self.status_kind.contains(status_kind)
    }

    pub fn listener_mut(&mut self) -> &mut Option<Box<T>> {
        &mut self.listener
    }

    pub fn _add_communication_state(&self, state: StatusKind) {
        self.status_condition
            .write_lock()
            .add_communication_state(state)
    }

    pub fn _remove_communication_state(&self, state: StatusKind) {
        self.status_condition
            .write_lock()
            .remove_communication_state(state)
    }

    pub fn _get_status_condition(&self) -> StatusCondition {
        StatusCondition::_new(self.status_condition.clone())
    }

    pub fn _get_status_changes(&self) -> Vec<StatusKind> {
        self.status_condition.read_lock()._get_status_changes()
    }
}

pub enum ListenerTriggerKind {
    RequestedDeadlineMissed(DataReaderNode),
    OnDataAvailable(DataReaderNode),
    SubscriptionMatched(DataReaderNode),
    RequestedIncompatibleQos(DataReaderNode),
    OnSampleRejected(DataReaderNode),
    OnSampleLost(DataReaderNode),
    OfferedIncompatibleQos(DataWriterNode),
    PublicationMatched(DataWriterNode),
    InconsistentTopic(TopicNode),
}
