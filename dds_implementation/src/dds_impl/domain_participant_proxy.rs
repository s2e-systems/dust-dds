use std::sync::atomic::{AtomicU8, Ordering};

use dds_api::{
    builtin_topics::{ParticipantBuiltinTopicData, TopicBuiltinTopicData},
    dcps_psm::{BuiltInTopicKey, DomainId, Duration, InstanceHandle, StatusMask, Time},
    domain::{
        domain_participant::{DomainParticipant, DomainParticipantTopicFactory},
        domain_participant_listener::DomainParticipantListener,
    },
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::{DomainParticipantQos, PublisherQos, SubscriberQos, TopicQos},
    },
    publication::{
        data_writer::DataWriter,
        publisher::{Publisher, PublisherDataWriterFactory},
    },
    return_type::{DdsError, DdsResult},
    subscription::subscriber::Subscriber,
};
use rtps_pim::{
    messages::types::Count,
    structure::{
        entity::RtpsEntityAttributes,
        group::RtpsGroupConstructor,
        participant::RtpsParticipantConstructor,
        types::{
            EntityId, Guid, GuidPrefix, Locator, ENTITYID_PARTICIPANT, PROTOCOLVERSION,
            USER_DEFINED_READER_GROUP, USER_DEFINED_WRITER_GROUP, VENDOR_ID_S2E,
        },
    },
};

use crate::{
    data_representation_builtin_endpoints::discovered_topic_data::{
        DiscoveredTopicData, DCPS_TOPIC,
    },
    dds_type::{DdsSerialize, DdsType},
    utils::{
        rtps_structure::RtpsStructure,
        shared_object::{DdsRwLock, DdsShared, DdsWeak},
    },
};

use super::{
    publisher_proxy::{PublisherAttributes, PublisherProxy},
    subscriber_proxy::{SubscriberAttributes, SubscriberProxy},
    topic_proxy::{TopicAttributes, TopicProxy},
};

pub struct DomainParticipantAttributes<Rtps>
where
    Rtps: RtpsStructure,
{
    pub rtps_participant: Rtps::Participant,
    pub domain_id: DomainId,
    pub domain_tag: String,
    pub qos: DomainParticipantQos,
    pub builtin_subscriber: DdsRwLock<Option<DdsShared<SubscriberAttributes<Rtps>>>>,
    pub builtin_publisher: DdsRwLock<Option<DdsShared<PublisherAttributes<Rtps>>>>,
    pub user_defined_subscriber_list: DdsRwLock<Vec<DdsShared<SubscriberAttributes<Rtps>>>>,
    pub user_defined_subscriber_counter: AtomicU8,
    pub default_subscriber_qos: SubscriberQos,
    pub user_defined_publisher_list: DdsRwLock<Vec<DdsShared<PublisherAttributes<Rtps>>>>,
    pub user_defined_publisher_counter: AtomicU8,
    pub default_publisher_qos: PublisherQos,
    pub topic_list: DdsRwLock<Vec<DdsShared<TopicAttributes<Rtps>>>>,
    pub default_topic_qos: TopicQos,
    pub manual_liveliness_count: Count,
    pub lease_duration: rtps_pim::behavior::types::Duration,
    pub metatraffic_unicast_locator_list: Vec<Locator>,
    pub metatraffic_multicast_locator_list: Vec<Locator>,
    pub enabled: DdsRwLock<bool>,
}

impl<Rtps> DomainParticipantAttributes<Rtps>
where
    Rtps: RtpsStructure,
{
    pub fn new(
        guid_prefix: GuidPrefix,
        domain_id: DomainId,
        domain_tag: String,
        domain_participant_qos: DomainParticipantQos,
        metatraffic_unicast_locator_list: Vec<Locator>,
        metatraffic_multicast_locator_list: Vec<Locator>,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
    ) -> Self {
        let lease_duration = rtps_pim::behavior::types::Duration::new(100, 0);
        let protocol_version = PROTOCOLVERSION;
        let vendor_id = VENDOR_ID_S2E;
        let rtps_participant = Rtps::Participant::new(
            Guid::new(guid_prefix, ENTITYID_PARTICIPANT),
            &default_unicast_locator_list,
            &default_multicast_locator_list,
            protocol_version,
            vendor_id,
        );

        Self {
            rtps_participant,
            domain_id,
            domain_tag,
            qos: domain_participant_qos,
            builtin_subscriber: DdsRwLock::new(None),
            builtin_publisher: DdsRwLock::new(None),
            user_defined_subscriber_list: DdsRwLock::new(Vec::new()),
            user_defined_subscriber_counter: AtomicU8::new(0),
            default_subscriber_qos: SubscriberQos::default(),
            user_defined_publisher_list: DdsRwLock::new(Vec::new()),
            user_defined_publisher_counter: AtomicU8::new(0),
            default_publisher_qos: PublisherQos::default(),
            topic_list: DdsRwLock::new(Vec::new()),
            default_topic_qos: TopicQos::default(),
            manual_liveliness_count: Count(0),
            lease_duration,
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
            enabled: DdsRwLock::new(false),
        }
    }
}

