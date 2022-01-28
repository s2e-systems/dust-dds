use std::sync::{
    atomic::{self, AtomicBool, AtomicU8},
    Arc,
};

use rust_dds_api::{
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
    publication::{publisher::Publisher, publisher_listener::PublisherListener},
    return_type::{DDSError, DDSResult},
    subscription::{subscriber::Subscriber, subscriber_listener::SubscriberListener},
    topic::{topic_description::TopicDescription, topic_listener::TopicListener},
};
use rust_rtps_pim::{
    discovery::types::{BuiltinEndpointQos, BuiltinEndpointSet},
    messages::types::Count,
    structure::{
        entity::RtpsEntityAttributes,
        participant::RtpsParticipantAttributes,
        types::{
            EntityId, Guid, GuidPrefix, Locator, ENTITYID_PARTICIPANT, PROTOCOLVERSION,
            USER_DEFINED_READER_GROUP, USER_DEFINED_WRITER_GROUP, VENDOR_ID_S2E,
        },
    },
};

use crate::{
    data_representation_builtin_endpoints::{
        sedp_discovered_writer_data::SedpDiscoveredWriterData,
        spdp_discovered_participant_data::{ParticipantProxy, SpdpDiscoveredParticipantData},
    },
    dds_type::DdsType,
    rtps_impl::{rtps_group_impl::RtpsGroupImpl, rtps_participant_impl::RtpsParticipantImpl},
    utils::shared_object::{
        rtps_shared_downgrade, rtps_shared_new, rtps_shared_read_lock, rtps_shared_write_lock,
        rtps_weak_upgrade, RtpsShared, RtpsWeak,
    },
};

use super::{
    publisher_impl::PublisherImpl, publisher_proxy::PublisherProxy,
    subscriber_impl::SubscriberImpl, subscriber_proxy::SubscriberProxy, topic_impl::TopicImpl,
    topic_proxy::TopicProxy,
};

pub struct DomainParticipantAttributes {
    rtps_participant: RtpsParticipantImpl,
    domain_id: DomainId,
    domain_tag: Arc<String>,
    qos: DomainParticipantQos,
    builtin_subscriber: RtpsShared<SubscriberImpl>,
    builtin_publisher: RtpsShared<PublisherImpl>,
    user_defined_subscriber_list: RtpsShared<Vec<RtpsShared<SubscriberImpl>>>,
    user_defined_subscriber_counter: AtomicU8,
    default_subscriber_qos: SubscriberQos,
    user_defined_publisher_list: RtpsShared<Vec<RtpsShared<PublisherImpl>>>,
    user_defined_publisher_counter: AtomicU8,
    default_publisher_qos: PublisherQos,
    topic_list: RtpsShared<Vec<RtpsShared<TopicImpl>>>,
    default_topic_qos: TopicQos,
    manual_liveliness_count: Count,
    lease_duration: rust_rtps_pim::behavior::types::Duration,
    metatraffic_unicast_locator_list: Vec<Locator>,
    metatraffic_multicast_locator_list: Vec<Locator>,
    enabled: Arc<AtomicBool>,
}

impl DomainParticipantAttributes {
    pub fn new(
        guid_prefix: GuidPrefix,
        domain_id: DomainId,
        domain_tag: Arc<String>,
        domain_participant_qos: DomainParticipantQos,
        metatraffic_unicast_locator_list: Vec<Locator>,
        metatraffic_multicast_locator_list: Vec<Locator>,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
        builtin_subscriber: RtpsShared<SubscriberImpl>,
        builtin_publisher: RtpsShared<PublisherImpl>,
        user_defined_subscriber_list: RtpsShared<Vec<RtpsShared<SubscriberImpl>>>,
        user_defined_publisher_list: RtpsShared<Vec<RtpsShared<PublisherImpl>>>,
        enabled: Arc<AtomicBool>,
    ) -> Self {
        let lease_duration = rust_rtps_pim::behavior::types::Duration::new(100, 0);
        let protocol_version = PROTOCOLVERSION;
        let vendor_id = VENDOR_ID_S2E;
        let rtps_participant = RtpsParticipantImpl::new(
            Guid::new(guid_prefix, ENTITYID_PARTICIPANT),
            protocol_version,
            vendor_id,
            default_unicast_locator_list,
            default_multicast_locator_list,
        );

        Self {
            rtps_participant,
            domain_id,
            domain_tag,
            qos: domain_participant_qos,
            builtin_subscriber,
            builtin_publisher,
            user_defined_subscriber_list,
            user_defined_subscriber_counter: AtomicU8::new(0),
            default_subscriber_qos: SubscriberQos::default(),
            user_defined_publisher_list,
            user_defined_publisher_counter: AtomicU8::new(0),
            default_publisher_qos: PublisherQos::default(),
            topic_list: rtps_shared_new(Vec::new()),
            default_topic_qos: TopicQos::default(),
            manual_liveliness_count: Count(0),
            lease_duration,
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
            enabled,
        }
    }

