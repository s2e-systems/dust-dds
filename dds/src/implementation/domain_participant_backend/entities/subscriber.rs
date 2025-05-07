use crate::{
    dcps::data_reader::DataReaderEntity,
    implementation::{
        listeners::subscriber_listener::SubscriberListenerMail,
        status_condition::status_condition_actor::StatusConditionActor,
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DataReaderQos, SubscriberQos},
        status::StatusKind,
    },
    runtime::{actor::Actor, mpsc::MpscSender},
};

pub struct SubscriberEntity {
    instance_handle: InstanceHandle,
    qos: SubscriberQos,
    data_reader_list: Vec<DataReaderEntity<Actor<StatusConditionActor>>>,
    enabled: bool,
    default_data_reader_qos: DataReaderQos,
    status_condition: Actor<StatusConditionActor>,
    listener_sender: MpscSender<SubscriberListenerMail>,
    listener_mask: Vec<StatusKind>,
}

impl SubscriberEntity {
    pub fn new(
        instance_handle: InstanceHandle,
        qos: SubscriberQos,
        status_condition: Actor<StatusConditionActor>,
        listener_sender: MpscSender<SubscriberListenerMail>,
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

    pub fn data_reader_list(
        &self,
    ) -> impl Iterator<Item = &DataReaderEntity<Actor<StatusConditionActor>>> {
        self.data_reader_list.iter()
    }

    pub fn data_reader_list_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut DataReaderEntity<Actor<StatusConditionActor>>> {
        self.data_reader_list.iter_mut()
    }

    pub fn drain_data_reader_list(
        &mut self,
    ) -> impl Iterator<Item = DataReaderEntity<Actor<StatusConditionActor>>> + '_ {
        self.data_reader_list.drain(..)
    }

    pub fn insert_data_reader(
        &mut self,
        data_reader: DataReaderEntity<Actor<StatusConditionActor>>,
    ) {
        self.data_reader_list.push(data_reader);
    }

    pub fn remove_data_reader(
        &mut self,
        handle: InstanceHandle,
    ) -> Option<DataReaderEntity<Actor<StatusConditionActor>>> {
        let index = self
            .data_reader_list
            .iter()
            .position(|x| x.instance_handle() == handle)?;
        Some(self.data_reader_list.remove(index))
    }

    pub fn get_data_reader(
        &self,
        handle: InstanceHandle,
    ) -> Option<&DataReaderEntity<Actor<StatusConditionActor>>> {
        self.data_reader_list
            .iter()
            .find(|x| x.instance_handle() == handle)
    }

    pub fn get_mut_data_reader(
        &mut self,
        handle: InstanceHandle,
    ) -> Option<&mut DataReaderEntity<Actor<StatusConditionActor>>> {
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
        listener_sender: MpscSender<SubscriberListenerMail>,
        listener_mask: Vec<StatusKind>,
    ) {
        self.listener_sender = listener_sender;
        self.listener_mask = listener_mask;
    }

    pub fn status_condition(&self) -> &Actor<StatusConditionActor> {
        &self.status_condition
    }

    pub fn listener(&self) -> MpscSender<SubscriberListenerMail> {
        self.listener_sender.clone()
    }

    pub fn listener_mask(&self) -> &[StatusKind] {
        &self.listener_mask
    }
}
