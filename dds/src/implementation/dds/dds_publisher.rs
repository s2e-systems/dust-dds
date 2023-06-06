use crate::{
    implementation::{
        rtps::{
            endpoint::RtpsEndpoint,
            group::RtpsGroup,
            messages::overall_structure::RtpsMessageWrite,
            stateful_writer::RtpsStatefulWriter,
            stateless_writer::RtpsStatelessWriter,
            types::{
                EntityId, EntityKey, Guid, Locator, TopicKind, USER_DEFINED_WRITER_NO_KEY,
                USER_DEFINED_WRITER_WITH_KEY,
            },
            writer::RtpsWriter,
        },
        utils::actor::{spawn_actor, Actor, ActorAddress},
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, QosKind},
        time::{Duration, DURATION_ZERO},
    },
    DdsType,
};

use super::dds_data_writer::DdsDataWriter;

pub struct DdsPublisher {
    qos: PublisherQos,
    rtps_group: RtpsGroup,
    stateless_data_writer_list: Vec<DdsDataWriter<RtpsStatelessWriter>>,
    stateful_data_writer_list: Vec<Actor<DdsDataWriter<RtpsStatefulWriter>>>,
    enabled: bool,
    user_defined_data_writer_counter: u8,
    default_datawriter_qos: DataWriterQos,
}