pub struct DomainParticipantProxy<Rtps>
where
    Rtps: RtpsStructure,
{
    domain_participant: DdsWeak<DomainParticipantAttributes<Rtps>>,
}

impl<Rtps> Clone for DomainParticipantProxy<Rtps>
where
    Rtps: RtpsStructure,
{
    fn clone(&self) -> Self {
        Self {
            domain_participant: self.domain_participant.clone(),
        }
    }
}

impl<Rtps> DomainParticipantProxy<Rtps>
where
    Rtps: RtpsStructure,
{
    pub fn new(domain_participant: DdsWeak<DomainParticipantAttributes<Rtps>>) -> Self {
        Self { domain_participant }
    }
}

impl<Foo, Rtps> DomainParticipantTopicFactory<Foo> for DomainParticipantProxy<Rtps>
where
    Foo: DdsType + DdsSerialize + Send + Sync + 'static,
    Rtps: RtpsStructure,
{
    type TopicType = TopicProxy<Foo, Rtps>;

    fn topic_factory_create_topic(
        &self,
        topic_name: &str,
        qos: Option<TopicQos>,
        _a_listener: Option<<Self::TopicType as Entity>::Listener>,
        _mask: StatusMask,
    ) -> DdsResult<Self::TopicType> {
        let participant_shared = self.domain_participant.upgrade()?;

        let qos = qos.unwrap_or(participant_shared.default_topic_qos.clone());

        // /////// Create topic
        let topic_shared = DdsShared::new(TopicAttributes::new(
            qos.clone(),
            Foo::type_name(),
            topic_name,
            participant_shared.downgrade(),
        ));

        participant_shared
            .topic_list
            .write_lock()
            .push(topic_shared.clone());

        // /////// Announce the topic creation
        {
            let domain_participant_proxy =
                DomainParticipantProxy::new(participant_shared.downgrade());
            let builtin_publisher_option = participant_shared.builtin_publisher.read_lock().clone();
            if let Some(builtin_publisher) = builtin_publisher_option {
                let builtin_publisher_proxy = PublisherProxy::new(builtin_publisher.downgrade());

                if let Ok(topic_creation_topic) =
                    domain_participant_proxy.topic_factory_lookup_topicdescription(DCPS_TOPIC)
                {
                    if let Ok(sedp_builtin_topic_announcer) = builtin_publisher_proxy
                        .datawriter_factory_lookup_datawriter(&topic_creation_topic)
                    {
                        let sedp_discovered_topic_data = DiscoveredTopicData {
                            topic_builtin_topic_data: TopicBuiltinTopicData {
                                key: BuiltInTopicKey { value: [1; 16] },
                                name: topic_name.to_string(),
                                type_name: Foo::type_name().to_string(),
                                durability: qos.durability.clone(),
                                durability_service: qos.durability_service.clone(),
                                deadline: qos.deadline.clone(),
                                latency_budget: qos.latency_budget.clone(),
                                liveliness: qos.liveliness.clone(),
                                reliability: qos.reliability.clone(),
                                transport_priority: qos.transport_priority.clone(),
                                lifespan: qos.lifespan.clone(),
                                destination_order: qos.destination_order.clone(),
                                history: qos.history.clone(),
                                resource_limits: qos.resource_limits.clone(),
                                ownership: qos.ownership.clone(),
                                topic_data: qos.topic_data.clone(),
                            },
                        };

                        sedp_builtin_topic_announcer
                            .write(&sedp_discovered_topic_data, None)
                            .unwrap();
                    }
                }
            }
        }

        Ok(TopicProxy::new(topic_shared.downgrade()))
    }

    fn topic_factory_delete_topic(&self, topic: &Self::TopicType) -> DdsResult<()> {
        let domain_participant_shared = self.domain_participant.upgrade()?;
        let topic_shared = topic.as_ref().upgrade()?;

        let topic_list = &mut domain_participant_shared.topic_list.write_lock();
        let topic_list_position = topic_list
            .iter()
            .position(|topic| topic == &topic_shared)
            .ok_or(DdsError::PreconditionNotMet(
                "Topic can only be deleted from its parent publisher".to_string(),
            ))?;
        topic_list.remove(topic_list_position);

        Ok(())
    }

    fn topic_factory_find_topic(
        &self,
        topic_name: &str,
        _timeout: Duration,
    ) -> DdsResult<Self::TopicType> {
        self.domain_participant
            .upgrade()?
            .topic_list
            .read_lock()
            .iter()
            .find_map(|topic| {
                if topic.topic_name == topic_name && topic.type_name == Foo::type_name() {
                    Some(TopicProxy::new(topic.downgrade()))
                } else {
                    None
                }
            })
            .ok_or(DdsError::PreconditionNotMet("Not found".to_string()))
    }

    fn topic_factory_lookup_topicdescription(
        &self,
        topic_name: &str,
    ) -> DdsResult<Self::TopicType> {
        self.domain_participant
            .upgrade()?
            .topic_list
            .read_lock()
            .iter()
            .find_map(|topic| {
                if topic.topic_name == topic_name && topic.type_name == Foo::type_name() {
                    Some(TopicProxy::new(topic.downgrade()))
                } else {
                    None
                }
            })
            .ok_or(DdsError::PreconditionNotMet("Not found".to_string()))
    }
}

