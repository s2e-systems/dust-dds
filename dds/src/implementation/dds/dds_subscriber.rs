use std::sync::mpsc::Sender;

use crate::{
    implementation::rtps::{
        group::RtpsGroup,
        messages::submessages::{
            DataFragSubmessage, DataSubmessage, GapSubmessage, HeartbeatFragSubmessage,
            HeartbeatSubmessage,
        },
        stateful_reader::RtpsStatefulReader,
        stateless_reader::RtpsStatelessReader,
        types::{Guid, GuidPrefix},
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DataReaderQos, QosKind, SubscriberQos, TopicQos},
        time::Time,
    },
};

use super::{
    dds_data_reader::DdsDataReader, message_receiver::MessageReceiver,
    status_listener::ListenerTriggerKind,
};

pub struct DdsSubscriber {
    qos: SubscriberQos,
    rtps_group: RtpsGroup,
    stateless_data_reader_list: Vec<DdsDataReader<RtpsStatelessReader>>,
    stateful_data_reader_list: Vec<DdsDataReader<RtpsStatefulReader>>,
    enabled: bool,
    user_defined_data_reader_counter: u8,
    default_data_reader_qos: DataReaderQos,
}

impl DdsSubscriber {
    pub fn new(qos: SubscriberQos, rtps_group: RtpsGroup) -> Self {
        DdsSubscriber {
            qos,
            rtps_group,
            stateless_data_reader_list: Vec::new(),
            stateful_data_reader_list: Vec::new(),
            enabled: false,
            user_defined_data_reader_counter: 0,
            default_data_reader_qos: Default::default(),
        }
    }

    pub fn guid(&self) -> Guid {
        self.rtps_group.guid()
    }

    pub fn copy_from_topic_qos(
        _a_datareader_qos: &mut DataReaderQos,
        _a_topic_qos: &TopicQos,
    ) -> DdsResult<()> {
        todo!()
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn get_qos(&self) -> SubscriberQos {
        self.qos.clone()
    }

    pub fn get_unique_reader_id(&mut self) -> u8 {
        let counter = self.user_defined_data_reader_counter;
        self.user_defined_data_reader_counter += 1;
        counter
    }

    pub fn stateless_data_reader_add(&mut self, data_reader: DdsDataReader<RtpsStatelessReader>) {
        self.stateless_data_reader_list.push(data_reader)
    }

    pub fn _stateless_data_reader_delete(&mut self, a_datareader_handle: InstanceHandle) {
        self.stateless_data_reader_list
            .retain(|x| x._get_instance_handle() != a_datareader_handle)
    }

    pub fn stateless_data_reader_list(&self) -> &[DdsDataReader<RtpsStatelessReader>] {
        &self.stateless_data_reader_list
    }

    pub fn stateless_data_reader_list_mut(&mut self) -> &mut [DdsDataReader<RtpsStatelessReader>] {
        &mut self.stateless_data_reader_list
    }

    pub fn get_stateless_data_reader(
        &self,
        data_reader: Guid,
    ) -> Option<&DdsDataReader<RtpsStatelessReader>> {
        self.stateless_data_reader_list
            .iter()
            .find(|s| s.guid() == data_reader)
    }

    pub fn get_stateless_data_reader_mut(
        &mut self,
        data_reader: Guid,
    ) -> Option<&mut DdsDataReader<RtpsStatelessReader>> {
        self.stateless_data_reader_list
            .iter_mut()
            .find(|s| s.guid() == data_reader)
    }

    pub fn stateful_data_reader_add(&mut self, data_reader: DdsDataReader<RtpsStatefulReader>) {
        self.stateful_data_reader_list.push(data_reader)
    }

    pub fn stateful_data_reader_delete(&mut self, datareader_guid: Guid) {
        self.stateful_data_reader_list
            .retain(|x| x.guid() != datareader_guid)
    }

    pub fn stateful_data_reader_list(&self) -> &[DdsDataReader<RtpsStatefulReader>] {
        &self.stateful_data_reader_list
    }

    pub fn stateful_data_reader_list_mut(&mut self) -> &mut [DdsDataReader<RtpsStatefulReader>] {
        &mut self.stateful_data_reader_list
    }

    pub fn get_stateful_data_reader(
        &self,
        data_reader: Guid,
    ) -> Option<&DdsDataReader<RtpsStatefulReader>> {
        self.stateful_data_reader_list
            .iter()
            .find(|s| s.guid() == data_reader)
    }

    pub fn get_stateful_data_reader_mut(
        &mut self,
        data_reader: Guid,
    ) -> Option<&mut DdsDataReader<RtpsStatefulReader>> {
        self.stateful_data_reader_list
            .iter_mut()
            .find(|s| s.guid() == data_reader)
    }

    pub fn stateful_data_reader_drain(
        &mut self,
    ) -> std::vec::Drain<DdsDataReader<RtpsStatefulReader>> {
        self.stateful_data_reader_list.drain(..)
    }

    pub fn set_default_datareader_qos(&mut self, qos: QosKind<DataReaderQos>) -> DdsResult<()> {
        match qos {
            QosKind::Default => self.default_data_reader_qos = DataReaderQos::default(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                self.default_data_reader_qos = q;
            }
        }
        Ok(())
    }

    pub fn get_default_datareader_qos(&self) -> DataReaderQos {
        self.default_data_reader_qos.clone()
    }

    pub fn update_communication_status(
        &mut self,
        now: Time,
        parent_participant_guid: Guid,
        listener_sender: &tokio::sync::mpsc::Sender<ListenerTriggerKind>,
    ) {
        let guid = self.guid();
        for data_reader in self.stateful_data_reader_list.iter_mut() {
            data_reader.update_communication_status(
                now,
                parent_participant_guid,
                guid,
                listener_sender,
            );
        }
    }

    pub fn set_qos(&mut self, qos: QosKind<SubscriberQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => Default::default(),
            QosKind::Specific(q) => q,
        };

        if self.enabled {
            self.qos.check_immutability(&qos)?;
        }

        self.qos = qos;

        Ok(())
    }

