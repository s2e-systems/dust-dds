use crate::{
    dcps::data_reader::DataReaderEntity,
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DataReaderQos, SubscriberQos},
        status::StatusKind,
    },
};
use alloc::vec::Vec;

use super::{
    actor::Actor, listeners::domain_participant_listener::ListenerMail, runtime::DdsRuntime,
    status_condition_actor::StatusConditionActor,
};

pub struct SubscriberEntity<R: DdsRuntime> {
    instance_handle: InstanceHandle,
    qos: SubscriberQos,
    data_reader_list: Vec<DataReaderEntity<R>>,
    enabled: bool,
    default_data_reader_qos: DataReaderQos,
    status_condition: Actor<R, StatusConditionActor<R>>,
    listener_sender: Option<R::ChannelSender<ListenerMail<R>>>,
    listener_mask: Vec<StatusKind>,
}

impl<R: DdsRuntime> SubscriberEntity<R> {
    pub fn new(
        instance_handle: InstanceHandle,
        qos: SubscriberQos,
        status_condition: Actor<R, StatusConditionActor<R>>,
        listener_sender: Option<R::ChannelSender<ListenerMail<R>>>,
        listener_mask: Vec<StatusKind>,
    ) -> Self {
        Self {
            instance_handle,
            qos,
            data_reader_list: Vec::new(),
            enabled: false,
            default_data_reader_qos: DataReaderQos::default(),
            status_condition,
            listener_sender,
            listener_mask,
        }
    }

    pub fn data_reader_list(&self) -> impl Iterator<Item = &DataReaderEntity<R>> {
        self.data_reader_list.iter()
    }

    pub fn data_reader_list_mut(&mut self) -> impl Iterator<Item = &mut DataReaderEntity<R>> {
        self.data_reader_list.iter_mut()
    }

    pub fn drain_data_reader_list(&mut self) -> impl Iterator<Item = DataReaderEntity<R>> + '_ {
        self.data_reader_list.drain(..)
    }

    pub fn insert_data_reader(&mut self, data_reader: DataReaderEntity<R>) {
        self.data_reader_list.push(data_reader);
    }

    pub fn remove_data_reader(&mut self, handle: InstanceHandle) -> Option<DataReaderEntity<R>> {
        let index = self
            .data_reader_list
            .iter()
            .position(|x| x.instance_handle() == handle)?;
        Some(self.data_reader_list.remove(index))
    }

    pub fn get_data_reader(&self, handle: InstanceHandle) -> Option<&DataReaderEntity<R>> {
        self.data_reader_list
            .iter()
            .find(|x| x.instance_handle() == handle)
    }

    pub fn get_mut_data_reader(
        &mut self,
        handle: InstanceHandle,
    ) -> Option<&mut DataReaderEntity<R>> {
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
        if self.enabled {
            self.qos.check_immutability(&qos)?;
        }
        self.qos = qos;
        Ok(())
    }

    pub fn set_listener(
        &mut self,
        listener_sender: Option<R::ChannelSender<ListenerMail<R>>>,
        listener_mask: Vec<StatusKind>,
    ) {
        self.listener_sender = listener_sender;
        self.listener_mask = listener_mask;
    }

    pub fn status_condition(&self) -> &Actor<R, StatusConditionActor<R>> {
        &self.status_condition
    }

    pub fn listener(&self) -> &Option<R::ChannelSender<ListenerMail<R>>> {
        &self.listener_sender
    }

    pub fn listener_mask(&self) -> &[StatusKind] {
        &self.listener_mask
    }
}