impl<Rtps> DomainParticipant for DomainParticipantProxy<Rtps>
where
    Rtps: RtpsStructure,
{
    type PublisherType = PublisherProxy<Rtps>;
    type SubscriberType = SubscriberProxy<Rtps>;

    fn create_publisher(
        &self,
        qos: Option<PublisherQos>,
        _a_listener: Option<<Self::PublisherType as Entity>::Listener>,
        _mask: StatusMask,
    ) -> DdsResult<Self::PublisherType> {
        let domain_participant_attributes = self.domain_participant.upgrade()?;
        let publisher_qos =
            qos.unwrap_or(domain_participant_attributes.default_publisher_qos.clone());
        let publisher_counter = domain_participant_attributes
            .user_defined_publisher_counter
            .fetch_add(1, Ordering::Relaxed);
        let entity_id = EntityId::new([publisher_counter, 0, 0], USER_DEFINED_WRITER_GROUP);
        let guid = Guid::new(
            domain_participant_attributes
                .rtps_participant
                .guid()
                .prefix(),
            entity_id,
        );
        let rtps_group = Rtps::Group::new(guid);
        // let sedp_builtin_publications_topic =
        // rtps_shared_new(TopicAttributes::new(TopicQos::default(), "", ""));
        // let sedp_builtin_publications_announcer =
        //     rtps_shared_read_lock(&domain_participant_attributes_lock.builtin_publisher)
        //         .lookup_datawriter::<DiscoveredWriterData>(&sedp_builtin_publications_topic);
        let publisher_impl =
            PublisherAttributes::new(publisher_qos, rtps_group, self.domain_participant.clone());
        let publisher_impl_shared = DdsShared::new(publisher_impl);
        domain_participant_attributes
            .user_defined_publisher_list
            .write_lock()
            .push(publisher_impl_shared.clone());

        let publisher_weak = publisher_impl_shared.downgrade();

        Ok(PublisherProxy::new(publisher_weak))
    }

    fn delete_publisher(&self, a_publisher: &Self::PublisherType) -> DdsResult<()> {
        let domain_participant_attributes = self.domain_participant.upgrade()?;
        let publisher_shared = a_publisher.0.upgrade()?;
        if std::ptr::eq(&a_publisher.get_participant()?, self) {
            // rtps_shared_read_lock(&domain_participant_lock).delete_publisher(&publisher_shared)
            domain_participant_attributes
                .user_defined_publisher_list
                .write_lock()
                .retain(|x| *x != publisher_shared);
            Ok(())
        } else {
            Err(DdsError::PreconditionNotMet(
                "Publisher can only be deleted from its parent participant".to_string(),
            ))
        }
    }

    fn create_subscriber(
        &self,
        qos: Option<SubscriberQos>,
        _a_listener: Option<<Self::SubscriberType as Entity>::Listener>,
        _mask: StatusMask,
    ) -> DdsResult<Self::SubscriberType> {
        let domain_participant_attributes = self.domain_participant.upgrade()?;
        let subscriber_qos =
            qos.unwrap_or(domain_participant_attributes.default_subscriber_qos.clone());
        let subcriber_counter = domain_participant_attributes
            .user_defined_subscriber_counter
            .fetch_add(1, Ordering::Relaxed);
        let entity_id = EntityId::new([subcriber_counter, 0, 0], USER_DEFINED_READER_GROUP);
        let guid = Guid::new(
            domain_participant_attributes
                .rtps_participant
                .guid()
                .prefix(),
            entity_id,
        );
        let rtps_group = Rtps::Group::new(guid);
        let subscriber =
            SubscriberAttributes::new(subscriber_qos, rtps_group, self.domain_participant.clone());
        let subscriber_shared = DdsShared::new(subscriber);
        domain_participant_attributes
            .user_defined_subscriber_list
            .write_lock()
            .push(subscriber_shared.clone());

        let subscriber_weak = subscriber_shared.downgrade();
        Ok(SubscriberProxy::new(self.clone(), subscriber_weak))
    }

    fn delete_subscriber(&self, a_subscriber: &Self::SubscriberType) -> DdsResult<()> {
        let domain_participant_attributes = self.domain_participant.upgrade()?;
        let subscriber_shared = a_subscriber.as_ref().upgrade()?;
        if std::ptr::eq(&a_subscriber.get_participant()?, self) {
            domain_participant_attributes
                .user_defined_subscriber_list
                .write_lock()
                .retain(|x| *x != subscriber_shared);
            Ok(())
        } else {
            Err(DdsError::PreconditionNotMet(
                "Subscriber can only be deleted from its parent participant".to_string(),
            ))
        }
    }

    fn get_builtin_subscriber(&self) -> DdsResult<Self::SubscriberType> {
        let domain_participant_shared = self.domain_participant.upgrade()?;
        let subscriber = domain_participant_shared
            .builtin_subscriber
            .read_lock()
            .as_ref()
            .unwrap()
            .clone();
        Ok(SubscriberProxy::new(self.clone(), subscriber.downgrade()))
    }

    fn ignore_participant(&self, _handle: InstanceHandle) -> DdsResult<()> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.ignore_participant(handle)
        todo!()
    }

    fn ignore_topic(&self, _handle: InstanceHandle) -> DdsResult<()> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.ignore_topic(handle)
        todo!()
    }

    fn ignore_publication(&self, _handle: InstanceHandle) -> DdsResult<()> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.ignore_publication(handle)
        todo!()
    }

    fn ignore_subscription(&self, _handle: InstanceHandle) -> DdsResult<()> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.ignore_subscription(handle)
        todo!()
    }

    fn get_domain_id(&self) -> DdsResult<DomainId> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.get_domain_id()
        todo!()
    }

    fn delete_contained_entities(&self) -> DdsResult<()> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.delete_contained_entities()
        todo!()
    }

    fn assert_liveliness(&self) -> DdsResult<()> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.assert_liveliness()
        todo!()
    }

    fn set_default_publisher_qos(&self, _qos: Option<PublisherQos>) -> DdsResult<()> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let mut domain_participant_lock = rtps_shared_write_lock(&domain_participant_shared);
        // domain_participant_lock.set_default_publisher_qos(qos)
        todo!()
    }

    fn get_default_publisher_qos(&self) -> DdsResult<PublisherQos> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.get_default_publisher_qos()
        todo!()
    }

    fn set_default_subscriber_qos(&self, _qos: Option<SubscriberQos>) -> DdsResult<()> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let mut domain_participant_lock = rtps_shared_write_lock(&domain_participant_shared);
        // domain_participant_lock.set_default_subscriber_qos(qos)
        todo!()
    }

    fn get_default_subscriber_qos(&self) -> DdsResult<SubscriberQos> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.get_default_subscriber_qos()
        todo!()
    }

    fn set_default_topic_qos(&self, _qos: Option<TopicQos>) -> DdsResult<()> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let mut domain_participant_lock = rtps_shared_write_lock(&domain_participant_shared);
        // domain_participant_lock.set_default_topic_qos(qos)
        todo!()
    }

    fn get_default_topic_qos(&self) -> DdsResult<TopicQos> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.get_default_topic_qos()
        todo!()
    }

    fn get_discovered_participants(
        &self,
        _participant_handles: &mut [InstanceHandle],
    ) -> DdsResult<()> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.get_discovered_participants(participant_handles)
        todo!()
    }

    fn get_discovered_participant_data(
        &self,
        _participant_data: ParticipantBuiltinTopicData,
        _participant_handle: InstanceHandle,
    ) -> DdsResult<()> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock
        //     .get_discovered_participant_data(participant_data, participant_handle)
        todo!()
    }

    fn get_discovered_topics(&self, _topic_handles: &mut [InstanceHandle]) -> DdsResult<()> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.get_discovered_topics(topic_handles)
        todo!()
    }

    fn get_discovered_topic_data(
        &self,
        _topic_data: TopicBuiltinTopicData,
        _topic_handle: InstanceHandle,
    ) -> DdsResult<()> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.get_discovered_topic_data(topic_data, topic_handle)
        todo!()
    }

    fn contains_entity(&self, _a_handle: InstanceHandle) -> DdsResult<bool> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.contains_entity(a_handle)
        todo!()
    }

    fn get_current_time(&self) -> DdsResult<Time> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.get_current_time()
        todo!()
    }
}

