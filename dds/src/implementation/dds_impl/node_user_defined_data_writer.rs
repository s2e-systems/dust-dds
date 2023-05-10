use crate::{
    builtin_topics::SubscriptionBuiltinTopicData,
    implementation::{
        data_representation_builtin_endpoints::discovered_writer_data::{
            DiscoveredWriterData, WriterProxy,
        },
        rtps::types::Guid,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, QosKind},
        status::{
            LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
            PublicationMatchedStatus,
        },
        time::Time,
    },
    publication::data_writer::AnyDataWriter,
    topic_definition::type_support::{DdsSerialize, DdsSerializedKey, DdsType},
};

use super::{
    dds_domain_participant::DdsDomainParticipant,
    node_user_defined_publisher::UserDefinedPublisherNode,
    node_user_defined_topic::UserDefinedTopicNode,
};

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub struct UserDefinedDataWriterNode {
    this: Guid,
    parent_publisher: Guid,
    parent_participant: Guid,
}

impl UserDefinedDataWriterNode {
    pub fn new(this: Guid, parent_publisher: Guid, parent_participant: Guid) -> Self {
        Self {
            this,
            parent_publisher,
            parent_participant,
        }
    }

    pub fn guid(&self) -> Guid {
        self.this
    }

    pub fn parent_publisher(&self) -> Guid {
        self.parent_publisher
    }

    pub fn parent_participant(&self) -> Guid {
        self.parent_participant
    }

    pub fn register_instance_w_timestamp(
        &self,
        _instance_serialized_key: DdsSerializedKey,
        _timestamp: Time,
    ) -> DdsResult<Option<InstanceHandle>> {
        // self.0
        //     .get()?
        //     .register_instance_w_timestamp(instance_serialized_key, timestamp)
        todo!()
    }

