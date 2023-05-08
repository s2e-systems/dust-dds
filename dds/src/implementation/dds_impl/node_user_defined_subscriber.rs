use crate::{
    implementation::rtps::{
        endpoint::RtpsEndpoint,
        reader::RtpsReader,
        stateful_reader::RtpsStatefulReader,
        types::{
            EntityId, EntityKey, Guid, TopicKind, USER_DEFINED_READER_NO_KEY,
            USER_DEFINED_READER_WITH_KEY,
        },
    },
    infrastructure::{
        condition::StatusCondition,
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataReaderQos, QosKind, SubscriberQos},
        status::{SampleLostStatus, StatusKind},
        time::DURATION_ZERO,
    },
    subscription::subscriber_listener::SubscriberListener,
    topic_definition::type_support::{DdsDeserialize, DdsType},
};

use super::{
    any_data_reader_listener::AnyDataReaderListener,
    dds_data_reader::DdsDataReader,
    dds_domain_participant::{AnnounceKind, DdsDomainParticipant},
    node_domain_participant::DomainParticipantNode,
    node_user_defined_data_reader::UserDefinedDataReaderNode,
};

#[derive(PartialEq, Debug)]
pub struct UserDefinedSubscriberNode {
    this: Guid,
    parent: Guid,
}

impl UserDefinedSubscriberNode {
    pub fn new(this: Guid, parent: Guid) -> Self {
        Self { this, parent }
    }

    pub fn guid(&self) -> DdsResult<Guid> {
        Ok(self.this)
    }

