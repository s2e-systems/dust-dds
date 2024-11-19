use crate::{
    implementation::{
        actor::Actor, listeners::publisher_listener::PublisherListenerThread,
        status_condition::status_condition_actor::StatusConditionActor,
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos},
        status::StatusKind,
    },
};

use super::data_writer::DataWriterActor;

pub struct PublisherActor {
    qos: PublisherQos,
    instance_handle: InstanceHandle,
    data_writer_list: Vec<DataWriterActor>,
    enabled: bool,
    default_datawriter_qos: DataWriterQos,
    publisher_listener_thread: Option<PublisherListenerThread>,
    status_kind: Vec<StatusKind>,
    _status_condition: Actor<StatusConditionActor>,
}

impl PublisherActor {
    pub fn new(
        qos: PublisherQos,
        instance_handle: InstanceHandle,
        publisher_listener_thread: Option<PublisherListenerThread>,
        status_kind: Vec<StatusKind>,
        status_condition: Actor<StatusConditionActor>,
    ) -> Self {
        Self {
            qos,
            instance_handle,
            data_writer_list: Vec::new(),
            enabled: false,
            default_datawriter_qos: DataWriterQos::default(),
            publisher_listener_thread,
            status_kind,
            _status_condition: status_condition,
        }
    }

    pub fn data_writer_list(&self) -> impl Iterator<Item = &DataWriterActor> {
        self.data_writer_list.iter()
    }

    pub fn data_writer_list_mut(&mut self) -> impl Iterator<Item = &mut DataWriterActor> {
        self.data_writer_list.iter_mut()
    }

    pub fn drain_data_writer_list(&mut self) -> impl Iterator<Item = DataWriterActor> + '_ {
        self.data_writer_list.drain(..)
    }

    pub fn insert_data_writer(&mut self, data_writer: DataWriterActor) {
        self.data_writer_list.push(data_writer);
    }

    pub fn remove_data_writer(&mut self, handle: InstanceHandle) -> Option<DataWriterActor> {
        let index = self
            .data_writer_list
            .iter()
            .position(|x| x.instance_handle() == handle)?;
        Some(self.data_writer_list.remove(index))
    }

    pub fn get_data_writer(&self, handle: InstanceHandle) -> Option<&DataWriterActor> {
        self.data_writer_list
            .iter()
            .find(|x| x.instance_handle() == handle)
    }

    pub fn get_mut_data_writer(&mut self, handle: InstanceHandle) -> Option<&mut DataWriterActor> {
        self.data_writer_list
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

    pub fn default_datawriter_qos(&self) -> &DataWriterQos {
        &self.default_datawriter_qos
    }

    pub fn set_default_datawriter_qos(
        &mut self,
        default_datawriter_qos: DataWriterQos,
    ) -> DdsResult<()> {
        default_datawriter_qos.is_consistent()?;
        self.default_datawriter_qos = default_datawriter_qos;
        Ok(())
    }

    pub fn qos(&self) -> &PublisherQos {
        &self.qos
    }

    pub fn set_qos(&mut self, qos: PublisherQos) -> DdsResult<()> {
        if self.enabled {}
        self.qos = qos;
        Ok(())
    }

    pub fn set_listener(
        &mut self,
        a_listener: Option<PublisherListenerThread>,
        status_kind: Vec<StatusKind>,
    ) {
        self.publisher_listener_thread = a_listener;
        self.status_kind = status_kind;
    }
}