    pub fn unregister_instance_w_timestamp(
        &self,
        domain_participant: &DdsDomainParticipant,
        instance_serialized_key: Vec<u8>,
        handle: InstanceHandle,
        timestamp: Time,
    ) -> DdsResult<()> {
        domain_participant
            .get_publisher(self.parent_publisher)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_data_writer(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .unregister_instance_w_timestamp(instance_serialized_key, handle, timestamp)
    }

    pub fn get_key_value<Foo>(
        &self,
        _key_holder: &mut Foo,
        _handle: InstanceHandle,
    ) -> DdsResult<()>
    where
        Foo: DdsType,
    {
        // self.0.get()?.get_key_value(key_holder, handle)
        todo!()
    }

    pub fn lookup_instance(
        &self,
        domain_participant: &DdsDomainParticipant,
        instance_serialized_key: DdsSerializedKey,
    ) -> DdsResult<Option<InstanceHandle>> {
        domain_participant
            .get_publisher(self.parent_publisher)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_data_writer(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .lookup_instance(instance_serialized_key)
    }

    pub fn write_w_timestamp(
        &self,
        domain_participant: &DdsDomainParticipant,
        serialized_data: Vec<u8>,
        instance_serialized_key: DdsSerializedKey,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()> {
        let is_parent_enabled = domain_participant
            .get_publisher(self.parent_publisher)
            .ok_or(DdsError::AlreadyDeleted)?
            .is_enabled();

        if !is_parent_enabled {
            return Err(DdsError::NotEnabled);
        }

        domain_participant
            .get_publisher(self.parent_publisher)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_data_writer(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .write_w_timestamp(serialized_data, instance_serialized_key, handle, timestamp)?;

        domain_participant
            .user_defined_data_send_condvar()
            .notify_all();

        Ok(())
    }

    pub fn dispose_w_timestamp(
        &self,
        domain_participant: &DdsDomainParticipant,
        instance_serialized_key: Vec<u8>,
        handle: InstanceHandle,
        timestamp: Time,
    ) -> DdsResult<()> {
        domain_participant
            .get_publisher(self.parent_publisher)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_data_writer(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .dispose_w_timestamp(instance_serialized_key, handle, timestamp)
    }

    pub fn are_all_changes_acknowledge(
        &self,
        domain_participant: &DdsDomainParticipant,
    ) -> DdsResult<bool> {
        let is_parent_enabled = domain_participant
            .get_publisher(self.parent_publisher)
            .ok_or(DdsError::AlreadyDeleted)?
            .is_enabled();

        if !is_parent_enabled {
            return Err(DdsError::NotEnabled);
        }

        if domain_participant
            .get_publisher(self.parent_publisher)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_data_writer(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .is_enabled()
        {
            Ok(domain_participant
                .get_publisher(self.parent_publisher)
                .ok_or(DdsError::AlreadyDeleted)?
                .get_data_writer(self.this)
                .ok_or(DdsError::AlreadyDeleted)?
                .are_all_changes_acknowledge())
        } else {
            Err(DdsError::NotEnabled)
        }
    }

    pub fn get_liveliness_lost_status(
        &self,
        domain_participant: &DdsDomainParticipant,
    ) -> DdsResult<LivelinessLostStatus> {
        Ok(domain_participant
            .get_publisher(self.parent_publisher)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_data_writer(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_liveliness_lost_status())
    }

    pub fn get_offered_deadline_missed_status(
        &self,
        domain_participant: &DdsDomainParticipant,
    ) -> DdsResult<OfferedDeadlineMissedStatus> {
        Ok(domain_participant
            .get_publisher(self.parent_publisher)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_data_writer(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_offered_deadline_missed_status())
    }

    pub fn get_offered_incompatible_qos_status(
        &self,
        domain_participant: &DdsDomainParticipant,
    ) -> DdsResult<OfferedIncompatibleQosStatus> {
        Ok(domain_participant
            .get_publisher(self.parent_publisher)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_data_writer(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_offered_incompatible_qos_status())
    }

    pub fn get_publication_matched_status(
        &self,
        domain_participant: &DdsDomainParticipant,
    ) -> DdsResult<PublicationMatchedStatus> {
        Ok(domain_participant
            .get_publisher(self.parent_publisher)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_data_writer(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_publication_matched_status())
    }

    pub fn get_topic(&self) -> DdsResult<UserDefinedTopicNode> {
        // let data_writer = self.0.get()?;

        // let topic =
        //     THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_participant(&self.0.parent().prefix(), |dp| {
        //         dp.unwrap()
        //             .topic_list()
        //             .into_iter()
        //             .find(|t| {
        //                 t.get_name() == data_writer.get_topic_name()
        //                     && t.get_type_name() == data_writer.get_type_name()
        //             })
        //             .cloned()
        //             .expect("Topic must exist")
        //     });

        // Ok(UserDefinedTopicNode::new(ChildNode::new(
        //     topic.downgrade(),
        //     *self.0.parent(),
        // )))
        todo!()
    }

    pub fn get_publisher(&self) -> UserDefinedPublisherNode {
        UserDefinedPublisherNode::new(self.parent_publisher, self.parent_participant)
    }

    pub fn assert_liveliness(&self) -> DdsResult<()> {
        // if !self.0.get()?.is_enabled() {
        //     return Err(DdsError::NotEnabled);
        // }

        todo!()
    }

    pub fn get_matched_subscription_data(
        &self,
        domain_participant: &DdsDomainParticipant,
        subscription_handle: InstanceHandle,
    ) -> DdsResult<SubscriptionBuiltinTopicData> {
        let is_parent_enabled = domain_participant
            .get_publisher(self.parent_publisher)
            .ok_or(DdsError::AlreadyDeleted)?
            .is_enabled();
        if !is_parent_enabled {
            return Err(DdsError::PreconditionNotMet(
                "Parent publisher disabled".to_string(),
            ));
        }

        domain_participant
            .get_publisher(self.parent_publisher)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_data_writer(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_matched_subscription_data(subscription_handle)
            .ok_or(DdsError::BadParameter)
    }

    pub fn get_matched_subscriptions(
        &self,
        domain_participant: &DdsDomainParticipant,
    ) -> DdsResult<Vec<InstanceHandle>> {
        let is_parent_enabled = domain_participant
            .get_publisher(self.parent_publisher)
            .ok_or(DdsError::AlreadyDeleted)?
            .is_enabled();
        if !is_parent_enabled {
            return Err(DdsError::PreconditionNotMet(
                "Parent publisher disabled".to_string(),
            ));
        }

        Ok(domain_participant
            .get_publisher(self.parent_publisher)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_data_writer(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_matched_subscriptions())
    }

    pub fn set_qos(
        &self,
        domain_participant: &DdsDomainParticipant,
        qos: QosKind<DataWriterQos>,
    ) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => Default::default(),
            QosKind::Specific(q) => q,
        };
        qos.is_consistent()?;

        let is_data_writer_enabled = domain_participant
            .get_publisher(self.parent_publisher)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_data_writer(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .is_enabled();

        if is_data_writer_enabled {
            domain_participant
                .get_publisher(self.parent_publisher)
                .ok_or(DdsError::AlreadyDeleted)?
                .get_data_writer(self.this)
                .ok_or(DdsError::AlreadyDeleted)?
                .get_qos()
                .check_immutability(&qos)?;
        }

        domain_participant
            .get_publisher(self.parent_publisher)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_data_writer(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .set_qos(qos);

        if is_data_writer_enabled {
            let type_name = domain_participant
                .get_publisher(self.parent_publisher)
                .ok_or(DdsError::AlreadyDeleted)?
                .get_data_writer(self.this)
                .ok_or(DdsError::AlreadyDeleted)?
                .get_type_name();

            let topic_name = domain_participant
                .get_publisher(self.parent_publisher)
                .ok_or(DdsError::AlreadyDeleted)?
                .get_data_writer(self.this)
                .ok_or(DdsError::AlreadyDeleted)?
                .get_topic_name();

            let topic = domain_participant
                .get_topic(topic_name, type_name)
                .expect("Topic must exist");
            let publisher_qos = domain_participant
                .get_publisher(self.parent_publisher)
                .ok_or(DdsError::AlreadyDeleted)?
                .get_qos();
            let discovered_writer_data = domain_participant
                .get_publisher(self.parent_publisher)
                .ok_or(DdsError::AlreadyDeleted)?
                .get_data_writer(self.this)
                .ok_or(DdsError::AlreadyDeleted)?
                .as_discovered_writer_data(&topic.get_qos(), &publisher_qos);
            announce_created_data_writer(domain_participant, discovered_writer_data);
        }

        Ok(())
    }

    pub fn get_qos(&self, domain_participant: &DdsDomainParticipant) -> DdsResult<DataWriterQos> {
        Ok(domain_participant
            .get_publisher(self.parent_publisher)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_data_writer(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_qos())
    }

    pub fn enable(&self, domain_participant: &mut DdsDomainParticipant) -> DdsResult<()> {
        enable_data_writer(domain_participant, self.parent_publisher, self.this)
    }

    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        Ok(self.this.into())
    }
}

fn enable_data_writer(
    domain_participant: &mut DdsDomainParticipant,
    publisher_guid: Guid,
    data_writer_guid: Guid,
) -> DdsResult<()> {
    let is_parent_enabled = domain_participant
        .get_publisher(publisher_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .is_enabled();
    if !is_parent_enabled {
        return Err(DdsError::PreconditionNotMet(
            "Parent publisher disabled".to_string(),
        ));
    }
    domain_participant
        .get_publisher(publisher_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_data_writer(data_writer_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .enable();

    let type_name = domain_participant
        .get_publisher(publisher_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_data_writer(data_writer_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_type_name();

    let topic_name = domain_participant
        .get_publisher(publisher_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_data_writer(data_writer_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_topic_name();

    let topic = domain_participant
        .get_topic(topic_name, type_name)
        .expect("Topic must exist");
    let publisher_qos = domain_participant
        .get_publisher(publisher_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_qos();
    let discovered_writer_data = domain_participant
        .get_publisher(publisher_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_data_writer(data_writer_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .as_discovered_writer_data(&topic.get_qos(), &publisher_qos);
    announce_created_data_writer(domain_participant, discovered_writer_data);

    Ok(())
}

fn announce_created_data_writer(
    domain_participant: &DdsDomainParticipant,
    discovered_writer_data: DiscoveredWriterData,
) {
    let writer_data = &DiscoveredWriterData::new(
        discovered_writer_data.dds_publication_data().clone(),
        WriterProxy::new(
            discovered_writer_data.writer_proxy().remote_writer_guid(),
            discovered_writer_data
                .writer_proxy()
                .remote_group_entity_id(),
            domain_participant.default_unicast_locator_list().to_vec(),
            domain_participant.default_multicast_locator_list().to_vec(),
            discovered_writer_data
                .writer_proxy()
                .data_max_size_serialized(),
        ),
    );

    let mut serialized_data = Vec::new();
    writer_data
        .dds_serialize(&mut serialized_data)
        .expect("Failed to serialize data");

    let timestamp = domain_participant.get_current_time();

    domain_participant
        .get_builtin_publisher()
        .stateful_data_writer_list()
        .iter()
        .find(|x| x.get_type_name() == DiscoveredWriterData::type_name())
        .unwrap()
        .write_w_timestamp(
            serialized_data,
            writer_data.get_serialized_key(),
            None,
            timestamp,
        )
        .expect("Should not fail to write built-in message");
}

impl AnyDataWriter for UserDefinedDataWriterNode {}