    pub fn as_spdp_discovered_participant_data(&self) -> SpdpDiscoveredParticipantData {
        SpdpDiscoveredParticipantData {
            dds_participant_data: ParticipantBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: (*self.rtps_participant.guid()).into(),
                },
                user_data: self.qos.user_data.clone(),
            },
            participant_proxy: ParticipantProxy {
                domain_id: self.domain_id as u32,
                domain_tag: self.domain_tag.as_ref().clone(),
                protocol_version: *self.rtps_participant.protocol_version(),
                guid_prefix: *self.rtps_participant.guid().prefix(),
                vendor_id: *self.rtps_participant.vendor_id(),
                expects_inline_qos: false,
                metatraffic_unicast_locator_list: self.metatraffic_unicast_locator_list.clone(),
                metatraffic_multicast_locator_list: self.metatraffic_multicast_locator_list.clone(),
                default_unicast_locator_list: self
                    .rtps_participant
                    .default_unicast_locator_list()
                    .to_vec(),
                default_multicast_locator_list: self
                    .rtps_participant
                    .default_multicast_locator_list()
                    .to_vec(),
                available_builtin_endpoints: BuiltinEndpointSet::default(),
                manual_liveliness_count: self.manual_liveliness_count,
                builtin_endpoint_qos: BuiltinEndpointQos::default(),
            },
            lease_duration: self.lease_duration,
        }
    }
}

#[derive(Clone)]
pub struct DomainParticipantProxy {
    domain_participant: RtpsWeak<DomainParticipantAttributes>,
}

impl DomainParticipantProxy {
    pub fn new(domain_participant: RtpsWeak<DomainParticipantAttributes>) -> Self {
        Self { domain_participant }
    }
}

impl<Foo> DomainParticipantTopicFactory<Foo> for DomainParticipantProxy
where
    Foo: DdsType + 'static,
{
    type TopicType = TopicProxy<Foo>;

    fn topic_factory_create_topic(
        &self,
        topic_name: &str,
        qos: Option<TopicQos>,
        _a_listener: Option<Box<dyn TopicListener>>,
        _mask: StatusMask,
    ) -> Option<Self::TopicType> {
        let domain_participant_attributes = rtps_weak_upgrade(&self.domain_participant).ok()?;
        let domain_participant_attributes_lock =
            rtps_shared_read_lock(&domain_participant_attributes);
        let topic_qos = qos.unwrap_or(domain_participant_attributes_lock.default_topic_qos.clone());

        let _builtin_publisher_lock =
            rtps_shared_read_lock(&domain_participant_attributes_lock.builtin_publisher);
        // if let Some(sedp_builtin_topics_writer) =
        //     builtin_publisher_lock.lookup_datawriter::<SedpDiscoveredTopicData>(&())
        // {
        //     let mut sedp_builtin_topics_writer_lock =
        //         rtps_shared_write_lock(&sedp_builtin_topics_writer);
        //     let sedp_discovered_topic_data = SedpDiscoveredTopicData {
        //         topic_builtin_topic_data: TopicBuiltinTopicData {
        //             key: BuiltInTopicKey { value: [1; 16] },
        //             name: topic_name.to_string(),
        //             type_name: Foo::type_name().to_string(),
        //             durability: topic_qos.durability.clone(),
        //             durability_service: topic_qos.durability_service.clone(),
        //             deadline: topic_qos.deadline.clone(),
        //             latency_budget: topic_qos.latency_budget.clone(),
        //             liveliness: topic_qos.liveliness.clone(),
        //             reliability: topic_qos.reliability.clone(),
        //             transport_priority: topic_qos.transport_priority.clone(),
        //             lifespan: topic_qos.lifespan.clone(),
        //             destination_order: topic_qos.destination_order.clone(),
        //             history: topic_qos.history.clone(),
        //             resource_limits: topic_qos.resource_limits.clone(),
        //             ownership: topic_qos.ownership.clone(),
        //             topic_data: topic_qos.topic_data.clone(),
        //         },
        //     };
        //     sedp_builtin_topics_writer_lock
        //         .write_w_timestamp(
        //             &sedp_discovered_topic_data,
        //             None,
        //             Time { sec: 0, nanosec: 0 },
        //         )
        //         .ok()?;
        // }

        let topic_impl = TopicImpl::new(topic_qos, Foo::type_name(), topic_name);
        let topic_impl_shared = rtps_shared_new(topic_impl);
        rtps_shared_write_lock(&domain_participant_attributes_lock.topic_list)
            .push(topic_impl_shared.clone());

        let topic_weak = rtps_shared_downgrade(&topic_impl_shared);
        Some(TopicProxy::new(self.clone(), topic_weak))
    }

    fn topic_factory_delete_topic(&self, a_topic: &Self::TopicType) -> DDSResult<()> {
        let domain_participant_attributes = rtps_weak_upgrade(&self.domain_participant)?;
        let domain_participant_attributes_lock =
            rtps_shared_read_lock(&domain_participant_attributes);
        let topic_shared = rtps_weak_upgrade(a_topic.as_ref())?;
        if std::ptr::eq(&a_topic.get_participant(), self) {
            rtps_shared_write_lock(&domain_participant_attributes_lock.topic_list)
                .retain(|x| !Arc::ptr_eq(x, &topic_shared));
            Ok(())
        } else {
            Err(DDSError::PreconditionNotMet(
                "Subscriber can only be deleted from its parent participant".to_string(),
            ))
        }
    }

    fn topic_factory_find_topic(
        &self,
        _topic_name: &str,
        _timeout: Duration,
    ) -> Option<Self::TopicType> {
        // Explicit call with the complete function path otherwise the generic type can't be infered.
        // This happens because TopicImpl has no generic type information.
        // let domain_participant = rtps_shared_read_lock(&domain_participant_lock)
        // let topic_shared = DomainParticipantTopicFactory::<'t, Foo>::topic_factory_find_topic(
        //     &*,
        //     topic_name,
        //     timeout,
        // )?;
        // let topic_weak = rtps_shared_downgrade(&topic_shared);
        // Some(TopicProxy::new(self, topic_weak))
        todo!()
    }
}

