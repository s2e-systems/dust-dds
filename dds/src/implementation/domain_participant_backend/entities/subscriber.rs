use crate::{
    implementation::{
        listeners::subscriber_listener::SubscriberListenerActor,
        status_condition::status_condition_actor::StatusConditionActor,
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DataReaderQos, SubscriberQos},
        status::StatusKind,
    },
    runtime::actor::Actor,
};

use super::data_reader::DataReaderEntity;

pub struct SubscriberEntity {
    instance_handle: InstanceHandle,
    qos: SubscriberQos,
    data_reader_list: Vec<DataReaderEntity>,
    enabled: bool,
    default_data_reader_qos: DataReaderQos,
    status_condition: Actor<StatusConditionActor>,
    listener: Option<Actor<SubscriberListenerActor>>,
    listener_mask: Vec<StatusKind>,
}

impl SubscriberEntity {
    pub fn new(
        instance_handle: InstanceHandle,
        qos: SubscriberQos,
        status_condition: Actor<StatusConditionActor>,
        listener: Option<Actor<SubscriberListenerActor>>,
        listener_mask: Vec<StatusKind>,
    ) -> Self {
        Self {
            instance_handle,
            qos,
            data_reader_list: Vec::new(),
            enabled: false,
            default_data_reader_qos: DataReaderQos::default(),
            status_condition,
            listener,
            listener_mask,
        }
    }

    pub fn data_reader_list(&self) -> impl Iterator<Item = &DataReaderEntity> {
        self.data_reader_list.iter()
    }

    pub fn data_reader_list_mut(&mut self) -> impl Iterator<Item = &mut DataReaderEntity> {
        self.data_reader_list.iter_mut()
    }

    pub fn drain_data_reader_list(&mut self) -> impl Iterator<Item = DataReaderEntity> + '_ {
        self.data_reader_list.drain(..)
    }

    pub fn insert_data_reader(&mut self, data_reader: DataReaderEntity) {
        self.data_reader_list.push(data_reader);
    }

    pub fn remove_data_reader(&mut self, handle: InstanceHandle) -> Option<DataReaderEntity> {
        let index = self
            .data_reader_list
            .iter()
            .position(|x| x.instance_handle() == handle)?;
        Some(self.data_reader_list.remove(index))
    }

    pub fn get_data_reader(&self, handle: InstanceHandle) -> Option<&DataReaderEntity> {
        self.data_reader_list
            .iter()
            .find(|x| x.instance_handle() == handle)
    }

    pub fn get_mut_data_reader(&mut self, handle: InstanceHandle) -> Option<&mut DataReaderEntity> {
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
        a_listener: Option<Actor<SubscriberListenerActor>>,
        status_kind: Vec<StatusKind>,
    ) {
        self.listener = a_listener;
        self.listener_mask = status_kind;
    }

    pub fn status_condition(&self) -> &Actor<StatusConditionActor> {
        &self.status_condition
    }

    pub fn listener(&self) -> Option<&Actor<SubscriberListenerActor>> {
        self.listener.as_ref()
    }

    pub fn listener_mask(&self) -> &[StatusKind] {
        &self.listener_mask
    }
}
