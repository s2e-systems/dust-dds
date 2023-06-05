use super::{
    dds_data_reader::DdsDataReader, message_receiver::MessageReceiver,
    status_listener::ListenerTriggerKind,
};
use crate::{
    implementation::{
        dds_actor,
        rtps::{
            endpoint::RtpsEndpoint,
            group::RtpsGroup,
            messages::submessages::{
                data::DataSubmessageRead, data_frag::DataFragSubmessageRead,
                gap::GapSubmessageRead, heartbeat::HeartbeatSubmessageRead,
                heartbeat_frag::HeartbeatFragSubmessageRead,
            },
            reader::RtpsReader,
            stateful_reader::RtpsStatefulReader,
            stateless_reader::RtpsStatelessReader,
            types::{
                EntityId, EntityKey, Guid, GuidPrefix, Locator, TopicKind,
                USER_DEFINED_READER_NO_KEY, USER_DEFINED_READER_WITH_KEY,
            },
        },
        utils::actor::{spawn_actor, ActorAddress, OwnedActor},
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DataReaderQos, QosKind, SubscriberQos, TopicQos},
        time::{Time, DURATION_ZERO},
    },
    topic_definition::type_support::DdsDeserialize,
    DdsType,
};

pub struct DdsSubscriber {
    qos: SubscriberQos,
    rtps_group: RtpsGroup,
    stateless_data_reader_list: Vec<DdsDataReader<RtpsStatelessReader>>,
    stateful_data_reader_list: Vec<OwnedActor<DdsDataReader<RtpsStatefulReader>>>,
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

    pub fn create_datareader<Foo>(
        &mut self,
        topic_name: String,
        qos: QosKind<DataReaderQos>,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
    ) -> DdsResult<ActorAddress<DdsDataReader<RtpsStatefulReader>>>
    where
        Foo: DdsType + for<'de> DdsDeserialize<'de>,
    {
        let qos = match qos {
            QosKind::Default => self.default_data_reader_qos.clone(),
            QosKind::Specific(q) => q,
        };
        qos.is_consistent()?;

        let entity_kind = match Foo::has_key() {
            true => USER_DEFINED_READER_WITH_KEY,
            false => USER_DEFINED_READER_NO_KEY,
        };

        let entity_key = EntityKey::new([
            <[u8; 3]>::from(self.guid().entity_id().entity_key())[0],
            self.get_unique_reader_id(),
            0,
        ]);

        let entity_id = EntityId::new(entity_key, entity_kind);

        let guid = Guid::new(self.guid().prefix(), entity_id);

        let topic_kind = match Foo::has_key() {
            true => TopicKind::WithKey,
            false => TopicKind::NoKey,
        };

        let rtps_reader = RtpsStatefulReader::new(RtpsReader::new::<Foo>(
            RtpsEndpoint::new(
                guid,
                topic_kind,
                &default_unicast_locator_list,
                &default_multicast_locator_list,
            ),
            DURATION_ZERO,
            DURATION_ZERO,
            false,
            qos,
        ));

        let mut data_reader = DdsDataReader::new(rtps_reader, Foo::type_name(), topic_name);

        if self.is_enabled() && self.qos.entity_factory.autoenable_created_entities {
            data_reader.enable()?;
        }

        let reader_actor = spawn_actor(data_reader);
        let reader_address = reader_actor.address();
        self.stateful_data_reader_list.push(reader_actor);

        Ok(reader_address)
    }

    pub fn delete_datareader(&mut self, datareader_guid: Guid) -> DdsResult<()> {
        todo!()
        // let data_reader = domain_participant
        //     .get_subscriber(subscriber_guid)
        //     .ok_or(DdsError::AlreadyDeleted)?
        //     .stateful_data_reader_list()
        //     .iter()
        //     .find(|x| x.guid() == datareader_guid)
        //     .ok_or_else(|| {
        //         DdsError::PreconditionNotMet(
        //             "Data reader can only be deleted from its parent subscriber".to_string(),
        //         )
        //     })?;

        // if data_reader.is_enabled() {
        //     domain_participant
        //         .announce_sender()
        //         .try_send(AnnounceKind::DeletedDataReader(
        //             data_reader.get_instance_handle(),
        //         ))
        //         .ok();
        // }

        // domain_participant
        //     .get_subscriber_mut(subscriber_guid)
        //     .ok_or(DdsError::AlreadyDeleted)?
        //     .stateful_data_reader_delete(datareader_guid);

        // Ok(())
    }