    pub fn create_datareader<Foo>(
        &self,
        domain_participant: &mut DdsDomainParticipant,
        type_name: &'static str,
        topic_name: String,
        qos: QosKind<DataReaderQos>,
        a_listener: Option<Box<dyn AnyDataReaderListener + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsResult<UserDefinedDataReaderNode>
    where
        Foo: DdsType + for<'de> DdsDeserialize<'de>,
    {
        let default_unicast_locator_list =
            domain_participant.default_unicast_locator_list().to_vec();
        let default_multicast_locator_list =
            domain_participant.default_multicast_locator_list().to_vec();

        let qos = match qos {
            QosKind::Default => domain_participant
                .get_subscriber(self.this)
                .ok_or(DdsError::AlreadyDeleted)?
                .get_default_datareader_qos(),
            QosKind::Specific(q) => q,
        };
        qos.is_consistent()?;

        let entity_kind = match Foo::has_key() {
            true => USER_DEFINED_READER_WITH_KEY,
            false => USER_DEFINED_READER_NO_KEY,
        };

        let entity_key = EntityKey::new([
            <[u8; 3]>::from(
                domain_participant
                    .get_subscriber(self.this)
                    .ok_or(DdsError::AlreadyDeleted)?
                    .guid()
                    .entity_id()
                    .entity_key(),
            )[0],
            domain_participant
                .get_subscriber(self.this)
                .ok_or(DdsError::AlreadyDeleted)?
                .get_unique_reader_id(),
            0,
        ]);

        let entity_id = EntityId::new(entity_key, entity_kind);

        let guid = Guid::new(
            domain_participant
                .get_subscriber(self.this)
                .ok_or(DdsError::AlreadyDeleted)?
                .guid()
                .prefix(),
            entity_id,
        );

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

        let data_reader = DdsDataReader::new(rtps_reader, type_name, topic_name, a_listener, mask);

        domain_participant
            .get_subscriber_mut(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .stateful_data_reader_add(data_reader);

        let node = UserDefinedDataReaderNode::new(guid, self.this, self.parent);

        if domain_participant
            .get_subscriber(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .is_enabled()
            && domain_participant
                .get_subscriber(self.this)
                .ok_or(DdsError::AlreadyDeleted)?
                .get_qos()
                .entity_factory
                .autoenable_created_entities
        {
            node.enable(domain_participant)?;
        }

        Ok(node)
    }

    pub fn delete_datareader(
        &self,
        domain_participant: &mut DdsDomainParticipant,
        a_datareader_handle: InstanceHandle,
    ) -> DdsResult<()> {
        let data_reader = domain_participant
            .get_subscriber(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .stateful_data_reader_list()
            .iter()
            .find(|x| x.get_instance_handle() == a_datareader_handle)
            .ok_or_else(|| {
                DdsError::PreconditionNotMet(
                    "Data reader can only be deleted from its parent subscriber".to_string(),
                )
            })?
            .clone();

        domain_participant
            .get_subscriber_mut(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .stateful_data_reader_delete(a_datareader_handle);

        if data_reader.is_enabled() {
            domain_participant
                .announce_sender()
                .send(AnnounceKind::DeletedDataReader(
                    data_reader.get_instance_handle(),
                ))
                .ok();
        }

        Ok(())
    }

    pub fn lookup_datareader(
        &self,
        domain_participant: &DdsDomainParticipant,
        type_name: &str,
        topic_name: &str,
    ) -> DdsResult<Option<UserDefinedDataReaderNode>> {
        let reader = domain_participant
            .get_subscriber(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .stateful_data_reader_list()
            .iter()
            .find(|data_reader| {
                data_reader.get_topic_name() == topic_name
                    && data_reader.get_type_name() == type_name
            })
            .cloned()
            .ok_or_else(|| DdsError::PreconditionNotMet("Not found".to_string()))?;
        Ok(Some(UserDefinedDataReaderNode::new(
            reader.guid(),
            self.this,
            self.parent,
        )))
    }

    pub fn notify_datareaders(&self) -> DdsResult<()> {
        todo!()
    }

    pub fn get_participant(&self) -> DdsResult<DomainParticipantNode> {
        Ok(DomainParticipantNode::new(self.parent))
    }

    pub fn get_sample_lost_status(&self) -> DdsResult<SampleLostStatus> {
        todo!()
    }

    pub fn delete_contained_entities(
        &self,
        domain_participant: &mut DdsDomainParticipant,
    ) -> DdsResult<()> {
        for data_reader in domain_participant
            .get_subscriber_mut(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .stateful_data_reader_drain()
        {
            if data_reader.is_enabled() {
                todo!()
                // domain_participant
                //     .announce_sender()
                //     .send(AnnounceKind::DeletedDataReader(
                //         data_reader.get_instance_handle(),
                //     ))
                //     .ok();
            }
        }
        Ok(())
    }

    pub fn set_default_datareader_qos(
        &self,
        domain_participant: &DdsDomainParticipant,
        qos: QosKind<DataReaderQos>,
    ) -> DdsResult<()> {
        domain_participant
            .get_subscriber(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .set_default_datareader_qos(qos)
    }

    pub fn get_default_datareader_qos(
        &self,
        domain_participant: &DdsDomainParticipant,
    ) -> DdsResult<DataReaderQos> {
        Ok(domain_participant
            .get_subscriber(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_default_datareader_qos())
    }

    pub fn set_qos(
        &self,
        domain_participant: &DdsDomainParticipant,
        qos: QosKind<SubscriberQos>,
    ) -> DdsResult<()> {
        domain_participant
            .get_subscriber(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .set_qos(qos)
    }

    pub fn get_qos(&self, domain_participant: &DdsDomainParticipant) -> DdsResult<SubscriberQos> {
        Ok(domain_participant
            .get_subscriber(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_qos())
    }

    pub fn set_listener(
        &self,
        _a_listener: Option<Box<dyn SubscriberListener + Send + Sync>>,
        _mask: &[StatusKind],
    ) -> DdsResult<()> {
        todo!()
        // *self.0.get()?.get_status_listener_lock() = StatusListener::new(a_listener, mask);
        // Ok(())
    }

    pub fn get_status_changes(
        &self,
        domain_participant: &DdsDomainParticipant,
    ) -> DdsResult<Vec<StatusKind>> {
        Ok(domain_participant
            .get_subscriber(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_status_changes())
    }

    pub fn enable(&self, domain_participant: &DdsDomainParticipant) -> DdsResult<()> {
        let is_parent_enabled = domain_participant.is_enabled();
        if !is_parent_enabled {
            return Err(DdsError::PreconditionNotMet(
                "Parent participant is disabled".to_string(),
            ));
        }

        if !domain_participant
            .get_subscriber(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .is_enabled()
        {
            domain_participant
                .get_subscriber(self.this)
                .ok_or(DdsError::AlreadyDeleted)?
                .enable()?;

            if domain_participant
                .get_subscriber(self.this)
                .ok_or(DdsError::AlreadyDeleted)?
                .get_qos()
                .entity_factory
                .autoenable_created_entities
            {
                for data_reader in domain_participant
                    .get_subscriber(self.this)
                    .ok_or(DdsError::AlreadyDeleted)?
                    .stateful_data_reader_list()
                {
                    data_reader.enable()?;
                    // let topic = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_participant(
                    //     &self.0.parent().prefix(),
                    //     |dp| {
                    //         dp.unwrap()
                    //             .topic_list()
                    //             .into_iter()
                    //             .find(|t| {
                    //                 t.get_name() == data_reader.get_topic_name()
                    //                     && t.get_type_name() == data_reader.get_type_name()
                    //             })
                    //             .cloned()
                    //             .expect("Topic must exist")
                    //     },
                    // );

                    // THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_dcps_service(
                    //     &self.0.parent().prefix(),
                    //     |dcps| {
                    //         let discovered_reader_data = data_reader.as_discovered_reader_data(
                    //             &topic.get_qos(),
                    //             &self.0.get().unwrap().get_qos(),
                    //         );
                    //         domain_participant
                    //             .announce_sender()
                    //             .send(AnnounceKind::CreatedDataReader(discovered_reader_data))
                    //             .ok()
                    //     },
                    // );
                }
            }
        }

        Ok(())
    }

    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        Ok(self.this.into())
    }

    pub fn get_statuscondition(
        &self,
        domain_participant: &DdsDomainParticipant,
    ) -> DdsResult<StatusCondition> {
        Ok(StatusCondition::new(
            domain_participant
                .get_subscriber(self.this)
                .ok_or(DdsError::AlreadyDeleted)?
                .get_statuscondition(),
        ))
    }
}
