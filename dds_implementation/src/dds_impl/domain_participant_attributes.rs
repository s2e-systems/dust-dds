use std::{
    sync::atomic::{AtomicU8, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use dds_api::{
    builtin_topics::{ParticipantBuiltinTopicData, TopicBuiltinTopicData},
    dcps_psm::{BuiltInTopicKey, DomainId, InstanceHandle, StatusMask, Time},
    domain::{
        domain_participant::{DomainParticipant, DomainParticipantTopicFactory},
        domain_participant_listener::DomainParticipantListener,
    },
    infrastructure::{
        entity::Entity,
        qos::{DomainParticipantQos, PublisherQos, SubscriberQos, TopicQos},
    },
    publication::{data_writer::DataWriter, publisher::Publisher},
    return_type::{DdsError, DdsResult},
    subscription::subscriber::Subscriber,
    topic::topic_description::TopicDescription,
};
use rtps_pim::{
    behavior::{
        reader::{
            stateful_reader::{RtpsStatefulReaderAttributes, RtpsStatefulReaderOperations},
            writer_proxy::{RtpsWriterProxyAttributes, RtpsWriterProxyConstructor},
        },
        writer::{
            reader_proxy::{RtpsReaderProxyAttributes, RtpsReaderProxyConstructor},
            stateful_writer::{RtpsStatefulWriterAttributes, RtpsStatefulWriterOperations},
        },
    },
    discovery::participant_discovery::ParticipantDiscovery,
    messages::types::Count,
    structure::{
        entity::RtpsEntityAttributes,
        group::RtpsGroupConstructor,
        participant::{RtpsParticipantAttributes, RtpsParticipantConstructor},
        types::{
            EntityId, Guid, GuidPrefix, Locator, ProtocolVersion, VendorId, ENTITYID_PARTICIPANT,
            PROTOCOLVERSION, USER_DEFINED_READER_GROUP, USER_DEFINED_WRITER_GROUP, VENDOR_ID_S2E,
        },
    },
};

use crate::{
    data_representation_builtin_endpoints::{
        discovered_reader_data::{DiscoveredReaderData, DCPS_SUBSCRIPTION},
        discovered_topic_data::{DiscoveredTopicData, DCPS_TOPIC},
        discovered_writer_data::{DiscoveredWriterData, DCPS_PUBLICATION},
        spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
    },
    dds_type::DdsType,
    utils::{
        rtps_structure::RtpsStructure,
        shared_object::{DdsRwLock, DdsShared},
    },
};

use super::{
    publisher_attributes::PublisherAttributes, subscriber_attributes::SubscriberAttributes,
    topic_attributes::TopicAttributes,
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

    pub fn get_builtin_publisher(&self) -> DdsResult<DdsShared<PublisherAttributes<Rtps>>> {
        Ok(self.builtin_publisher.read_lock().as_ref().unwrap().clone())
    }
}

impl<Rtps, Foo> DomainParticipantTopicFactory<Foo> for DdsShared<DomainParticipantAttributes<Rtps>>
where
    Rtps: RtpsStructure,
    Rtps::StatefulWriter: for<'a> RtpsStatefulWriterAttributes<'a>,
    for<'a> <Rtps::StatefulWriter as RtpsStatefulWriterAttributes<'a>>::ReaderProxyListType:
        IntoIterator,
    Foo: DdsType,
{
    type TopicType = DdsShared<TopicAttributes<Rtps>>;

    fn topic_factory_create_topic(
        &self,
        topic_name: &str,
        qos: Option<TopicQos>,
        _a_listener: Option<<Self::TopicType as Entity>::Listener>,
        _mask: StatusMask,
    ) -> DdsResult<Self::TopicType>
    where
        Self::TopicType: Entity,
    {
        let qos = qos.unwrap_or(self.default_topic_qos.clone());

        // /////// Create topic
        let topic_shared =
            TopicAttributes::new(qos.clone(), Foo::type_name(), topic_name, self.downgrade());

        self.topic_list.write_lock().push(topic_shared.clone());

        // /////// Announce the topic creation
        {
            let builtin_publisher_option = self.builtin_publisher.read_lock().clone();
            if let Some(builtin_publisher) = builtin_publisher_option {
                if let Ok(topic_creation_topic) =
                    self.lookup_topicdescription::<DiscoveredTopicData>(DCPS_TOPIC)
                {
                    if let Ok(sedp_builtin_topic_announcer) = builtin_publisher
                        .lookup_datawriter::<DiscoveredTopicData>(&topic_creation_topic)
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

        Ok(topic_shared)
    }

    fn topic_factory_delete_topic(&self, a_topic: &Self::TopicType) -> DdsResult<()> {
        let mut topic_list = self.topic_list.write_lock();
        let topic_list_position = topic_list.iter().position(|topic| topic == a_topic).ok_or(
            DdsError::PreconditionNotMet(
                "Topic can only be deleted from its parent publisher".to_string(),
            ),
        )?;

        // If topic is not attached to any reader or writer there must be no more than 2 strong counts
        // 1 strong stored in the list of the participant and 1 strong used to call this function
        if a_topic.strong_count() > 2 {
            return Err(DdsError::PreconditionNotMet(
                "Topic still attached to some data reader or data writer".to_string(),
            ));
        }

        topic_list.remove(topic_list_position);

        Ok(())
    }

    fn topic_factory_find_topic(
        &self,
        topic_name: &str,
        _timeout: dds_api::dcps_psm::Duration,
    ) -> DdsResult<Self::TopicType> {
        self.topic_list
            .read_lock()
            .iter()
            .find_map(|topic| {
                if topic.get_name().unwrap() == topic_name
                    && topic.get_type_name().unwrap() == Foo::type_name()
                {
                    Some(topic.clone())
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
        self.topic_list
            .read_lock()
            .iter()
            .find_map(|topic| {
                if topic.get_name().unwrap() == topic_name
                    && topic.get_type_name().unwrap() == Foo::type_name()
                {
                    Some(topic.clone())
                } else {
                    None
                }
            })
            .ok_or(DdsError::PreconditionNotMet("Not found".to_string()))
    }
}

impl<Rtps> DomainParticipant for DdsShared<DomainParticipantAttributes<Rtps>>
where
    Rtps: RtpsStructure,
{
    type PublisherType = DdsShared<PublisherAttributes<Rtps>>;
    type SubscriberType = DdsShared<SubscriberAttributes<Rtps>>;

    fn create_publisher(
        &self,
        qos: Option<PublisherQos>,
        _a_listener: Option<<Self::PublisherType as Entity>::Listener>,
        _mask: StatusMask,
    ) -> DdsResult<Self::PublisherType>
    where
        Self::PublisherType: Entity,
    {
        let publisher_qos = qos.unwrap_or(self.default_publisher_qos.clone());
        let publisher_counter = self
            .user_defined_publisher_counter
            .fetch_add(1, Ordering::Relaxed);
        let entity_id = EntityId::new([publisher_counter, 0, 0], USER_DEFINED_WRITER_GROUP);
        let guid = Guid::new(self.rtps_participant.guid().prefix(), entity_id);
        let rtps_group = Rtps::Group::new(guid);
        let publisher_impl_shared =
            PublisherAttributes::new(publisher_qos, rtps_group, self.downgrade());
        self.user_defined_publisher_list
            .write_lock()
            .push(publisher_impl_shared.clone());

        Ok(publisher_impl_shared)
    }

    fn delete_publisher(&self, a_publisher: &Self::PublisherType) -> DdsResult<()> {
        if !DdsShared::ptr_eq(&a_publisher.get_participant()?.upgrade()?, self) {
            return Err(DdsError::PreconditionNotMet(
                "Publisher can only be deleted from its parent participant".to_string(),
            ));
        }

        if !a_publisher.is_empty() {
            return Err(DdsError::PreconditionNotMet(
                "Publisher still contains data writers".to_string(),
            ));
        }

        self.user_defined_publisher_list
            .write_lock()
            .retain(|x| x != a_publisher);

        Ok(())
    }

    fn create_subscriber(
        &self,
        qos: Option<SubscriberQos>,
        _a_listener: Option<<Self::SubscriberType as Entity>::Listener>,
        _mask: StatusMask,
    ) -> DdsResult<Self::SubscriberType>
    where
        Self::SubscriberType: Entity,
    {
        let subscriber_qos = qos.unwrap_or(self.default_subscriber_qos.clone());
        let subcriber_counter = self
            .user_defined_subscriber_counter
            .fetch_add(1, Ordering::Relaxed);
        let entity_id = EntityId::new([subcriber_counter, 0, 0], USER_DEFINED_READER_GROUP);
        let guid = Guid::new(self.rtps_participant.guid().prefix(), entity_id);
        let rtps_group = Rtps::Group::new(guid);
        let subscriber_shared =
            SubscriberAttributes::new(subscriber_qos, rtps_group, self.downgrade());
        self.user_defined_subscriber_list
            .write_lock()
            .push(subscriber_shared.clone());

        Ok(subscriber_shared)
    }

    fn delete_subscriber(&self, a_subscriber: &Self::SubscriberType) -> DdsResult<()> {
        if !DdsShared::ptr_eq(&a_subscriber.get_participant()?.upgrade()?, self) {
            return Err(DdsError::PreconditionNotMet(
                "Subscriber can only be deleted from its parent participant".to_string(),
            ));
        }

        if !a_subscriber.is_empty() {
            return Err(DdsError::PreconditionNotMet(
                "Subscriber still contains data readers".to_string(),
            ));
        }

        self.user_defined_subscriber_list
            .write_lock()
            .retain(|x| x != a_subscriber);
        Ok(())
    }

    fn get_builtin_subscriber(&self) -> DdsResult<Self::SubscriberType> {
        Ok(self
            .builtin_subscriber
            .read_lock()
            .as_ref()
            .unwrap()
            .clone())
    }

    fn ignore_participant(&self, _handle: InstanceHandle) -> DdsResult<()> {
        todo!()
    }

    fn ignore_topic(&self, _handle: InstanceHandle) -> DdsResult<()> {
        todo!()
    }

    fn ignore_publication(&self, _handle: InstanceHandle) -> DdsResult<()> {
        todo!()
    }

    fn ignore_subscription(&self, _handle: InstanceHandle) -> DdsResult<()> {
        todo!()
    }

    fn get_domain_id(&self) -> DdsResult<DomainId> {
        todo!()
    }

    fn delete_contained_entities(&self) -> DdsResult<()> {
        todo!()
    }

    fn assert_liveliness(&self) -> DdsResult<()> {
        todo!()
    }

    fn set_default_publisher_qos(&self, _qos: Option<PublisherQos>) -> DdsResult<()> {
        todo!()
    }

    fn get_default_publisher_qos(&self) -> DdsResult<PublisherQos> {
        todo!()
    }

    fn set_default_subscriber_qos(&self, _qos: Option<SubscriberQos>) -> DdsResult<()> {
        todo!()
    }

    fn get_default_subscriber_qos(&self) -> DdsResult<SubscriberQos> {
        todo!()
    }

    fn set_default_topic_qos(&self, _qos: Option<TopicQos>) -> DdsResult<()> {
        todo!()
    }

    fn get_default_topic_qos(&self) -> DdsResult<TopicQos> {
        todo!()
    }

    fn get_discovered_participants(&self) -> DdsResult<Vec<InstanceHandle>> {
        todo!()
    }

    fn get_discovered_participant_data(
        &self,
        _participant_data: ParticipantBuiltinTopicData,
        _participant_handle: InstanceHandle,
    ) -> DdsResult<()> {
        todo!()
    }

    fn get_discovered_topics(&self, _topic_handles: &mut [InstanceHandle]) -> DdsResult<()> {
        todo!()
    }

    fn get_discovered_topic_data(
        &self,
        _topic_data: TopicBuiltinTopicData,
        _topic_handle: InstanceHandle,
    ) -> DdsResult<()> {
        todo!()
    }

    fn contains_entity(&self, _a_handle: InstanceHandle) -> DdsResult<bool> {
        todo!()
    }

    fn get_current_time(&self) -> DdsResult<dds_api::dcps_psm::Time> {
        let now_system_time = SystemTime::now();
        match now_system_time.duration_since(UNIX_EPOCH) {
            Ok(unix_time) => Ok(Time {
                sec: unix_time.as_secs() as i32,
                nanosec: unix_time.subsec_nanos(),
            }),
            Err(_) => Err(DdsError::Error),
        }
    }
}

impl<Rtps> Entity for DdsShared<DomainParticipantAttributes<Rtps>>
where
    Rtps: RtpsStructure,
{
    type Qos = DomainParticipantQos;
    type Listener = Box<dyn DomainParticipantListener>;

    fn set_qos(&self, _qos: Option<Self::Qos>) -> DdsResult<()> {
        todo!()
    }

    fn get_qos(&self) -> DdsResult<Self::Qos> {
        todo!()
    }

    fn set_listener(
        &self,
        _a_listener: Option<Self::Listener>,
        _mask: StatusMask,
    ) -> DdsResult<()> {
        todo!()
    }

    fn get_listener(&self) -> DdsResult<Option<Self::Listener>> {
        todo!()
    }

    fn get_statuscondition(&self) -> DdsResult<dds_api::infrastructure::entity::StatusCondition> {
        todo!()
    }

    fn get_status_changes(&self) -> DdsResult<StatusMask> {
        todo!()
    }

    fn enable(&self) -> DdsResult<()> {
        *self.enabled.write_lock() = true;
        Ok(())
    }

    fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        todo!()
    }
}

pub trait AddDiscoveredParticipant {
    fn add_discovered_participant(
        &self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    );
}

impl<Rtps> AddDiscoveredParticipant for DdsShared<DomainParticipantAttributes<Rtps>>
where
    Rtps: RtpsStructure,
    Rtps::StatefulWriter: for<'a> RtpsStatefulWriterAttributes<'a> + RtpsStatefulWriterOperations,
    <Rtps::StatefulWriter as RtpsStatefulWriterOperations>::ReaderProxyType: RtpsReaderProxyConstructor,
    Rtps::StatefulReader: for<'a> RtpsStatefulReaderAttributes<'a> + RtpsStatefulReaderOperations,
    <Rtps::StatefulReader as RtpsStatefulReaderOperations>::WriterProxyType: RtpsWriterProxyConstructor,
    for<'a> <Rtps::StatefulWriter as RtpsStatefulWriterAttributes<'a>>::ReaderProxyListType:
        IntoIterator,
    for<'a> <<Rtps::StatefulWriter as RtpsStatefulWriterAttributes<'a>>::ReaderProxyListType as
        IntoIterator>::Item: RtpsReaderProxyAttributes,
    for<'a> <Rtps::StatefulReader as RtpsStatefulReaderAttributes<'a>>::WriterProxyListType:
        IntoIterator,
    for<'a> <<Rtps::StatefulReader as RtpsStatefulReaderAttributes<'a>>::WriterProxyListType as
        IntoIterator>::Item: RtpsWriterProxyAttributes,
{
    fn add_discovered_participant(
        &self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    ) {
        let dcps_publication_topic = self
            .lookup_topicdescription::<DiscoveredWriterData>(DCPS_PUBLICATION)
            .unwrap();
        let dcps_subscription_topic = self
            .lookup_topicdescription::<DiscoveredReaderData>(DCPS_SUBSCRIPTION)
            .unwrap();
        let dcps_topic_topic = self
            .lookup_topicdescription::<DiscoveredTopicData>(DCPS_TOPIC)
            .unwrap();

        if let Ok(participant_discovery) = ParticipantDiscovery::new(
            discovered_participant_data,
            self.domain_id as u32,
            &self.domain_tag,
        ) {
            let builtin_publisher_lock = self.builtin_publisher.read_lock();
            let builtin_subscriber_lock = self.builtin_subscriber.read_lock();
            let builtin_publisher = builtin_publisher_lock.as_ref().unwrap();
            let builtin_subscriber = builtin_subscriber_lock.as_ref().unwrap();
            let sedp_builtin_publication_writer_shared = builtin_publisher
                .lookup_datawriter::<DiscoveredWriterData>(&dcps_publication_topic)
                .unwrap();
            sedp_builtin_publication_writer_shared.add_matched_participant(&participant_discovery);

            let sedp_builtin_publication_reader_shared = builtin_subscriber
                .lookup_datareader::<DiscoveredWriterData>(&dcps_publication_topic)
                .unwrap();
            sedp_builtin_publication_reader_shared.add_matched_participant(&participant_discovery);

            let sedp_builtin_subscription_writer_shared = builtin_publisher
                .lookup_datawriter::<DiscoveredReaderData>(&dcps_subscription_topic)
                .unwrap();
            sedp_builtin_subscription_writer_shared.add_matched_participant(&participant_discovery);

            let sedp_builtin_subscription_reader_shared = builtin_subscriber
                .lookup_datareader::<DiscoveredReaderData>(&dcps_subscription_topic)
                .unwrap();
            sedp_builtin_subscription_reader_shared.add_matched_participant(&participant_discovery);

            let sedp_builtin_topic_writer_shared = builtin_publisher
                .lookup_datawriter::<DiscoveredTopicData>(&dcps_topic_topic)
                .unwrap();
            sedp_builtin_topic_writer_shared.add_matched_participant(&participant_discovery);

            let sedp_builtin_topic_reader_shared = builtin_subscriber
                .lookup_datareader::<DiscoveredTopicData>(&dcps_topic_topic)
                .unwrap();
            sedp_builtin_topic_reader_shared.add_matched_participant(&participant_discovery);
        }
    }
}

impl<Rtps> RtpsParticipantAttributes for DdsShared<DomainParticipantAttributes<Rtps>>
where
    Rtps: RtpsStructure,
{
    fn default_unicast_locator_list(&self) -> &[Locator] {
        self.rtps_participant.default_unicast_locator_list()
    }

    fn default_multicast_locator_list(&self) -> &[Locator] {
        self.rtps_participant.default_multicast_locator_list()
    }

    fn protocol_version(&self) -> ProtocolVersion {
        self.rtps_participant.protocol_version()
    }

    fn vendor_id(&self) -> VendorId {
        self.rtps_participant.vendor_id()
    }
}

#[cfg(test)]
mod tests {

    use dds_api::return_type::{DdsError, DdsResult};

    use super::*;
    use crate::{
        dds_type::{DdsSerialize, DdsType, Endianness},
        test_utils::mock_rtps::MockRtps,
    };
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
        let domain_participant = DdsShared::new(DomainParticipantAttributes::<MockRtps>::new(
            GuidPrefix([1; 12]),
            DomainId::default(),
            "".to_string(),
            DomainParticipantQos::default(),
            vec![],
            vec![],
            vec![],
            vec![],
        ));

        let len_before = domain_participant.topic_list.read_lock().len();

        let topic = domain_participant.create_topic::<Foo>("topic", None, None, 0);

        assert!(topic.is_ok());
        assert_eq!(
            len_before + 1,
            domain_participant.topic_list.read_lock().len()
        );
    }

    #[test]
    fn topic_factory_delete_topic() {
        let domain_participant = DdsShared::new(DomainParticipantAttributes::<MockRtps>::new(
            GuidPrefix([1; 12]),
            DomainId::default(),
            "".to_string(),
            DomainParticipantQos::default(),
            vec![],
            vec![],
            vec![],
            vec![],
        ));

        let len_before = domain_participant.topic_list.read_lock().len();

        let topic = domain_participant
            .create_topic::<Foo>("topic", None, None, 0)
            .unwrap();

        assert_eq!(
            len_before + 1,
            domain_participant.topic_list.read_lock().len()
        );

        domain_participant.delete_topic::<Foo>(&topic).unwrap();

        assert_eq!(len_before, domain_participant.topic_list.read_lock().len());
    }

    #[test]
    fn topic_factory_delete_topic_from_other_participant() {
        let domain_participant = DdsShared::new(DomainParticipantAttributes::<MockRtps>::new(
            GuidPrefix([1; 12]),
            DomainId::default(),
            "".to_string(),
            DomainParticipantQos::default(),
            vec![],
            vec![],
            vec![],
            vec![],
        ));

        let domain_participant2 = DdsShared::new(DomainParticipantAttributes::<MockRtps>::new(
            GuidPrefix([1; 12]),
            DomainId::default(),
            "".to_string(),
            DomainParticipantQos::default(),
            vec![],
            vec![],
            vec![],
            vec![],
        ));

        let len_before = domain_participant.topic_list.read_lock().len();
        let len_before2 = domain_participant2.topic_list.read_lock().len();

        let topic = domain_participant
            .create_topic::<Foo>("topic", None, None, 0)
            .unwrap();

        assert_eq!(
            len_before + 1,
            domain_participant.topic_list.read_lock().len()
        );
        assert_eq!(
            len_before2,
            domain_participant2.topic_list.read_lock().len()
        );

        assert!(matches!(
            domain_participant2.delete_topic::<Foo>(&topic),
            Err(DdsError::PreconditionNotMet(_))
        ));
    }

    #[test]
    fn topic_factory_lookup_topic_with_no_topic() {
        let domain_participant = DdsShared::new(DomainParticipantAttributes::<MockRtps>::new(
            GuidPrefix([1; 12]),
            DomainId::default(),
            "".to_string(),
            DomainParticipantQos::default(),
            vec![],
            vec![],
            vec![],
            vec![],
        ));

        assert!(domain_participant
            .lookup_topicdescription::<Foo>("topic")
            .is_err());
    }

    #[test]
    fn topic_factory_lookup_topic_with_one_topic() {
        let domain_participant = DdsShared::new(DomainParticipantAttributes::<MockRtps>::new(
            GuidPrefix([1; 12]),
            DomainId::default(),
            "".to_string(),
            DomainParticipantQos::default(),
            vec![],
            vec![],
            vec![],
            vec![],
        ));

        let topic = domain_participant
            .create_topic::<Foo>("topic", None, None, 0)
            .unwrap();

        assert!(
            domain_participant
                .lookup_topicdescription::<Foo>("topic")
                .unwrap()
                == topic
        );
    }

    make_empty_dds_type!(Bar);

    #[test]
    fn topic_factory_lookup_topic_with_one_topic_with_wrong_type() {
        let domain_participant = DdsShared::new(DomainParticipantAttributes::<MockRtps>::new(
            GuidPrefix([1; 12]),
            DomainId::default(),
            "".to_string(),
            DomainParticipantQos::default(),
            vec![],
            vec![],
            vec![],
            vec![],
        ));

        domain_participant
            .create_topic::<Bar>("topic", None, None, 0)
            .unwrap();

        assert!(domain_participant
            .lookup_topicdescription::<Foo>("topic")
            .is_err());
    }

    #[test]
    fn topic_factory_lookup_topic_with_one_topic_with_wrong_name() {
        let domain_participant = DdsShared::new(DomainParticipantAttributes::<MockRtps>::new(
            GuidPrefix([1; 12]),
            DomainId::default(),
            "".to_string(),
            DomainParticipantQos::default(),
            vec![],
            vec![],
            vec![],
            vec![],
        ));

        domain_participant
            .create_topic::<Foo>("other_topic", None, None, 0)
            .unwrap();

        assert!(domain_participant
            .lookup_topicdescription::<Foo>("topic")
            .is_err());
    }

    #[test]
    fn topic_factory_lookup_topic_with_two_types() {
        let domain_participant = DdsShared::new(DomainParticipantAttributes::<MockRtps>::new(
            GuidPrefix([1; 12]),
            DomainId::default(),
            "".to_string(),
            DomainParticipantQos::default(),
            vec![],
            vec![],
            vec![],
            vec![],
        ));

        let topic_foo = domain_participant
            .create_topic::<Foo>("topic", None, None, 0)
            .unwrap();
        let topic_bar = domain_participant
            .create_topic::<Bar>("topic", None, None, 0)
            .unwrap();

        assert!(
            domain_participant
                .lookup_topicdescription::<Foo>("topic")
                .unwrap()
                == topic_foo
        );

        assert!(
            domain_participant
                .lookup_topicdescription::<Bar>("topic")
                .unwrap()
                == topic_bar
        );
    }

    #[test]
    fn topic_factory_lookup_topic_with_two_topics() {
        let domain_participant = DdsShared::new(DomainParticipantAttributes::<MockRtps>::new(
            GuidPrefix([1; 12]),
            DomainId::default(),
            "".to_string(),
            DomainParticipantQos::default(),
            vec![],
            vec![],
            vec![],
            vec![],
        ));

        let topic1 = domain_participant
            .create_topic::<Foo>("topic1", None, None, 0)
            .unwrap();
        let topic2 = domain_participant
            .create_topic::<Foo>("topic2", None, None, 0)
            .unwrap();

        assert!(
            domain_participant
                .lookup_topicdescription::<Foo>("topic1")
                .unwrap()
                == topic1
        );

        assert!(
            domain_participant
                .lookup_topicdescription::<Foo>("topic2")
                .unwrap()
                == topic2
        );
    }
}
