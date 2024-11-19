use crate::{
    implementation::{
        actor::Actor, listeners::subscriber_listener::SubscriberListenerThread,
        status_condition::status_condition_actor::StatusConditionActor,
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DataReaderQos, SubscriberQos},
        status::StatusKind,
    },
};

use super::data_reader::DataReaderActor;

pub struct SubscriberActor {
    instance_handle: InstanceHandle,
    qos: SubscriberQos,
    data_reader_list: Vec<DataReaderActor>,
    enabled: bool,
    default_data_reader_qos: DataReaderQos,
    status_condition: Actor<StatusConditionActor>,
    subscriber_listener_thread: Option<SubscriberListenerThread>,
    status_kind: Vec<StatusKind>,
}

impl SubscriberActor {
    pub fn new(
        instance_handle: InstanceHandle,
        qos: SubscriberQos,
        status_condition: Actor<StatusConditionActor>,
        subscriber_listener_thread: Option<SubscriberListenerThread>,
        status_kind: Vec<StatusKind>,
    ) -> Self {
        Self {
            instance_handle,
            qos,
            data_reader_list: Vec::new(),
            enabled: false,
            default_data_reader_qos: DataReaderQos::default(),
            status_condition,
            subscriber_listener_thread,
            status_kind,
        }
    }

    pub fn data_reader_list(&self) -> impl Iterator<Item = &DataReaderActor> {
        self.data_reader_list.iter()
    }

    pub fn data_reader_list_mut(&mut self) -> impl Iterator<Item = &mut DataReaderActor> {
        self.data_reader_list.iter_mut()
    }

    pub fn drain_data_reader_list(&mut self) -> impl Iterator<Item = DataReaderActor> + '_ {
        self.data_reader_list.drain(..)
    }

    pub fn insert_data_reader(&mut self, data_reader: DataReaderActor) {
        self.data_reader_list.push(data_reader);
    }

    pub fn remove_data_reader(&mut self, handle: InstanceHandle) -> Option<DataReaderActor> {
        let index = self
            .data_reader_list
            .iter()
            .position(|x| x.instance_handle() == handle)?;
        Some(self.data_reader_list.remove(index))
    }

    pub fn get_data_reader(&self, handle: InstanceHandle) -> Option<&DataReaderActor> {
        self.data_reader_list
            .iter()
            .find(|x| x.instance_handle() == handle)
    }

    pub fn get_mut_data_reader(&mut self, handle: InstanceHandle) -> Option<&mut DataReaderActor> {
        self.data_reader_list
            .iter_mut()
            .find(|x| x.instance_handle() == handle)
    }

    pub fn instance_handle(&self) -> InstanceHandle {
        self.instance_handle
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn default_data_reader_qos(&self) -> &DataReaderQos {
        &self.default_data_reader_qos
    }

    pub fn set_default_data_reader_qos(
        &mut self,
        default_data_reader_qos: DataReaderQos,
    ) -> DdsResult<()> {
        default_data_reader_qos.is_consistent()?;
        self.default_data_reader_qos = default_data_reader_qos;
        Ok(())
    }

    pub fn qos(&self) -> &SubscriberQos {
        &self.qos
    }

    pub fn set_qos(&mut self, qos: SubscriberQos) -> DdsResult<()> {
        if self.enabled {}
        self.qos = qos;
        Ok(())
    }

    pub fn set_listener(
        &mut self,
        a_listener: Option<SubscriberListenerThread>,
        status_kind: Vec<StatusKind>,
    ) {
        self.subscriber_listener_thread = a_listener;
        self.status_kind = status_kind;
    }

    pub fn status_condition(&self) -> &Actor<StatusConditionActor> {
        &self.status_condition
    }
}