impl DdsPublisher {
    pub fn new(qos: PublisherQos, rtps_group: RtpsGroup) -> Self {
        Self {
            qos,
            rtps_group,
            stateless_data_writer_list: Vec::new(),
            stateful_data_writer_list: Vec::new(),
            enabled: false,
            user_defined_data_writer_counter: 0,
            default_datawriter_qos: DataWriterQos::default(),
        }
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn create_datawriter<Foo>(
        &mut self,
        topic_name: String,
        qos: QosKind<DataWriterQos>,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
        data_max_size_serialized: usize,
        user_defined_rtps_message_channel_sender: tokio::sync::mpsc::Sender<(
            RtpsMessageWrite,
            Vec<Locator>,
        )>,
    ) -> DdsResult<ActorAddress<DdsDataWriter<RtpsStatefulWriter>>>
    where
        Foo: DdsType,
    {
        let qos = match qos {
            QosKind::Default => self.default_datawriter_qos.clone(),
            QosKind::Specific(q) => q,
        };
        qos.is_consistent()?;

        let entity_kind = match Foo::has_key() {
            true => USER_DEFINED_WRITER_WITH_KEY,
            false => USER_DEFINED_WRITER_NO_KEY,
        };

        let entity_key = EntityKey::new([
            <[u8; 3]>::from(self.guid().entity_id().entity_key())[0],
            self.get_unique_writer_id(),
            0,
        ]);

        let entity_id = EntityId::new(entity_key, entity_kind);

        let guid = Guid::new(self.guid().prefix(), entity_id);

        let topic_kind = match Foo::has_key() {
            true => TopicKind::WithKey,
            false => TopicKind::NoKey,
        };

        let rtps_writer_impl = RtpsStatefulWriter::new(RtpsWriter::new(
            RtpsEndpoint::new(
                guid,
                topic_kind,
                &default_unicast_locator_list,
                &default_multicast_locator_list,
            ),
            true,
            Duration::new(0, 200_000_000),
            DURATION_ZERO,
            DURATION_ZERO,
            data_max_size_serialized,
            qos,
        ));

        let mut data_writer = DdsDataWriter::new(
            rtps_writer_impl,
            Foo::type_name(),
            topic_name,
            user_defined_rtps_message_channel_sender,
        );

        if self.enabled && self.qos.entity_factory.autoenable_created_entities {
            data_writer.enable();
        }

        let data_writer_actor = spawn_actor(data_writer);
        let data_writer_address = data_writer_actor.address();
        self.stateful_data_writer_list.push(data_writer_actor);

        Ok(data_writer_address)
    }

    pub fn delete_datawriter(&mut self, data_writer_guid: Guid) -> DdsResult<()> {
        todo!()
        // if publisher_guid != data_writer_parent_publisher_guid {
        //     return Err(DdsError::PreconditionNotMet(
        //         "Data writer can only be deleted from its parent publisher".to_string(),
        //     ));
        // }

        // let data_writer_guid = domain_participant
        //     .user_defined_publisher_list_mut()
        //     .iter_mut()
        //     .find(|p| p.guid() == publisher_guid)
        //     .ok_or(DdsError::AlreadyDeleted)?
        //     .stateful_data_writer_list()
        //     .iter()
        //     .find(|x| x.guid() == data_writer_guid)
        //     .ok_or(DdsError::AlreadyDeleted)?
        //     .guid();

        // // The writer creation is announced only on enabled so its deletion must be announced only if it is enabled
        // if domain_participant
        //     .user_defined_publisher_list_mut()
        //     .iter_mut()
        //     .find(|p| p.guid() == publisher_guid)
        //     .ok_or(DdsError::AlreadyDeleted)?
        //     .stateful_data_writer_list()
        //     .iter()
        //     .find(|x| x.guid() == data_writer_guid)
        //     .ok_or_else(|| {
        //         DdsError::PreconditionNotMet(
        //             "Data writer can only be deleted from its parent publisher".to_string(),
        //         )
        //     })?
        //     .is_enabled()
        // {
        //     domain_participant
        //         .announce_sender()
        //         .try_send(AnnounceKind::DeletedDataWriter(data_writer_guid.into()))
        //         .ok();
        // }

        // domain_participant
        //     .user_defined_publisher_list_mut()
        //     .iter_mut()
        //     .find(|p| p.guid() == publisher_guid)
        //     .ok_or(DdsError::AlreadyDeleted)?
        //     .stateful_datawriter_delete(InstanceHandle::from(data_writer_guid));

        // Ok(())
    }

    pub fn lookup_datawriter(
        &mut self,
        type_name: &'static str,
        topic_name: &str,
    ) -> DdsResult<Option<ActorAddress<DdsDataWriter<RtpsStatefulWriter>>>> {
        todo!()
        // Ok(domain_participant
        //     .get_publisher(publisher_guid)
        //     .ok_or(DdsError::AlreadyDeleted)?
        //     .stateful_data_writer_list()
        //     .iter()
        //     .find(|data_reader| {
        //         data_reader.get_topic_name() == topic_name
        //             && data_reader.get_type_name() == type_name
        //     })
        //     .map(|x| DataWriterNode::new(x.guid(), publisher_guid, domain_participant.guid())))
    }

    pub fn get_unique_writer_id(&mut self) -> u8 {
        let counter = self.user_defined_data_writer_counter;
        self.user_defined_data_writer_counter += 1;
        counter
    }

    pub fn delete_contained_entities(&mut self) {
        todo!()
        // for data_writer in self.stateful_data_writer_list.drain(..) {
        //     data_writer.0.
        // }
    }

    pub fn stateful_datawriter_add(
        &mut self,
        data_writer: Actor<DdsDataWriter<RtpsStatefulWriter>>,
    ) {
        self.stateful_data_writer_list.push(data_writer)
    }

    pub fn stateful_datawriter_drain(
        &mut self,
    ) -> std::vec::Drain<Actor<DdsDataWriter<RtpsStatefulWriter>>> {
        self.stateful_data_writer_list.drain(..)
    }

    pub fn stateful_datawriter_delete(&mut self, data_writer_handle: InstanceHandle) {
        todo!()
        // self.stateful_data_writer_list
        //     .retain(|x| InstanceHandle::from(x.guid()) != data_writer_handle);
    }

    pub fn stateful_data_writer_list(
        &self,
    ) -> Vec<ActorAddress<DdsDataWriter<RtpsStatefulWriter>>> {
        self.stateful_data_writer_list
            .iter()
            .map(|x| x.address().clone())
            .collect()
    }

    pub fn stateless_datawriter_add(&mut self, data_writer: DdsDataWriter<RtpsStatelessWriter>) {
        self.stateless_data_writer_list.push(data_writer)
    }

    pub fn _stateless_datawriter_drain(
        &mut self,
    ) -> std::vec::Drain<DdsDataWriter<RtpsStatelessWriter>> {
        self.stateless_data_writer_list.drain(..)
    }

    pub fn _stateless_datawriter_delete(&mut self, data_writer_handle: InstanceHandle) {
        self.stateless_data_writer_list
            .retain(|x| InstanceHandle::from(x.guid()) != data_writer_handle);
    }

    pub fn stateless_data_writer_list(&self) -> &[DdsDataWriter<RtpsStatelessWriter>] {
        &self.stateless_data_writer_list
    }

    pub fn stateless_data_writer_list_mut(&mut self) -> &mut [DdsDataWriter<RtpsStatelessWriter>] {
        &mut self.stateless_data_writer_list
    }

    pub fn get_data_writer(
        &self,
        data_writer_guid: Guid,
    ) -> Option<ActorAddress<DdsDataWriter<RtpsStatefulWriter>>> {
        todo!()
        // self.stateful_data_writer_list()
        //     .iter()
        //     .find(|dw| dw.guid() == data_writer_guid)
    }

    pub fn set_default_datawriter_qos(&mut self, qos: QosKind<DataWriterQos>) -> DdsResult<()> {
        match qos {
            QosKind::Default => self.default_datawriter_qos = DataWriterQos::default(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                self.default_datawriter_qos = q;
            }
        }
        Ok(())
    }

    pub fn get_default_datawriter_qos(&self) -> DataWriterQos {
        self.default_datawriter_qos.clone()
    }

    pub fn set_qos(&mut self, qos: QosKind<PublisherQos>) -> DdsResult<()> {
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

    pub fn get_qos(&self) -> PublisherQos {
        self.qos.clone()
    }

    pub fn guid(&self) -> Guid {
        self.rtps_group.guid()
    }

    pub fn get_instance_handle(&self) -> InstanceHandle {
        self.rtps_group.guid().into()
    }
}