impl DomainParticipant for DomainParticipantProxy {
    type PublisherType = PublisherProxy;
    type SubscriberType = SubscriberProxy;

    fn create_publisher(
        &self,
        qos: Option<PublisherQos>,
        _a_listener: Option<&'static dyn PublisherListener>,
        _mask: StatusMask,
    ) -> Option<Self::PublisherType> {
        let domain_participant_attributes = rtps_weak_upgrade(&self.domain_participant).ok()?;
        let domain_participant_attributes_lock =
            rtps_shared_read_lock(&domain_participant_attributes);
        let publisher_qos = qos.unwrap_or(
            domain_participant_attributes_lock
                .default_publisher_qos
                .clone(),
        );
        let user_defined_publisher_counter = domain_participant_attributes_lock
            .user_defined_publisher_counter
            .fetch_add(1, atomic::Ordering::SeqCst);
        let entity_id = EntityId::new(
            [user_defined_publisher_counter, 0, 0],
            USER_DEFINED_WRITER_GROUP,
        );
        let guid = Guid::new(
            *domain_participant_attributes_lock
                .rtps_participant
                .guid()
                .prefix(),
            entity_id,
        );
        let rtps_group = RtpsGroupImpl::new(guid);
        let sedp_builtin_publications_topic =
            rtps_shared_new(TopicImpl::new(TopicQos::default(), "", ""));
        let sedp_builtin_publications_announcer =
            rtps_shared_read_lock(&domain_participant_attributes_lock.builtin_publisher)
                .lookup_datawriter::<SedpDiscoveredWriterData>(&sedp_builtin_publications_topic);
        let publisher_impl = PublisherImpl::new(
            publisher_qos,
            rtps_group,
            Vec::new(),
            sedp_builtin_publications_announcer,
        );
        let publisher_impl_shared = rtps_shared_new(publisher_impl);
        rtps_shared_write_lock(&domain_participant_attributes_lock.user_defined_publisher_list)
            .push(publisher_impl_shared.clone());

        let publisher_weak = rtps_shared_downgrade(&publisher_impl_shared);

        Some(PublisherProxy::new(self.clone(), publisher_weak))
    }