    pub fn enable(&mut self) -> DdsResult<()> {
        self.enabled = true;

        if self.qos.entity_factory.autoenable_created_entities {
            for data_reader in self.stateful_data_reader_list.iter_mut() {
                data_reader.enable()?;
            }
        }

        Ok(())
    }

    pub fn get_instance_handle(&self) -> InstanceHandle {
        self.rtps_group.guid().into()
    }

    pub fn on_heartbeat_submessage_received(
        &mut self,
        heartbeat_submessage: &HeartbeatSubmessage,
        source_guid_prefix: GuidPrefix,
    ) {
        for data_reader in self.stateful_data_reader_list.iter_mut() {
            data_reader.on_heartbeat_submessage_received(heartbeat_submessage, source_guid_prefix)
        }
    }

    pub fn on_heartbeat_frag_submessage_received(
        &mut self,
        heartbeat_frag_submessage: &HeartbeatFragSubmessage,
        source_guid_prefix: GuidPrefix,
    ) {
        for data_reader in self.stateful_data_reader_list.iter_mut() {
            data_reader.on_heartbeat_frag_submessage_received(
                heartbeat_frag_submessage,
                source_guid_prefix,
            )
        }
    }

    pub fn on_data_submessage_received(
        &mut self,
        data_submessage: &DataSubmessage<'_>,
        message_receiver: &MessageReceiver,
        parent_participant_guid: Guid,
        listener_sender: &Sender<ListenerTriggerKind>,
    ) {
        let guid = self.guid();
        for stateless_data_reader in self.stateless_data_reader_list.iter_mut() {
            stateless_data_reader.on_data_submessage_received(data_submessage, message_receiver);
        }

        for data_reader in self.stateful_data_reader_list.iter_mut() {
            data_reader.on_data_submessage_received(
                data_submessage,
                message_receiver,
                guid,
                parent_participant_guid,
                listener_sender,
            );
        }
    }

    pub fn on_data_frag_submessage_received(
        &mut self,
        data_frag_submessage: &DataFragSubmessage<'_>,
        message_receiver: &MessageReceiver,
        parent_participant_guid: Guid,
        listener_sender: &Sender<ListenerTriggerKind>,
    ) {
        let guid = self.guid();
        for data_reader in self.stateful_data_reader_list.iter_mut() {
            data_reader.on_data_frag_submessage_received(
                data_frag_submessage,
                message_receiver,
                guid,
                parent_participant_guid,
                listener_sender,
            );
        }
    }

    pub fn on_gap_submessage_received(
        &mut self,
        gap_submessage: &GapSubmessage,
        message_receiver: &MessageReceiver,
    ) {
        for data_reader in self.stateful_data_reader_list.iter_mut() {
            data_reader
                .on_gap_submessage_received(gap_submessage, message_receiver.source_guid_prefix());
        }
    }
}