    pub fn lookup_datareader(
        &self,
        type_name: &str,
        topic_name: &str,
    ) -> DdsResult<Option<ActorAddress<DdsDataReader<RtpsStatefulReader>>>> {
        todo!()
        // Ok(self
        //     .stateful_data_reader_list
        //     .iter()
        //     .find(|data_reader| {
        //         data_reader.get_topic_name() == topic_name
        //             && data_reader.get_type_name() == type_name
        //     })
        //     .map(|x| DataReaderNode::new(x.guid(), subscriber_guid, domain_participant.guid())))
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

    pub fn stateful_data_reader_add(
        &mut self,
        data_reader: OwnedActor<DdsDataReader<RtpsStatefulReader>>,
    ) {
        self.stateful_data_reader_list.push(data_reader)
    }

    pub fn stateful_data_reader_delete(&mut self, datareader_guid: Guid) {
        todo!()
        // self.stateful_data_reader_list
        //     .retain(|x| x.guid() != datareader_guid)
    }

    pub fn stateful_data_reader_list(&self) -> &[DdsDataReader<RtpsStatefulReader>] {
        todo!()
        // &self.stateful_data_reader_list
    }

    pub fn stateful_data_reader_list_mut(&mut self) -> &mut [DdsDataReader<RtpsStatefulReader>] {
        todo!()
        // &mut self.stateful_data_reader_list
    }

    pub fn get_stateful_data_reader(
        &self,
        data_reader: Guid,
    ) -> Option<&DdsDataReader<RtpsStatefulReader>> {
        todo!()
        // self.stateful_data_reader_list
        //     .iter()
        //     .find(|s| s.guid() == data_reader)
    }

    pub fn get_stateful_data_reader_mut(
        &mut self,
        data_reader: Guid,
    ) -> Option<&mut DdsDataReader<RtpsStatefulReader>> {
        todo!()
        // self.stateful_data_reader_list
        //     .iter_mut()
        //     .find(|s| s.guid() == data_reader)
    }

    pub fn stateful_data_reader_drain(
        &mut self,
    ) -> std::vec::Drain<DdsDataReader<RtpsStatefulReader>> {
        todo!()
        // self.stateful_data_reader_list.drain(..)
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
        todo!()
        // let guid = self.guid();
        // for data_reader in self.stateful_data_reader_list.iter_mut() {
        //     data_reader.update_communication_status(
        //         now,
        //         parent_participant_guid,
        //         guid,
        //         listener_sender,
        //     );
        // }
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
                data_reader
                    .address()
                    .send_blocking(dds_actor::data_reader::Enable)?;
            }
        }

        Ok(())
    }

    pub fn get_instance_handle(&self) -> InstanceHandle {
        self.rtps_group.guid().into()
    }

    pub fn on_heartbeat_submessage_received(
        &mut self,
        heartbeat_submessage: &HeartbeatSubmessageRead,
        source_guid_prefix: GuidPrefix,
    ) {
        todo!()
        // for data_reader in self.stateful_data_reader_list.iter_mut() {
        //     data_reader.on_heartbeat_submessage_received(heartbeat_submessage, source_guid_prefix)
        // }
    }

    pub fn on_heartbeat_frag_submessage_received(
        &mut self,
        heartbeat_frag_submessage: &HeartbeatFragSubmessageRead,
        source_guid_prefix: GuidPrefix,
    ) {
        todo!()
        // for data_reader in self.stateful_data_reader_list.iter_mut() {
        //     data_reader.on_heartbeat_frag_submessage_received(
        //         heartbeat_frag_submessage,
        //         source_guid_prefix,
        //     )
        // }
    }

    pub fn on_data_submessage_received(
        &mut self,
        data_submessage: &DataSubmessageRead<'_>,
        message_receiver: &MessageReceiver,
        parent_participant_guid: Guid,
        listener_sender: &tokio::sync::mpsc::Sender<ListenerTriggerKind>,
    ) {
        let guid = self.guid();
        for stateless_data_reader in self.stateless_data_reader_list.iter_mut() {
            stateless_data_reader.on_data_submessage_received(data_submessage, message_receiver);
        }
        todo!()

        // for data_reader in self.stateful_data_reader_list.iter_mut() {
        //     data_reader.on_data_submessage_received(
        //         data_submessage,
        //         message_receiver,
        //         guid,
        //         parent_participant_guid,
        //         listener_sender,
        //     );
        // }
    }

    pub fn on_data_frag_submessage_received(
        &mut self,
        data_frag_submessage: &DataFragSubmessageRead<'_>,
        message_receiver: &MessageReceiver,
        parent_participant_guid: Guid,
        listener_sender: &tokio::sync::mpsc::Sender<ListenerTriggerKind>,
    ) {
        let guid = self.guid();
        todo!()
        // for data_reader in self.stateful_data_reader_list.iter_mut() {
        //     data_reader.on_data_frag_submessage_received(
        //         data_frag_submessage,
        //         message_receiver,
        //         guid,
        //         parent_participant_guid,
        //         listener_sender,
        //     );
        // }
    }

    pub fn on_gap_submessage_received(
        &mut self,
        gap_submessage: &GapSubmessageRead,
        message_receiver: &MessageReceiver,
    ) {
        todo!()
        // for data_reader in self.stateful_data_reader_list.iter_mut() {
        //     data_reader
        //         .on_gap_submessage_received(gap_submessage, message_receiver.source_guid_prefix());
        // }
    }
}