    fn delete_publisher(&self, a_publisher: &Self::PublisherType) -> DDSResult<()> {
        let domain_participant_attributes = rtps_weak_upgrade(&self.domain_participant)?;
        let domain_participant_attributes_lock =
            rtps_shared_read_lock(&domain_participant_attributes);
        let publisher_shared = rtps_weak_upgrade(a_publisher.as_ref())?;
        if std::ptr::eq(&a_publisher.get_participant(), self) {
            // rtps_shared_read_lock(&domain_participant_lock).delete_publisher(&publisher_shared)
            rtps_shared_write_lock(&domain_participant_attributes_lock.user_defined_publisher_list)
                .retain(|x| !Arc::ptr_eq(&x, &publisher_shared));
            Ok(())
        } else {
            Err(DDSError::PreconditionNotMet(
                "Publisher can only be deleted from its parent participant".to_string(),
            ))
        }
    }

    fn create_subscriber(
        &self,
        qos: Option<SubscriberQos>,
        _a_listener: Option<&'static dyn SubscriberListener>,
        _mask: StatusMask,
    ) -> Option<Self::SubscriberType> {
        let domain_participant_attributes = rtps_weak_upgrade(&self.domain_participant).ok()?;
        let domain_participant_attributes_lock =
            rtps_shared_read_lock(&domain_participant_attributes);
        // let subscriber_shared = rtps_shared_read_lock(&domain_participant_lock)
        // .create_subscriber(qos, a_listener, mask)?;

        let subscriber_qos = qos.unwrap_or(
            domain_participant_attributes_lock
                .default_subscriber_qos
                .clone(),
        );
        let user_defined_subscriber_counter = domain_participant_attributes_lock
            .user_defined_subscriber_counter
            .fetch_add(1, atomic::Ordering::SeqCst);
        let entity_id = EntityId::new(
            [user_defined_subscriber_counter, 0, 0],
            USER_DEFINED_READER_GROUP,
        );
        let guid = Guid::new(
            *domain_participant_attributes_lock
                .rtps_participant
                .guid()
                .prefix(),
            entity_id,
        );
        let rtps_group = RtpsGroupImpl::new(guid);
        let subscriber = SubscriberImpl::new(subscriber_qos, rtps_group, Vec::new());
        let subscriber_shared = rtps_shared_new(subscriber);
        rtps_shared_write_lock(&domain_participant_attributes_lock.user_defined_subscriber_list)
            .push(subscriber_shared.clone());

        let subscriber_weak = rtps_shared_downgrade(&subscriber_shared);
        Some(SubscriberProxy::new(self.clone(), subscriber_weak))
    }

    fn delete_subscriber(&self, a_subscriber: &Self::SubscriberType) -> DDSResult<()> {
        let domain_participant_attributes = rtps_weak_upgrade(&self.domain_participant)?;
        let domain_participant_attributes_lock =
            rtps_shared_read_lock(&domain_participant_attributes);
        let subscriber_shared = rtps_weak_upgrade(a_subscriber.as_ref())?;
        if std::ptr::eq(&a_subscriber.get_participant(), self) {
            // rtps_shared_read_lock(&domain_participant_lock).delete_subscriber(&subscriber_shared)
            rtps_shared_write_lock(
                &domain_participant_attributes_lock.user_defined_subscriber_list,
            )
            .retain(|x| !Arc::ptr_eq(&x, &subscriber_shared));
            Ok(())
        } else {
            Err(DDSError::PreconditionNotMet(
                "Subscriber can only be deleted from its parent participant".to_string(),
            ))
        }
    }

    fn lookup_topicdescription<Foo>(
        &self,
        _name: &str,
    ) -> Option<&dyn TopicDescription<DomainParticipant = Self>>
    where
        Self: Sized,
    {
        todo!()
        // rtps_shared_read_lock(&domain_participant_lock).lookup_topicdescription(name)
    }

    fn get_builtin_subscriber(&self) -> DDSResult<Self::SubscriberType> {
        let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        let subscriber_shared = domain_participant_lock.builtin_subscriber.clone();
        let subscriber_weak = rtps_shared_downgrade(&subscriber_shared);
        Ok(SubscriberProxy::new(self.clone(), subscriber_weak))
    }

