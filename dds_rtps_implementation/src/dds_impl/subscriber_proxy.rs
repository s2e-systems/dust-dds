use rust_dds_api::{
    builtin_topics::SubscriptionBuiltinTopicData,
    dcps_psm::{
        BuiltInTopicKey, Duration, InstanceHandle, InstanceStateKind, SampleLostStatus,
        SampleStateKind, StatusMask, ViewStateKind,
    },
    domain::domain_participant::DomainParticipantTopicFactory,
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::{DataReaderQos, SubscriberQos, TopicQos},
        qos_policy::{
            DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy, GroupDataQosPolicy,
            LatencyBudgetQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy, PartitionQosPolicy,
            PresentationQosPolicy, ReliabilityQosPolicy, ReliabilityQosPolicyKind,
            TimeBasedFilterQosPolicy, TopicDataQosPolicy, UserDataQosPolicy,
        },
    },
    publication::{data_writer::DataWriter, publisher::PublisherDataWriterFactory},
    return_type::{DDSError, DDSResult},
    subscription::{
        data_reader::AnyDataReader,
        data_reader_listener::DataReaderListener,
        subscriber::{Subscriber, SubscriberDataReaderFactory},
        subscriber_listener::SubscriberListener,
    },
};

use rust_rtps_pim::{
    behavior::{
        reader::stateful_reader::RtpsStatefulReaderConstructor,
        writer::{
            stateful_writer::RtpsStatefulWriterConstructor,
            writer::{RtpsWriterAttributes, RtpsWriterOperations},
        },
    },
    structure::{
        entity::RtpsEntityAttributes,
        history_cache::RtpsHistoryCacheOperations,
        participant::RtpsParticipantAttributes,
        types::{
            EntityId, Guid, ReliabilityKind, TopicKind, USER_DEFINED_WRITER_NO_KEY,
            USER_DEFINED_WRITER_WITH_KEY,
        },
    },
};

use crate::{
    data_representation_builtin_endpoints::sedp_discovered_reader_data::{
        RtpsReaderProxy, SedpDiscoveredReaderData, DCPS_SUBSCRIPTION,
    },
    dds_type::{DdsDeserialize, DdsType},
    utils::{
        rtps_structure::RtpsStructure,
        shared_object::{RtpsShared, RtpsWeak},
    },
};

use super::{
    data_reader_proxy::{DataReaderAttributes, DataReaderProxy, RtpsReader},
    domain_participant_proxy::{DomainParticipantAttributes, DomainParticipantProxy},
    publisher_proxy::PublisherProxy,
    topic_proxy::TopicProxy,
};

pub struct SubscriberAttributes<Rtps>
where
    Rtps: RtpsStructure,
{
    pub qos: SubscriberQos,
    pub rtps_group: Rtps::Group,
    pub data_reader_list: Vec<RtpsShared<DataReaderAttributes<Rtps>>>,
    pub user_defined_data_reader_counter: u8,
    pub default_data_reader_qos: DataReaderQos,
    pub parent_domain_participant: RtpsWeak<DomainParticipantAttributes<Rtps>>,
}

impl<Rtps> SubscriberAttributes<Rtps>
where
    Rtps: RtpsStructure,
{
    pub fn new(
        qos: SubscriberQos,
        rtps_group: Rtps::Group,
        parent_domain_participant: RtpsWeak<DomainParticipantAttributes<Rtps>>,
    ) -> Self {
        Self {
            qos,
            rtps_group,
            data_reader_list: Vec::new(),
            user_defined_data_reader_counter: 0,
            default_data_reader_qos: DataReaderQos::default(),
            parent_domain_participant,
        }
    }
}

#[derive(Clone)]
pub struct SubscriberProxy<Rtps>
where
    Rtps: RtpsStructure,
{
    participant: DomainParticipantProxy<Rtps>,
    subscriber_impl: RtpsWeak<SubscriberAttributes<Rtps>>,
}

impl<Rtps> SubscriberProxy<Rtps>
where
    Rtps: RtpsStructure,
{
    pub fn new(
        participant: DomainParticipantProxy<Rtps>,
        subscriber_impl: RtpsWeak<SubscriberAttributes<Rtps>>,
    ) -> Self {
        Self {
            participant,
            subscriber_impl,
        }
    }
}

impl<Rtps> AsRef<RtpsWeak<SubscriberAttributes<Rtps>>> for SubscriberProxy<Rtps>
where
    Rtps: RtpsStructure,
{
    fn as_ref(&self) -> &RtpsWeak<SubscriberAttributes<Rtps>> {
        &self.subscriber_impl
    }
}