impl<Rtps> Entity for DomainParticipantProxy<Rtps>
where
    Rtps: RtpsStructure,
{
    type Qos = DomainParticipantQos;
    type Listener = &'static dyn DomainParticipantListener;

    fn set_qos(&self, _qos: Option<Self::Qos>) -> DdsResult<()> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let mut domain_participant_lock = rtps_shared_write_lock(&domain_participant_shared);
        // domain_participant_lock.set_qos(qos)
        todo!()
    }

    fn get_qos(&self) -> DdsResult<Self::Qos> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.get_qos()
        todo!()
    }

    fn set_listener(
        &self,
        _a_listener: Option<Self::Listener>,
        _mask: StatusMask,
    ) -> DdsResult<()> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.set_listener(a_listener, mask)
        todo!()
    }

    fn get_listener(&self) -> DdsResult<Option<Self::Listener>> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.get_listener()
        todo!()
    }

    fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.get_statuscondition()
        todo!()
    }

    fn get_status_changes(&self) -> DdsResult<StatusMask> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.get_status_changes()
        todo!()
    }

    fn enable(&self) -> DdsResult<()> {
        let domain_participant_shared = self.domain_participant.upgrade()?;
        *domain_participant_shared.enabled.write_lock() = true;
        Ok(())
    }

    fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.get_instance_handle()
        todo!()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{dds_type::Endianness, test_utils::mock_rtps::MockRtps};
    use std::io::Write;

    macro_rules! make_empty_dds_type {
        ($type_name:ident) => {
            struct $type_name {}

            impl DdsSerialize for $type_name {
                fn serialize<W: Write, E: Endianness>(&self, _writer: W) -> DdsResult<()> {
                    Ok(())
                }
            }

            impl DdsType for $type_name {
                fn type_name() -> &'static str {
                    stringify!($type_name)
                }

                fn has_key() -> bool {
                    false
                }
            }
        };
    }

    make_empty_dds_type!(Foo);

    #[test]
    fn topic_factory_create_topic() {
        type Topic = TopicProxy<Foo, MockRtps>;

        let domain_participant = DdsShared::new(DomainParticipantAttributes::new(
            GuidPrefix([1; 12]),
            DomainId::default(),
            "".to_string(),
            DomainParticipantQos::default(),
            vec![],
            vec![],
            vec![],
            vec![],
        ));
        let domain_participant_proxy = DomainParticipantProxy::new(domain_participant.downgrade());

        let len_before = domain_participant.topic_list.read_lock().len();

        let topic = domain_participant_proxy.topic_factory_create_topic("topic", None, None, 0)
            as DdsResult<Topic>;

        assert!(topic.is_ok());
        assert_eq!(
            len_before + 1,
            domain_participant.topic_list.read_lock().len()
        );
    }

    #[test]
    fn topic_factory_delete_topic() {
        type Topic = TopicProxy<Foo, MockRtps>;

        let domain_participant = DdsShared::new(DomainParticipantAttributes::new(
            GuidPrefix([1; 12]),
            DomainId::default(),
            "".to_string(),
            DomainParticipantQos::default(),
            vec![],
            vec![],
            vec![],
            vec![],
        ));
        let domain_participant_proxy = DomainParticipantProxy::new(domain_participant.downgrade());

        let len_before = domain_participant.topic_list.read_lock().len();

        let topic = domain_participant_proxy
            .topic_factory_create_topic("topic", None, None, 0)
            .unwrap() as Topic;

        assert_eq!(
            len_before + 1,
            domain_participant.topic_list.read_lock().len()
        );

        domain_participant_proxy
            .topic_factory_delete_topic(&topic)
            .unwrap();

        assert_eq!(len_before, domain_participant.topic_list.read_lock().len());
        assert!(topic.as_ref().upgrade().is_err());
    }

    #[test]
    fn topic_factory_delete_topic_from_other_participant() {
        type Topic = TopicProxy<Foo, MockRtps>;

        let domain_participant = DdsShared::new(DomainParticipantAttributes::new(
            GuidPrefix([1; 12]),
            DomainId::default(),
            "".to_string(),
            DomainParticipantQos::default(),
            vec![],
            vec![],
            vec![],
            vec![],
        ));
        let domain_participant_proxy = DomainParticipantProxy::new(domain_participant.downgrade());

        let domain_participant2 = DdsShared::new(DomainParticipantAttributes::new(
            GuidPrefix([1; 12]),
            DomainId::default(),
            "".to_string(),
            DomainParticipantQos::default(),
            vec![],
            vec![],
            vec![],
            vec![],
        ));
        let domain_participant2_proxy =
            DomainParticipantProxy::new(domain_participant2.downgrade());

        let len_before = domain_participant.topic_list.read_lock().len();
        let len_before2 = domain_participant2.topic_list.read_lock().len();

        let topic = domain_participant_proxy
            .topic_factory_create_topic("topic", None, None, 0)
            .unwrap() as Topic;

        assert_eq!(
            len_before + 1,
            domain_participant.topic_list.read_lock().len()
        );
        assert_eq!(
            len_before2,
            domain_participant2.topic_list.read_lock().len()
        );

        assert!(matches!(
            domain_participant2_proxy.topic_factory_delete_topic(&topic),
            Err(DdsError::PreconditionNotMet(_))
        ));
        assert!(topic.as_ref().upgrade().is_ok());
    }

    #[test]
    fn topic_factory_lookup_topic_with_no_topic() {
        type Topic = TopicProxy<Foo, MockRtps>;

        let domain_participant = DdsShared::new(DomainParticipantAttributes::new(
            GuidPrefix([1; 12]),
            DomainId::default(),
            "".to_string(),
            DomainParticipantQos::default(),
            vec![],
            vec![],
            vec![],
            vec![],
        ));
        let domain_participant_proxy = DomainParticipantProxy::new(domain_participant.downgrade());

        assert!(
            (domain_participant_proxy.topic_factory_lookup_topicdescription("topic")
                as DdsResult<Topic>)
                .is_err()
        );
    }

    #[test]
    fn topic_factory_lookup_topic_with_one_topic() {
        type Topic = TopicProxy<Foo, MockRtps>;

        let domain_participant = DdsShared::new(DomainParticipantAttributes::new(
            GuidPrefix([1; 12]),
            DomainId::default(),
            "".to_string(),
            DomainParticipantQos::default(),
            vec![],
            vec![],
            vec![],
            vec![],
        ));
        let domain_participant_proxy = DomainParticipantProxy::new(domain_participant.downgrade());

        let topic = domain_participant_proxy
            .topic_factory_create_topic("topic", None, None, 0)
            .unwrap() as Topic;

        assert!(
            (domain_participant_proxy.topic_factory_lookup_topicdescription("topic")
                as DdsResult<Topic>)
                .unwrap()
                .as_ref()
                .upgrade()
                .unwrap()
                == topic.as_ref().upgrade().unwrap()
        );
    }

    make_empty_dds_type!(Bar);

    #[test]
    fn topic_factory_lookup_topic_with_one_topic_with_wrong_type() {
        type TopicFoo = TopicProxy<Foo, MockRtps>;
        type TopicBar = TopicProxy<Bar, MockRtps>;

        let domain_participant = DdsShared::new(DomainParticipantAttributes::new(
            GuidPrefix([1; 12]),
            DomainId::default(),
            "".to_string(),
            DomainParticipantQos::default(),
            vec![],
            vec![],
            vec![],
            vec![],
        ));
        let domain_participant_proxy = DomainParticipantProxy::new(domain_participant.downgrade());

        domain_participant_proxy
            .topic_factory_create_topic("topic", None, None, 0)
            .unwrap() as TopicBar;

        assert!(
            (domain_participant_proxy.topic_factory_lookup_topicdescription("topic")
                as DdsResult<TopicFoo>)
                .is_err()
        );
    }

    #[test]
    fn topic_factory_lookup_topic_with_one_topic_with_wrong_name() {
        type Topic = TopicProxy<Foo, MockRtps>;

        let domain_participant = DdsShared::new(DomainParticipantAttributes::new(
            GuidPrefix([1; 12]),
            DomainId::default(),
            "".to_string(),
            DomainParticipantQos::default(),
            vec![],
            vec![],
            vec![],
            vec![],
        ));
        let domain_participant_proxy = DomainParticipantProxy::new(domain_participant.downgrade());

        domain_participant_proxy
            .topic_factory_create_topic("other_topic", None, None, 0)
            .unwrap() as Topic;

        assert!(
            (domain_participant_proxy.topic_factory_lookup_topicdescription("topic")
                as DdsResult<Topic>)
                .is_err()
        );
    }

    #[test]
    fn topic_factory_lookup_topic_with_two_types() {
        type TopicFoo = TopicProxy<Foo, MockRtps>;
        type TopicBar = TopicProxy<Bar, MockRtps>;

        let domain_participant = DdsShared::new(DomainParticipantAttributes::new(
            GuidPrefix([1; 12]),
            DomainId::default(),
            "".to_string(),
            DomainParticipantQos::default(),
            vec![],
            vec![],
            vec![],
            vec![],
        ));
        let domain_participant_proxy = DomainParticipantProxy::new(domain_participant.downgrade());

        let topic_foo = domain_participant_proxy
            .topic_factory_create_topic("topic", None, None, 0)
            .unwrap() as TopicFoo;
        let topic_bar = domain_participant_proxy
            .topic_factory_create_topic("topic", None, None, 0)
            .unwrap() as TopicBar;

        assert!(
            (domain_participant_proxy.topic_factory_lookup_topicdescription("topic")
                as DdsResult<TopicFoo>)
                .unwrap()
                .as_ref()
                .upgrade()
                .unwrap()
                == topic_foo.as_ref().upgrade().unwrap()
        );

        assert!(
            (domain_participant_proxy.topic_factory_lookup_topicdescription("topic")
                as DdsResult<TopicBar>)
                .unwrap()
                .as_ref()
                .upgrade()
                .unwrap()
                == topic_bar.as_ref().upgrade().unwrap()
        );
    }

    #[test]
    fn topic_factory_lookup_topic_with_two_topics() {
        type Topic = TopicProxy<Foo, MockRtps>;

        let domain_participant = DdsShared::new(DomainParticipantAttributes::new(
            GuidPrefix([1; 12]),
            DomainId::default(),
            "".to_string(),
            DomainParticipantQos::default(),
            vec![],
            vec![],
            vec![],
            vec![],
        ));
        let domain_participant_proxy = DomainParticipantProxy::new(domain_participant.downgrade());

        let topic1 = domain_participant_proxy
            .topic_factory_create_topic("topic1", None, None, 0)
            .unwrap() as Topic;
        let topic2 = domain_participant_proxy
            .topic_factory_create_topic("topic2", None, None, 0)
            .unwrap() as Topic;

        assert!(
            (domain_participant_proxy.topic_factory_lookup_topicdescription("topic1")
                as DdsResult<Topic>)
                .unwrap()
                .as_ref()
                .upgrade()
                .unwrap()
                == topic1.as_ref().upgrade().unwrap()
        );

        assert!(
            (domain_participant_proxy.topic_factory_lookup_topicdescription("topic2")
                as DdsResult<Topic>)
                .unwrap()
                .as_ref()
                .upgrade()
                .unwrap()
                == topic2.as_ref().upgrade().unwrap()
        );
    }
}