    fn ignore_participant(&self, _handle: InstanceHandle) -> DDSResult<()> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.ignore_participant(handle)
        todo!()
    }

    fn ignore_topic(&self, _handle: InstanceHandle) -> DDSResult<()> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.ignore_topic(handle)
        todo!()
    }

    fn ignore_publication(&self, _handle: InstanceHandle) -> DDSResult<()> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.ignore_publication(handle)
        todo!()
    }

    fn ignore_subscription(&self, _handle: InstanceHandle) -> DDSResult<()> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.ignore_subscription(handle)
        todo!()
    }

    fn get_domain_id(&self) -> DDSResult<DomainId> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.get_domain_id()
        todo!()
    }

    fn delete_contained_entities(&self) -> DDSResult<()> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.delete_contained_entities()
        todo!()
    }

    fn assert_liveliness(&self) -> DDSResult<()> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.assert_liveliness()
        todo!()
    }

    fn set_default_publisher_qos(&mut self, _qos: Option<PublisherQos>) -> DDSResult<()> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let mut domain_participant_lock = rtps_shared_write_lock(&domain_participant_shared);
        // domain_participant_lock.set_default_publisher_qos(qos)
        todo!()
    }

    fn get_default_publisher_qos(&self) -> DDSResult<PublisherQos> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.get_default_publisher_qos()
        todo!()
    }

    fn set_default_subscriber_qos(&mut self, _qos: Option<SubscriberQos>) -> DDSResult<()> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let mut domain_participant_lock = rtps_shared_write_lock(&domain_participant_shared);
        // domain_participant_lock.set_default_subscriber_qos(qos)
        todo!()
    }

    fn get_default_subscriber_qos(&self) -> DDSResult<SubscriberQos> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.get_default_subscriber_qos()
        todo!()
    }

    fn set_default_topic_qos(&mut self, _qos: Option<TopicQos>) -> DDSResult<()> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let mut domain_participant_lock = rtps_shared_write_lock(&domain_participant_shared);
        // domain_participant_lock.set_default_topic_qos(qos)
        todo!()
    }

    fn get_default_topic_qos(&self) -> DDSResult<TopicQos> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.get_default_topic_qos()
        todo!()
    }

    fn get_discovered_participants(
        &self,
        _participant_handles: &mut [InstanceHandle],
    ) -> DDSResult<()> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.get_discovered_participants(participant_handles)
        todo!()
    }

    fn get_discovered_participant_data(
        &self,
        _participant_data: ParticipantBuiltinTopicData,
        _participant_handle: InstanceHandle,
    ) -> DDSResult<()> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock
        //     .get_discovered_participant_data(participant_data, participant_handle)
        todo!()
    }

    fn get_discovered_topics(&self, _topic_handles: &mut [InstanceHandle]) -> DDSResult<()> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.get_discovered_topics(topic_handles)
        todo!()
    }

    fn get_discovered_topic_data(
        &self,
        _topic_data: TopicBuiltinTopicData,
        _topic_handle: InstanceHandle,
    ) -> DDSResult<()> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.get_discovered_topic_data(topic_data, topic_handle)
        todo!()
    }

    fn contains_entity(&self, _a_handle: InstanceHandle) -> DDSResult<bool> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.contains_entity(a_handle)
        todo!()
    }

    fn get_current_time(&self) -> DDSResult<Time> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.get_current_time()
        todo!()
    }
}

impl Entity for DomainParticipantProxy {
    type Qos = DomainParticipantQos;
    type Listener = &'static dyn DomainParticipantListener;

    fn set_qos(&mut self, _qos: Option<Self::Qos>) -> DDSResult<()> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let mut domain_participant_lock = rtps_shared_write_lock(&domain_participant_shared);
        // domain_participant_lock.set_qos(qos)
        todo!()
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.get_qos()
        todo!()
    }

    fn set_listener(
        &self,
        _a_listener: Option<Self::Listener>,
        _mask: StatusMask,
    ) -> DDSResult<()> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.set_listener(a_listener, mask)
        todo!()
    }

    fn get_listener(&self) -> DDSResult<Option<Self::Listener>> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.get_listener()
        todo!()
    }

    fn get_statuscondition(&self) -> DDSResult<StatusCondition> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.get_statuscondition()
        todo!()
    }

    fn get_status_changes(&self) -> DDSResult<StatusMask> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.get_status_changes()
        todo!()
    }

    fn enable(&self) -> DDSResult<()> {
        let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        domain_participant_lock
            .enabled
            .store(true, atomic::Ordering::SeqCst);
        Ok(())
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        // let domain_participant_shared = rtps_weak_upgrade(&self.domain_participant)?;
        // let domain_participant_lock = rtps_shared_read_lock(&domain_participant_shared);
        // domain_participant_lock.get_instance_handle()
        todo!()
    }
}