impl<Foo, Rtps> SubscriberDataReaderFactory<Foo> for SubscriberProxy<Rtps>
where
    Foo: DdsType + for<'a> DdsDeserialize<'a> + Send + Sync + 'static,
    Rtps: RtpsStructure,
    Rtps::Group: RtpsEntityAttributes,
    Rtps::Participant: RtpsParticipantAttributes,
    Rtps::StatefulReader: RtpsStatefulReaderConstructor,
    Rtps::StatelessWriter: RtpsWriterOperations<DataType = Vec<u8>, ParameterListType = Vec<u8>>
        + RtpsWriterAttributes,
    Rtps::StatefulWriter: RtpsWriterOperations<DataType = Vec<u8>, ParameterListType = Vec<u8>>
        + RtpsWriterAttributes
        + RtpsStatefulWriterConstructor,
    <Rtps::StatelessWriter as RtpsWriterAttributes>::HistoryCacheType: RtpsHistoryCacheOperations<
        CacheChangeType = <Rtps::StatelessWriter as RtpsWriterOperations>::CacheChangeType,
    >,
    <Rtps::StatefulWriter as RtpsWriterAttributes>::HistoryCacheType: RtpsHistoryCacheOperations<
        CacheChangeType = <Rtps::StatefulWriter as RtpsWriterOperations>::CacheChangeType,
    >,
{
    type TopicType = TopicProxy<Foo, Rtps>;
    type DataReaderType = DataReaderProxy<Foo, Rtps>;

    fn datareader_factory_create_datareader(
        &self,
        topic: &Self::TopicType,
        qos: Option<DataReaderQos>,
        listener: Option<Box<dyn DataReaderListener + Send + Sync>>,
        _mask: StatusMask,
    ) -> DDSResult<Self::DataReaderType> {
        let subscriber_shared = self.subscriber_impl.upgrade()?;

        let topic_shared = topic.as_ref().upgrade()?;

        // /////// Build the GUID
        let entity_id = {
            let entity_kind = match Foo::has_key() {
                true => USER_DEFINED_WRITER_WITH_KEY,
                false => USER_DEFINED_WRITER_NO_KEY,
            };

            EntityId::new(
                [
                    subscriber_shared
                        .read_lock()
                        .rtps_group
                        .guid()
                        .entity_id()
                        .entity_key()[0],
                    subscriber_shared
                        .read_lock()
                        .user_defined_data_reader_counter,
                    0,
                ],
                entity_kind,
            )
        };

        let guid = Guid::new(
            subscriber_shared.read_lock().rtps_group.guid().prefix(),
            entity_id,
        );

        // /////// Create data reader
        let data_reader_shared = {
            let qos = qos.unwrap_or(
                subscriber_shared
                    .read_lock()
                    .default_data_reader_qos
                    .clone(),
            );
            qos.is_consistent()?;

            let topic_kind = match Foo::has_key() {
                true => TopicKind::WithKey,
                false => TopicKind::NoKey,
            };

            let reliability_level = match qos.reliability.kind {
                ReliabilityQosPolicyKind::BestEffortReliabilityQos => ReliabilityKind::BestEffort,
                ReliabilityQosPolicyKind::ReliableReliabilityQos => ReliabilityKind::Reliable,
            };

            let domain_participant = subscriber_shared
                .read_lock()
                .parent_domain_participant
                .upgrade()?;
            let rtps_reader = RtpsReader::Stateful(Rtps::StatefulReader::new(
                guid,
                topic_kind,
                reliability_level,
                &domain_participant
                    .read_lock()
                    .rtps_participant
                    .default_unicast_locator_list(),
                &domain_participant
                    .read_lock()
                    .rtps_participant
                    .default_multicast_locator_list(),
                rust_rtps_pim::behavior::types::DURATION_ZERO,
                rust_rtps_pim::behavior::types::DURATION_ZERO,
                false,
            ));

            let data_reader = DataReaderAttributes::new(
                qos,
                rtps_reader,
                topic_shared.clone(),
                listener,
                self.subscriber_impl.clone(),
            );

            let data_reader_shared = RtpsShared::new(data_reader);

            subscriber_shared
                .write_lock()
                .data_reader_list
                .push(data_reader_shared.clone());

            data_reader_shared
        };

        // /////// Announce the data reader creation
        {
            let domain_participant = subscriber_shared
                .read_lock()
                .parent_domain_participant
                .upgrade()?;
            let domain_participant_proxy =
                DomainParticipantProxy::new(domain_participant.downgrade());
            let builtin_publisher = domain_participant
                .read_lock()
                .builtin_publisher
                .clone()
                .ok_or(DDSError::PreconditionNotMet(
                    "No builtin publisher".to_string(),
                ))?;
            let builtin_publisher_proxy = PublisherProxy::new(builtin_publisher.downgrade());

            let subscription_topic = domain_participant_proxy
                .topic_factory_lookup_topicdescription(DCPS_SUBSCRIPTION)?;

            let sedp_builtin_subscription_announcer = builtin_publisher_proxy
                .datawriter_factory_lookup_datawriter(&subscription_topic)?;

            let sedp_discovered_reader_data = SedpDiscoveredReaderData {
                reader_proxy: RtpsReaderProxy {
                    remote_reader_guid: guid,
                    remote_group_entity_id: entity_id,
                    unicast_locator_list: domain_participant
                        .read_lock()
                        .rtps_participant
                        .default_unicast_locator_list()
                        .to_vec(),
                    multicast_locator_list: domain_participant
                        .read_lock()
                        .rtps_participant
                        .default_multicast_locator_list()
                        .to_vec(),
                    expects_inline_qos: false,
                },

                subscription_builtin_topic_data: SubscriptionBuiltinTopicData {
                    key: BuiltInTopicKey { value: guid.into() },
                    participant_key: BuiltInTopicKey { value: [1; 16] },
                    topic_name: topic_shared.read_lock().topic_name.clone(),
                    type_name: Foo::type_name().to_string(),
                    durability: DurabilityQosPolicy::default(),
                    deadline: DeadlineQosPolicy::default(),
                    latency_budget: LatencyBudgetQosPolicy::default(),
                    liveliness: LivelinessQosPolicy::default(),
                    reliability: ReliabilityQosPolicy {
                        kind: ReliabilityQosPolicyKind::BestEffortReliabilityQos,
                        max_blocking_time: Duration::new(3, 0),
                    },
                    ownership: OwnershipQosPolicy::default(),
                    destination_order: DestinationOrderQosPolicy::default(),
                    user_data: UserDataQosPolicy::default(),
                    time_based_filter: TimeBasedFilterQosPolicy::default(),
                    presentation: PresentationQosPolicy::default(),
                    partition: PartitionQosPolicy::default(),
                    topic_data: TopicDataQosPolicy::default(),
                    group_data: GroupDataQosPolicy::default(),
                },
            };

            sedp_builtin_subscription_announcer
                .write_w_timestamp(
                    &sedp_discovered_reader_data,
                    None,
                    rust_dds_api::dcps_psm::Time { sec: 0, nanosec: 0 },
                )
                .unwrap();
        }

        Ok(DataReaderProxy::new(data_reader_shared.downgrade()))
    }

    fn datareader_factory_delete_datareader(
        &self,
        datareader: &Self::DataReaderType,
    ) -> DDSResult<()> {
        let subscriber_shared = self.subscriber_impl.upgrade()?;
        let datareader_shared = datareader.as_ref().upgrade()?;

        let data_reader_list = &mut subscriber_shared.write_lock().data_reader_list;

        data_reader_list.remove(
            data_reader_list
                .iter()
                .position(|x| x == &datareader_shared)
                .ok_or(DDSError::PreconditionNotMet(
                    "Data reader can only be deleted from its parent subscriber".to_string(),
                ))?,
        );

        Ok(())
    }

    fn datareader_factory_lookup_datareader(
        &self,
        topic: &Self::TopicType,
    ) -> DDSResult<Self::DataReaderType> {
        let subscriber_shared = self.subscriber_impl.upgrade()?;
        let data_reader_list = &subscriber_shared.write_lock().data_reader_list;

        let topic_shared = topic.as_ref().upgrade()?;
        let topic = topic_shared.read_lock();

        data_reader_list
            .iter()
            .find_map(|data_reader_shared| {
                let data_reader_lock = data_reader_shared.read_lock();
                let data_reader_topic = data_reader_lock.topic.read_lock();

                if data_reader_topic.topic_name == topic.topic_name
                    && data_reader_topic.type_name == Foo::type_name()
                {
                    Some(DataReaderProxy::new(data_reader_shared.downgrade()))
                } else {
                    None
                }
            })
            .ok_or(DDSError::PreconditionNotMet("Not found".to_string()))
    }
}

