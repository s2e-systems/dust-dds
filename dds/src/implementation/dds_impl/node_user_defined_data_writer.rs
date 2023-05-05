use crate::{
    builtin_topics::SubscriptionBuiltinTopicData,
    implementation::{
        data_representation_builtin_endpoints::discovered_writer_data::{
            DiscoveredWriterData, WriterProxy,
        },
        rtps::types::Guid,
        utils::shared_object::{DdsRwLock, DdsShared},
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, QosKind},
        status::{
            LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
            PublicationMatchedStatus, StatusKind,
        },
        time::{Duration, Time},
    },
    topic_definition::type_support::{DdsSerialize, DdsSerializedKey, DdsType},
};

use super::{
    any_data_writer_listener::AnyDataWriterListener, dds_domain_participant::DdsDomainParticipant,
    node_user_defined_publisher::UserDefinedPublisherNode,
    node_user_defined_topic::UserDefinedTopicNode, status_condition_impl::StatusConditionImpl,
};

#[derive(PartialEq, Debug)]
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

    pub fn register_instance_w_timestamp(
        &self,
        instance_serialized_key: DdsSerializedKey,
        timestamp: Time,
    ) -> DdsResult<Option<InstanceHandle>> {
        // self.0
        //     .get()?
        //     .register_instance_w_timestamp(instance_serialized_key, timestamp)
        todo!()
    }

    pub fn unregister_instance_w_timestamp(
        &self,
        instance_serialized_key: Vec<u8>,
        handle: InstanceHandle,
        timestamp: Time,
    ) -> DdsResult<()> {
        // self.0
        // .get()?
        // .unregister_instance_w_timestamp(instance_serialized_key, handle, timestamp)
        todo!()
    }

    pub fn get_key_value<Foo>(&self, key_holder: &mut Foo, handle: InstanceHandle) -> DdsResult<()>
    where
        Foo: DdsType,
    {
        // self.0.get()?.get_key_value(key_holder, handle)
        todo!()
    }

    pub fn lookup_instance(
        &self,
        instance_serialized_key: DdsSerializedKey,
    ) -> DdsResult<Option<InstanceHandle>> {
        // self.0.get()?.lookup_instance(instance_serialized_key)
        todo!()
    }

    pub fn write_w_timestamp(
        &self,
        serialized_data: Vec<u8>,
        instance_serialized_key: DdsSerializedKey,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()> {
        todo!()
        // if !self.0.get()?.is_enabled() {
        //     return Err(DdsError::NotEnabled);
        // }

        // self.0.get()?.write_w_timestamp(
        //     serialized_data,
        //     instance_serialized_key,
        //     handle,
        //     timestamp,
        // )?;

        // THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_dcps_service(&self.0.parent().prefix(), |dcps| {
        //     dcps.unwrap().user_defined_data_send_condvar().notify_all()
        // });

        // Ok(())
    }

    pub fn dispose_w_timestamp(
        &self,
        instance_serialized_key: Vec<u8>,
        handle: InstanceHandle,
        timestamp: Time,
    ) -> DdsResult<()> {
        // self.0
        //     .get()?
        //     .dispose_w_timestamp(instance_serialized_key, handle, timestamp)
        todo!()
    }

    pub fn wait_for_acknowledgments(&self, max_wait: Duration) -> DdsResult<()> {
        // self.0.get()?.wait_for_acknowledgments(max_wait)
        todo!()
    }

    pub fn get_liveliness_lost_status(&self) -> DdsResult<LivelinessLostStatus> {
        todo!()
    }

    pub fn get_offered_deadline_missed_status(&self) -> DdsResult<OfferedDeadlineMissedStatus> {
        todo!()
    }

    pub fn get_offered_incompatible_qos_status(&self) -> DdsResult<OfferedIncompatibleQosStatus> {
        // Ok(self.0.get()?.get_offered_incompatible_qos_status())
        todo!()
    }

    pub fn get_publication_matched_status(&self) -> DdsResult<PublicationMatchedStatus> {
        // self.0
        //     .get()?
        //     .remove_communication_state(StatusKind::PublicationMatched);
        // Ok(self.0.get()?.get_publication_matched_status())
        todo!()
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
        // UserDefinedPublisherNode::new(self.parent_publisher, self.parent_participant)
        todo!()
    }

    pub fn assert_liveliness(&self) -> DdsResult<()> {
        // if !self.0.get()?.is_enabled() {
        //     return Err(DdsError::NotEnabled);
        // }

        todo!()
    }

    pub fn get_matched_subscription_data(
        &self,
        subscription_handle: InstanceHandle,
    ) -> DdsResult<SubscriptionBuiltinTopicData> {
        // if !self.0.get()?.is_enabled() {
        //     return Err(DdsError::NotEnabled);
        // }

        // self.0
        //     .get()?
        //     .get_matched_subscription_data(subscription_handle)
        //     .ok_or(DdsError::BadParameter)
        todo!()
    }

    pub fn get_matched_subscriptions(&self) -> DdsResult<Vec<InstanceHandle>> {
        // if !self.0.get()?.is_enabled() {
        //     return Err(DdsError::NotEnabled);
        // }

        // Ok(self.0.get()?.get_matched_subscriptions())
        todo!()
    }

    pub fn set_qos(&self, qos: QosKind<DataWriterQos>) -> DdsResult<()> {
        // let data_writer = self.0.get()?;

        // let qos = match qos {
        //     QosKind::Default => Default::default(),
        //     QosKind::Specific(q) => q,
        // };
        // qos.is_consistent()?;

        // if self.0.get()?.is_enabled() {
        //     if self.0.get()?.is_enabled() {
        //         self.0.get()?.get_qos().check_immutability(&qos)?;
        //     }

        //     self.0.get()?.set_qos(qos);

        //     let topic = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_participant(
        //         &self.0.parent().prefix(),
        //         |dp| {
        //             dp.unwrap()
        //                 .topic_list()
        //                 .into_iter()
        //                 .find(|t| {
        //                     t.get_name() == data_writer.get_topic_name()
        //                         && t.get_type_name() == data_writer.get_type_name()
        //                 })
        //                 .cloned()
        //                 .expect("Topic must exist")
        //         },
        //     );
        //     todo!()
        //     // let discovered_writer_data = self
        //     //     .0
        //     //     .get()?
        //     //     .as_discovered_writer_data(&topic.get_qos(), &self.0.parent().get()?.get_qos());
        //     // THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_dcps_service(self.0.parent(), |dcps| {
        //     //     dcps.unwrap()
        //     //         .announce_sender()
        //     //         .send(AnnounceKind::CreatedDataWriter(discovered_writer_data))
        //     //         .ok()
        //     // });
        // } else {
        //     self.0.get()?.set_qos(qos);
        // }
        // Ok(())
        todo!()
    }

    pub fn get_qos(&self) -> DdsResult<DataWriterQos> {
        // Ok(self.0.get()?.get_qos())
        todo!()
    }

    pub fn set_listener(
        &self,
        a_listener: Option<Box<dyn AnyDataWriterListener + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        // self.0.get()?.set_listener(a_listener, mask);
        // Ok(())
        todo!()
    }

    pub fn get_statuscondition(
        &self,
        domain_participant: &mut DdsDomainParticipant,
    ) -> DdsResult<DdsShared<DdsRwLock<StatusConditionImpl>>> {
        get_data_writer_status_condition(domain_participant, self.parent_publisher, self.this)
    }

    pub fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        // Ok(self.0.get()?.get_status_changes())
        todo!()
    }

    pub fn enable(&self, domain_participant: &mut DdsDomainParticipant) -> DdsResult<()> {
        enable_data_writer(domain_participant, self.parent_publisher, self.this)
    }

    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        // Ok(self.0.get()?.guid().into())
        todo!()
    }

    pub fn this(&self) -> Guid {
        self.this
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
        .cloned()
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
        .into_iter()
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

fn get_data_writer_status_condition(
    domain_participant: &DdsDomainParticipant,
    publisher_guid: Guid,
    data_writer_guid: Guid,
) -> DdsResult<DdsShared<DdsRwLock<StatusConditionImpl>>> {
    Ok(domain_participant
        .get_publisher(publisher_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_data_writer(data_writer_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_statuscondition())
}
