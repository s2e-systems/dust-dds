use super::{
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos},
        status::StatusKind,
    },
    listeners::domain_participant_listener::ListenerMail,
    runtime::DdsRuntime,
};
use alloc::vec::Vec;

use crate::dcps::data_writer::DataWriterEntity;

pub struct PublisherEntity<R: DdsRuntime> {
    qos: PublisherQos,
    instance_handle: InstanceHandle,
    data_writer_list: Vec<DataWriterEntity<R>>,
    enabled: bool,
    default_datawriter_qos: DataWriterQos,
    listener_sender: Option<R::ChannelSender<ListenerMail<R>>>,
    listener_mask: Vec<StatusKind>,
}

impl<R: DdsRuntime> PublisherEntity<R> {
    pub fn new(
        qos: PublisherQos,
        instance_handle: InstanceHandle,
        listener_sender: Option<R::ChannelSender<ListenerMail<R>>>,
        listener_mask: Vec<StatusKind>,
    ) -> Self {
        Self {
            qos,
            instance_handle,
            data_writer_list: Vec::new(),
            enabled: false,
            default_datawriter_qos: DataWriterQos::default(),
            listener_sender,
            listener_mask,
        }
    }

    pub fn data_writer_list(&self) -> impl Iterator<Item = &DataWriterEntity<R>> {
        self.data_writer_list.iter()
    }

    pub fn data_writer_list_mut(&mut self) -> impl Iterator<Item = &mut DataWriterEntity<R>> {
        self.data_writer_list.iter_mut()
    }

    pub fn drain_data_writer_list(&mut self) -> impl Iterator<Item = DataWriterEntity<R>> + '_ {
        self.data_writer_list.drain(..)
    }

    pub fn insert_data_writer(&mut self, data_writer: DataWriterEntity<R>) {
        self.data_writer_list.push(data_writer);
    }

    pub fn remove_data_writer(&mut self, handle: InstanceHandle) -> Option<DataWriterEntity<R>> {
        let index = self
            .data_writer_list
            .iter()
            .position(|x| x.instance_handle() == handle)?;
        Some(self.data_writer_list.remove(index))
    }

    pub fn get_data_writer(&self, handle: InstanceHandle) -> Option<&DataWriterEntity<R>> {
        self.data_writer_list
            .iter()
            .find(|x| x.instance_handle() == handle)
    }

    pub fn get_mut_data_writer(
        &mut self,
        handle: InstanceHandle,
    ) -> Option<&mut DataWriterEntity<R>> {
        self.data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle() == handle)
    }

    pub fn lookup_datawriter_mut(&mut self, topic_name: &str) -> Option<&mut DataWriterEntity<R>> {
        self.data_writer_list
            .iter_mut()
            .find(|x| x.topic_name() == topic_name)
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
        self.qos = qos;
        Ok(())
    }

    pub fn set_listener(
        &mut self,
        listener_sender: Option<R::ChannelSender<ListenerMail<R>>>,
        mask: Vec<StatusKind>,
    ) {
        self.listener_sender = listener_sender;
        self.listener_mask = mask;
    }

    pub fn listener_mask(&self) -> &[StatusKind] {
        &self.listener_mask
    }

    pub fn listener(&self) -> &Option<R::ChannelSender<ListenerMail<R>>> {
        &self.listener_sender
    }
}