impl<Rtps> Subscriber for SubscriberProxy<Rtps>
where
    Rtps: RtpsStructure,
{
    type DomainParticipant = DomainParticipantProxy<Rtps>;

    fn begin_access(&self) -> DDSResult<()> {
        todo!()
    }

    fn end_access(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_datareaders(
        &self,
        _readers: &mut [&mut dyn AnyDataReader],
        _sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
    ) -> DDSResult<()> {
        todo!()
    }

    fn notify_datareaders(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_sample_lost_status(&self, _status: &mut SampleLostStatus) -> DDSResult<()> {
        todo!()
    }

    fn delete_contained_entities(&self) -> DDSResult<()> {
        todo!()
    }

    fn set_default_datareader_qos(&self, _qos: Option<DataReaderQos>) -> DDSResult<()> {
        todo!()
    }

    fn get_default_datareader_qos(&self) -> DDSResult<DataReaderQos> {
        todo!()
    }

    fn copy_from_topic_qos(
        &self,
        _a_datareader_qos: &mut DataReaderQos,
        _a_topic_qos: &TopicQos,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_participant(&self) -> Self::DomainParticipant {
        self.participant.clone()
    }
}

impl<Rtps> Entity for SubscriberProxy<Rtps>
where
    Rtps: RtpsStructure,
{
    type Qos = SubscriberQos;
    type Listener = &'static dyn SubscriberListener;

    fn set_qos(&self, _qos: Option<Self::Qos>) -> DDSResult<()> {
        // rtps_shared_write_lock(&rtps_weak_upgrade(&self.subscriber_impl)?).set_qos(qos)
        todo!()
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.subscriber_impl)?).get_qos()
        todo!()
    }

    fn set_listener(
        &self,
        _a_listener: Option<Self::Listener>,
        _mask: StatusMask,
    ) -> DDSResult<()> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.subscriber_impl)?)
        // .set_listener(a_listener, mask)
        todo!()
    }

    fn get_listener(&self) -> DDSResult<Option<Self::Listener>> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.subscriber_impl)?).get_listener()
        todo!()
    }

    fn get_statuscondition(&self) -> DDSResult<StatusCondition> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.subscriber_impl)?).get_statuscondition()
        todo!()
    }

    fn get_status_changes(&self) -> DDSResult<StatusMask> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.subscriber_impl)?).get_status_changes()
        todo!()
    }

    fn enable(&self) -> DDSResult<()> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.subscriber_impl)?).enable()
        todo!()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.subscriber_impl)?).get_instance_handle()
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use rust_dds_api::{
        dcps_psm::{DomainId, InstanceHandle},
        infrastructure::qos::{
            DataReaderQos, DataWriterQos, DomainParticipantQos, PublisherQos, SubscriberQos,
            TopicQos,
        },
        return_type::{DDSError, DDSResult},
        subscription::subscriber::{Subscriber, SubscriberDataReaderFactory},
    };

    use rust_rtps_pim::{
        behavior::{
            reader::stateful_reader::RtpsStatefulReaderConstructor,
            types::Duration,
            writer::{
                stateful_writer::RtpsStatefulWriterConstructor,
                writer::{RtpsWriterAttributes, RtpsWriterOperations},
            },
        },
        discovery::sedp::builtin_endpoints::SedpBuiltinSubscriptionsWriter,
        structure::{
            entity::RtpsEntityAttributes,
            history_cache::RtpsHistoryCacheOperations,
            participant::{RtpsParticipantAttributes, RtpsParticipantConstructor},
            types::{
                ChangeKind, Guid, GuidPrefix, Locator, ReliabilityKind, SequenceNumber, TopicKind,
                GUID_UNKNOWN,
            },
        },
    };

    use crate::{
        data_representation_builtin_endpoints::sedp_discovered_reader_data::{
            SedpDiscoveredReaderData, DCPS_SUBSCRIPTION,
        },
        dds_impl::{
            data_writer_proxy::{DataWriterAttributes, RtpsWriter},
            domain_participant_proxy::{DomainParticipantAttributes, DomainParticipantProxy},
            publisher_proxy::PublisherAttributes,
            topic_proxy::{TopicAttributes, TopicProxy},
        },
        dds_type::{DdsDeserialize, DdsType},
        utils::{
            rtps_structure::RtpsStructure,
            shared_object::{RtpsShared, RtpsWeak},
        },
    };

    use super::{SubscriberAttributes, SubscriberProxy};

    #[derive(Default)]
    struct EmptyGroup {}
    impl RtpsEntityAttributes for EmptyGroup {
        fn guid(&self) -> Guid {
            GUID_UNKNOWN
        }
    }

    struct EmptyHistoryCache {}
    impl RtpsHistoryCacheOperations for EmptyHistoryCache {
        type CacheChangeType = ();
        fn add_change(&mut self, _change: ()) {}

        fn remove_change<F>(&mut self, _f: F)
        where
            F: FnMut(&Self::CacheChangeType) -> bool,
        {
            todo!()
        }
        fn get_seq_num_min(&self) -> Option<SequenceNumber> {
            None
        }

        fn get_seq_num_max(&self) -> Option<SequenceNumber> {
            None
        }
    }

    struct EmptyWriter {
        push_mode: bool,
        heartbeat_period: Duration,
        nack_response_delay: Duration,
        nack_suppression_duration: Duration,
        last_change_sequence_number: SequenceNumber,
        data_max_serialized: Option<i32>,
        writer_cache: EmptyHistoryCache,
    }
    impl RtpsWriterOperations for EmptyWriter {
        type DataType = Vec<u8>;
        type ParameterListType = Vec<u8>;
        type CacheChangeType = ();

        fn new_change(
            &mut self,
            _kind: ChangeKind,
            _data: Vec<u8>,
            _inline_qos: Vec<u8>,
            _handle: InstanceHandle,
        ) -> () {
            ()
        }
    }
    impl RtpsWriterAttributes for EmptyWriter {
        type HistoryCacheType = EmptyHistoryCache;

        fn push_mode(&self) -> bool {
            self.push_mode
        }
        fn heartbeat_period(&self) -> Duration {
            self.heartbeat_period
        }
        fn nack_response_delay(&self) -> Duration {
            self.nack_response_delay
        }
        fn nack_suppression_duration(&self) -> Duration {
            self.nack_suppression_duration
        }
        fn last_change_sequence_number(&self) -> SequenceNumber {
            self.last_change_sequence_number
        }
        fn data_max_size_serialized(&self) -> Option<i32> {
            self.data_max_serialized
        }
        fn writer_cache(&mut self) -> &mut EmptyHistoryCache {
            &mut self.writer_cache
        }
    }
    impl RtpsStatefulWriterConstructor for EmptyWriter {
        fn new(
            _guid: Guid,
            _topic_kind: TopicKind,
            _reliability_level: ReliabilityKind,
            _unicast_locator_list: &[Locator],
            _multicast_locator_list: &[Locator],
            _push_mode: bool,
            _heartbeat_period: Duration,
            _nack_response_delay: Duration,
            _nack_suppression_duration: Duration,
            _data_max_size_serialized: Option<i32>,
        ) -> Self {
            EmptyWriter {
                push_mode: false,
                heartbeat_period: Duration::new(0, 0),
                nack_response_delay: Duration::new(0, 0),
                nack_suppression_duration: Duration::new(0, 0),
                last_change_sequence_number: SequenceNumber::default(),
                data_max_serialized: None,
                writer_cache: EmptyHistoryCache {},
            }
        }
    }

    struct EmptyReader {}

    impl RtpsStatefulReaderConstructor for EmptyReader {
        fn new(
            _guid: Guid,
            _topic_kind: TopicKind,
            _reliability_level: ReliabilityKind,
            _unicast_locator_list: &[Locator],
            _multicast_locator_list: &[Locator],
            _heartbeat_response_delay: Duration,
            _heartbeat_suppression_duration: Duration,
            _expects_inline_qos: bool,
        ) -> Self {
            EmptyReader {}
        }
    }

    #[derive(Default)]
    struct EmptyParticipant {}

    impl RtpsParticipantConstructor for EmptyParticipant {
        fn new(
            _guid: Guid,
            _default_unicast_locator_list: &[Locator],
            _default_multicast_locator_list: &[Locator],
            _protocol_version: rust_rtps_pim::structure::types::ProtocolVersion,
            _vendor_id: rust_rtps_pim::structure::types::VendorId,
        ) -> Self {
            EmptyParticipant {}
        }
    }

    impl RtpsEntityAttributes for EmptyParticipant {
        fn guid(&self) -> Guid {
            todo!()
        }
    }

    impl RtpsParticipantAttributes for EmptyParticipant {
        fn protocol_version(&self) -> rust_rtps_pim::structure::types::ProtocolVersion {
            todo!()
        }

        fn vendor_id(&self) -> rust_rtps_pim::structure::types::VendorId {
            todo!()
        }

        fn default_unicast_locator_list(&self) -> &[Locator] {
            &[]
        }

        fn default_multicast_locator_list(&self) -> &[Locator] {
            &[]
        }
    }

    struct EmptyRtps {}

    impl RtpsStructure for EmptyRtps {
        type Group = EmptyGroup;
        type Participant = EmptyParticipant;
        type StatelessWriter = EmptyWriter;
        type StatefulWriter = EmptyWriter;
        type StatelessReader = ();
        type StatefulReader = EmptyReader;
    }

    fn make_participant<Rtps>() -> RtpsShared<DomainParticipantAttributes<Rtps>>
    where
        Rtps: RtpsStructure<StatefulWriter = EmptyWriter>,
        Rtps::Participant: Default + RtpsParticipantConstructor,
        Rtps::Group: Default,
    {
        let domain_participant = RtpsShared::new(DomainParticipantAttributes::new(
            GuidPrefix([0; 12]),
            DomainId::default(),
            "".to_string(),
            DomainParticipantQos::default(),
            vec![],
            vec![],
            vec![],
            vec![],
        ));

        domain_participant.write_lock().builtin_publisher =
            Some(RtpsShared::new(PublisherAttributes::new(
                PublisherQos::default(),
                Rtps::Group::default(),
                domain_participant.downgrade(),
            )));

        let sedp_topic_subscription = RtpsShared::new(TopicAttributes::<Rtps>::new(
            TopicQos::default(),
            SedpDiscoveredReaderData::type_name(),
            DCPS_SUBSCRIPTION,
            RtpsWeak::new(),
        ));

        domain_participant
            .write_lock()
            .topic_list
            .push(sedp_topic_subscription.clone());

        let sedp_builtin_subscriptions_rtps_writer =
            SedpBuiltinSubscriptionsWriter::create::<EmptyWriter>(GuidPrefix([0; 12]), &[], &[]);
        let sedp_builtin_subscriptions_data_writer = RtpsShared::new(DataWriterAttributes::new(
            DataWriterQos::default(),
            RtpsWriter::Stateful(sedp_builtin_subscriptions_rtps_writer),
            None,
            sedp_topic_subscription.clone(),
            domain_participant
                .read_lock()
                .builtin_publisher
                .as_ref()
                .unwrap()
                .downgrade(),
        ));
        domain_participant
            .read_lock()
            .builtin_publisher
            .as_ref()
            .unwrap()
            .write_lock()
            .data_writer_list
            .push(sedp_builtin_subscriptions_data_writer.clone());

        domain_participant
    }

    fn make_subscriber<Rtps: RtpsStructure>(
        parent: RtpsWeak<DomainParticipantAttributes<Rtps>>,
    ) -> RtpsShared<SubscriberAttributes<Rtps>>
    where
        Rtps::Group: Default,
    {
        RtpsShared::new(SubscriberAttributes {
            qos: SubscriberQos::default(),
            rtps_group: Rtps::Group::default(),
            data_reader_list: Vec::new(),
            user_defined_data_reader_counter: 0,
            default_data_reader_qos: DataReaderQos::default(),
            parent_domain_participant: parent,
        })
    }

    fn make_topic<Rtps: RtpsStructure>(
        type_name: &'static str,
        topic_name: &'static str,
    ) -> TopicAttributes<Rtps> {
        TopicAttributes::new(TopicQos::default(), type_name, topic_name, RtpsWeak::new())
    }

    macro_rules! make_empty_dds_type {
        ($type_name:ident) => {
            struct $type_name {}

            impl<'de> DdsDeserialize<'de> for $type_name {
                fn deserialize(_buf: &mut &'de [u8]) -> DDSResult<Self> {
                    Ok($type_name {})
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
    fn create_datareader() {
        let domain_participant = make_participant::<EmptyRtps>();

        let subscriber = make_subscriber::<EmptyRtps>(domain_participant.downgrade());
        let subscriber_proxy = SubscriberProxy::new(
            DomainParticipantProxy::new(domain_participant.downgrade()),
            subscriber.downgrade(),
        );

        let topic = RtpsShared::new(make_topic(Foo::type_name(), "topic"));
        let topic_proxy = TopicProxy::<Foo, EmptyRtps>::new(topic.downgrade());

        let data_reader = subscriber_proxy.create_datareader(&topic_proxy, None, None, 0);

        assert!(data_reader.is_ok());
    }

    #[test]
    fn datareader_factory_create_datareader() {
        let domain_participant = make_participant::<EmptyRtps>();

        let subscriber = make_subscriber::<EmptyRtps>(domain_participant.downgrade());
        let subscriber_proxy = SubscriberProxy::new(
            DomainParticipantProxy::new(domain_participant.downgrade()),
            subscriber.downgrade(),
        );

        let topic = RtpsShared::new(make_topic(Foo::type_name(), "topic"));
        let topic_proxy = TopicProxy::<Foo, EmptyRtps>::new(topic.downgrade());

        let data_reader =
            subscriber_proxy.datareader_factory_create_datareader(&topic_proxy, None, None, 0);

        assert!(data_reader.is_ok());
        assert_eq!(1, subscriber.read_lock().data_reader_list.len());
    }

    #[test]
    fn datareader_factory_delete_datareader() {
        let domain_participant = make_participant::<EmptyRtps>();

        let subscriber = make_subscriber::<EmptyRtps>(domain_participant.downgrade());
        let subscriber_proxy = SubscriberProxy::new(
            DomainParticipantProxy::new(domain_participant.downgrade()),
            subscriber.downgrade(),
        );

        let topic = RtpsShared::new(make_topic(Foo::type_name(), "topic"));
        let topic_proxy = TopicProxy::<Foo, EmptyRtps>::new(topic.downgrade());

        let data_reader = subscriber_proxy
            .datareader_factory_create_datareader(&topic_proxy, None, None, 0)
            .unwrap();

        assert_eq!(1, subscriber.read_lock().data_reader_list.len());

        subscriber_proxy
            .datareader_factory_delete_datareader(&data_reader)
            .unwrap();
        assert_eq!(0, subscriber.read_lock().data_reader_list.len());
        assert!(data_reader.as_ref().upgrade().is_err());
    }

    #[test]
    fn datareader_factory_delete_datareader_from_other_subscriber() {
        let domain_participant = make_participant::<EmptyRtps>();

        let subscriber = make_subscriber::<EmptyRtps>(domain_participant.downgrade());
        let subscriber_proxy = SubscriberProxy::new(
            DomainParticipantProxy::new(domain_participant.downgrade()),
            subscriber.downgrade(),
        );

        let subscriber2 = make_subscriber::<EmptyRtps>(domain_participant.downgrade());
        let subscriber2_proxy = SubscriberProxy::new(
            DomainParticipantProxy::new(domain_participant.downgrade()),
            subscriber2.downgrade(),
        );

        let topic = RtpsShared::new(make_topic(Foo::type_name(), "topic"));
        let topic_proxy = TopicProxy::<Foo, EmptyRtps>::new(topic.downgrade());

        let data_reader = subscriber_proxy
            .datareader_factory_create_datareader(&topic_proxy, None, None, 0)
            .unwrap();

        assert_eq!(1, subscriber.read_lock().data_reader_list.len());
        assert_eq!(0, subscriber2.read_lock().data_reader_list.len());

        assert!(matches!(
            subscriber2_proxy.datareader_factory_delete_datareader(&data_reader),
            Err(DDSError::PreconditionNotMet(_))
        ));
        assert!(data_reader.as_ref().upgrade().is_ok());
    }

    #[test]
    fn datareader_factory_lookup_datareader_when_empty() {
        let domain_participant = make_participant::<EmptyRtps>();

        let subscriber = make_subscriber::<EmptyRtps>(domain_participant.downgrade());
        let subscriber_proxy = SubscriberProxy::new(
            DomainParticipantProxy::new(domain_participant.downgrade()),
            subscriber.downgrade(),
        );

        let topic = RtpsShared::new(make_topic(Foo::type_name(), "topic"));
        let topic_proxy = TopicProxy::<Foo, EmptyRtps>::new(topic.downgrade());

        assert!(subscriber_proxy
            .datareader_factory_lookup_datareader(&topic_proxy)
            .is_err());
    }

    #[test]
    fn datareader_factory_lookup_datareader_when_one_datareader() {
        let domain_participant = make_participant::<EmptyRtps>();

        let subscriber = make_subscriber::<EmptyRtps>(domain_participant.downgrade());
        let subscriber_proxy = SubscriberProxy::new(
            DomainParticipantProxy::new(domain_participant.downgrade()),
            subscriber.downgrade(),
        );

        let topic = RtpsShared::new(make_topic(Foo::type_name(), "topic"));
        let topic_proxy = TopicProxy::<Foo, EmptyRtps>::new(topic.downgrade());

        let data_reader = subscriber_proxy
            .datareader_factory_create_datareader(&topic_proxy, None, None, 0)
            .unwrap();

        assert!(
            subscriber_proxy
                .datareader_factory_lookup_datareader(&topic_proxy)
                .unwrap()
                .as_ref()
                .upgrade()
                .unwrap()
                == data_reader.as_ref().upgrade().unwrap()
        );
    }

    make_empty_dds_type!(Bar);

    #[test]
    fn datareader_factory_lookup_datareader_when_one_datareader_with_wrong_type() {
        let domain_participant = make_participant::<EmptyRtps>();

        let subscriber = make_subscriber::<EmptyRtps>(domain_participant.downgrade());
        let subscriber_proxy = SubscriberProxy::new(
            DomainParticipantProxy::new(domain_participant.downgrade()),
            subscriber.downgrade(),
        );

        let topic_foo = RtpsShared::new(make_topic(Foo::type_name(), "topic"));
        let topic_foo_proxy = TopicProxy::<Foo, EmptyRtps>::new(topic_foo.downgrade());

        let topic_bar = RtpsShared::new(make_topic(Bar::type_name(), "topic"));
        let topic_bar_proxy = TopicProxy::<Bar, EmptyRtps>::new(topic_bar.downgrade());

        subscriber_proxy
            .datareader_factory_create_datareader(&topic_bar_proxy, None, None, 0)
            .unwrap();

        assert!(subscriber_proxy
            .datareader_factory_lookup_datareader(&topic_foo_proxy)
            .is_err());
    }

    #[test]
    fn datareader_factory_lookup_datareader_when_one_datareader_with_wrong_topic() {
        let domain_participant = make_participant::<EmptyRtps>();

        let subscriber = make_subscriber::<EmptyRtps>(domain_participant.downgrade());
        let subscriber_proxy = SubscriberProxy::new(
            DomainParticipantProxy::new(domain_participant.downgrade()),
            subscriber.downgrade(),
        );

        let topic1 = RtpsShared::new(make_topic(Foo::type_name(), "topic1"));
        let topic1_proxy = TopicProxy::<Foo, EmptyRtps>::new(topic1.downgrade());

        let topic2 = RtpsShared::new(make_topic(Foo::type_name(), "topic2"));
        let topic2_proxy = TopicProxy::<Foo, EmptyRtps>::new(topic2.downgrade());

        subscriber_proxy
            .datareader_factory_create_datareader(&topic2_proxy, None, None, 0)
            .unwrap();

        assert!(subscriber_proxy
            .datareader_factory_lookup_datareader(&topic1_proxy)
            .is_err());
    }

    #[test]
    fn datareader_factory_lookup_datareader_with_two_types() {
        let domain_participant = make_participant::<EmptyRtps>();

        let subscriber = make_subscriber::<EmptyRtps>(domain_participant.downgrade());
        let subscriber_proxy = SubscriberProxy::new(
            DomainParticipantProxy::new(domain_participant.downgrade()),
            subscriber.downgrade(),
        );

        let topic_foo = RtpsShared::new(make_topic(Foo::type_name(), "topic"));
        let topic_foo_proxy = TopicProxy::<Foo, EmptyRtps>::new(topic_foo.downgrade());

        let topic_bar = RtpsShared::new(make_topic(Bar::type_name(), "topic"));
        let topic_bar_proxy = TopicProxy::<Bar, EmptyRtps>::new(topic_bar.downgrade());

        let data_reader_foo = subscriber_proxy
            .datareader_factory_create_datareader(&topic_foo_proxy, None, None, 0)
            .unwrap();
        let data_reader_bar = subscriber_proxy
            .datareader_factory_create_datareader(&topic_bar_proxy, None, None, 0)
            .unwrap();

        assert!(
            subscriber_proxy
                .datareader_factory_lookup_datareader(&topic_foo_proxy)
                .unwrap()
                .as_ref()
                .upgrade()
                .unwrap()
                == data_reader_foo.as_ref().upgrade().unwrap()
        );

        assert!(
            subscriber_proxy
                .datareader_factory_lookup_datareader(&topic_bar_proxy)
                .unwrap()
                .as_ref()
                .upgrade()
                .unwrap()
                == data_reader_bar.as_ref().upgrade().unwrap()
        );
    }

    #[test]
    fn datareader_factory_lookup_datareader_with_two_topics() {
        let domain_participant = make_participant::<EmptyRtps>();

        let subscriber = make_subscriber::<EmptyRtps>(domain_participant.downgrade());
        let subscriber_proxy = SubscriberProxy::new(
            DomainParticipantProxy::new(domain_participant.downgrade()),
            subscriber.downgrade(),
        );

        let topic1 = RtpsShared::new(make_topic(Foo::type_name(), "topic1"));
        let topic1_proxy = TopicProxy::<Foo, EmptyRtps>::new(topic1.downgrade());

        let topic2 = RtpsShared::new(make_topic(Foo::type_name(), "topic2"));
        let topic2_proxy = TopicProxy::<Foo, EmptyRtps>::new(topic2.downgrade());

        let data_reader1 = subscriber_proxy
            .datareader_factory_create_datareader(&topic1_proxy, None, None, 0)
            .unwrap();
        let data_reader2 = subscriber_proxy
            .datareader_factory_create_datareader(&topic2_proxy, None, None, 0)
            .unwrap();

        assert!(
            subscriber_proxy
                .datareader_factory_lookup_datareader(&topic1_proxy)
                .unwrap()
                .as_ref()
                .upgrade()
                .unwrap()
                == data_reader1.as_ref().upgrade().unwrap()
        );

        assert!(
            subscriber_proxy
                .datareader_factory_lookup_datareader(&topic2_proxy)
                .unwrap()
                .as_ref()
                .upgrade()
                .unwrap()
                == data_reader2.as_ref().upgrade().unwrap()
        );
    }
}
