use rust_dds_api::{
    dcps_psm::{
        InstanceHandle, InstanceStateKind, SampleLostStatus, SampleStateKind, StatusMask,
        ViewStateKind,
    },
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::{DataReaderQos, SubscriberQos, TopicQos},
        qos_policy::ReliabilityQosPolicyKind,
    },
    return_type::{DDSError, DDSResult},
    subscription::{
        data_reader::AnyDataReader,
        data_reader_listener::DataReaderListener,
        subscriber::{Subscriber, SubscriberDataReaderFactory},
        subscriber_listener::SubscriberListener,
    },
};

use rust_rtps_pim::{
    behavior::reader::stateful_reader::RtpsStatefulReaderConstructor,
    structure::{
        entity::RtpsEntityAttributes,
        types::{
            EntityId, Guid, ReliabilityKind, TopicKind, USER_DEFINED_WRITER_NO_KEY,
            USER_DEFINED_WRITER_WITH_KEY,
        },
    },
};

use crate::{
    dds_type::{DdsDeserialize, DdsType},
    rtps_impl::rtps_group_impl::RtpsGroupImpl,
    utils::{
        rtps_structure::RtpsStructure,
        shared_object::{RtpsShared, RtpsWeak},
    },
};

use super::{
    data_reader_proxy::{DataReaderAttributes, DataReaderProxy, RtpsReader},
    domain_participant_proxy::{DomainParticipantAttributes, DomainParticipantProxy},
    topic_proxy::TopicProxy,
};

pub struct SubscriberAttributes<Rtps>
where
    Rtps: RtpsStructure,
{
    pub qos: SubscriberQos,
    pub rtps_group: RtpsGroupImpl,
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
        rtps_group: RtpsGroupImpl,
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
    Rtps::StatefulReader: RtpsStatefulReaderConstructor,
{
    type TopicType = TopicProxy<Foo, Rtps>;
    type DataReaderType = DataReaderProxy<Foo, Rtps>;

    fn datareader_factory_create_datareader(
        &self,
        a_topic: &Self::TopicType,
        qos: Option<DataReaderQos>,
        _a_listener: Option<&'static dyn DataReaderListener>,
        _mask: StatusMask,
    ) -> Option<Self::DataReaderType> {
        let subscriber_shared = self.subscriber_impl.upgrade().ok()?;
        let mut subscriber_shared_lock = subscriber_shared.write().ok()?;

        let qos = qos.unwrap_or(subscriber_shared_lock.default_data_reader_qos.clone());
        qos.is_consistent().ok()?;

        let (entity_kind, topic_kind) = match Foo::has_key() {
            true => (USER_DEFINED_WRITER_WITH_KEY, TopicKind::WithKey),
            false => (USER_DEFINED_WRITER_NO_KEY, TopicKind::NoKey),
        };
        let entity_id = EntityId::new(
            [
                subscriber_shared_lock
                    .rtps_group
                    .guid()
                    .entity_id()
                    .entity_key()[0],
                subscriber_shared_lock.user_defined_data_reader_counter,
                0,
            ],
            entity_kind,
        );
        let guid = Guid::new(
            *subscriber_shared_lock.rtps_group.guid().prefix(),
            entity_id,
        );
        let reliability_level = match qos.reliability.kind {
            ReliabilityQosPolicyKind::BestEffortReliabilityQos => ReliabilityKind::BestEffort,
            ReliabilityQosPolicyKind::ReliableReliabilityQos => ReliabilityKind::Reliable,
        };

        let heartbeat_response_delay = rust_rtps_pim::behavior::types::DURATION_ZERO;
        let heartbeat_supression_duration = rust_rtps_pim::behavior::types::DURATION_ZERO;
        let expects_inline_qos = false;
        let rtps_reader = RtpsReader::Stateful(Rtps::StatefulReader::new(
            guid,
            topic_kind,
            reliability_level,
            &[],
            &[],
            heartbeat_response_delay,
            heartbeat_supression_duration,
            expects_inline_qos,
        ));
        let reader_storage = DataReaderAttributes {
            rtps_reader,
            _qos: qos,
            topic: a_topic.as_ref().upgrade().ok()?,
            _listener: None,
            parent_subscriber: self.subscriber_impl.clone(),
        };

        let reader_storage_shared = RtpsShared::new(reader_storage);

        subscriber_shared_lock
            .data_reader_list
            .push(reader_storage_shared.clone());

        Some(DataReaderProxy::new(reader_storage_shared.downgrade()))
    }

    fn datareader_factory_delete_datareader(
        &self,
        datareader: &Self::DataReaderType,
    ) -> DDSResult<()> {
        let subscriber_shared = self.subscriber_impl.upgrade()?;
        let datareader_shared = datareader.as_ref().upgrade()?;

        let data_reader_list = &mut subscriber_shared
            .write()
            .map_err(|_| DDSError::Error)?
            .data_reader_list;

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
    ) -> Option<Self::DataReaderType> {
        let subscriber_shared = self.subscriber_impl.upgrade().ok()?;
        let data_reader_list = &subscriber_shared.write().ok()?.data_reader_list;

        let topic_shared = topic.as_ref().upgrade().ok()?;
        let topic = topic_shared.read().ok()?;

        data_reader_list.iter().find_map(|data_reader| {
            data_reader
                .read()
                .ok()?
                .topic
                .read()
                .ok()
                .filter(|data_reader_topic| data_reader_topic.type_name == Foo::type_name())
                .filter(|data_reader_topic| data_reader_topic.topic_name == topic.topic_name)
                .and(Some(DataReaderProxy::new(data_reader.downgrade())))
        })
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

    fn set_qos(&mut self, _qos: Option<Self::Qos>) -> DDSResult<()> {
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
        dcps_psm::DomainId,
        infrastructure::qos::{
            DataReaderQos, DomainParticipantQos, PublisherQos, SubscriberQos, TopicQos,
        },
        return_type::{DDSError, DDSResult},
        subscription::subscriber::{Subscriber, SubscriberDataReaderFactory},
    };

    use rust_rtps_pim::{
        behavior::{reader::stateful_reader::RtpsStatefulReaderConstructor, types::Duration},
        messages::types::Count,
        structure::types::{
            Guid, Locator, ProtocolVersion, ReliabilityKind, TopicKind, VendorId, GUID_UNKNOWN,
        },
    };

    use crate::{
        dds_impl::{
            domain_participant_proxy::{DomainParticipantAttributes, DomainParticipantProxy},
            topic_proxy::{TopicAttributes, TopicProxy},
        },
        dds_type::{DdsDeserialize, DdsType},
        rtps_impl::{rtps_group_impl::RtpsGroupImpl, rtps_participant_impl::RtpsParticipantImpl},
        utils::{
            rtps_structure::RtpsStructure,
            shared_object::{RtpsShared, RtpsWeak},
        },
    };

    use super::{SubscriberAttributes, SubscriberProxy};

    struct EmptyReader {}

    impl RtpsStatefulReaderConstructor for EmptyReader {
        fn new(
            _guid: Guid,
            _topic_kind: TopicKind,
            _reliability_level: ReliabilityKind,
            _unicast_locator_list: &[Locator],
            _multicast_locator_list: &[Locator],
            _heartbeat_response_delay: Duration,
            _heartbeat_supression_duration: Duration,
            _expects_inline_qos: bool,
        ) -> Self {
            EmptyReader {}
        }
    }

    struct EmptyRtps {}

    impl RtpsStructure for EmptyRtps {
        type StatelessWriter = ();
        type StatefulWriter = ();
        type StatelessReader = ();
        type StatefulReader = EmptyReader;
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

    impl<Rtps: RtpsStructure> Default for DomainParticipantAttributes<Rtps> {
        fn default() -> Self {
            DomainParticipantAttributes {
                rtps_participant: RtpsParticipantImpl::new(
                    GUID_UNKNOWN,
                    ProtocolVersion { major: 0, minor: 0 },
                    VendorId::default(),
                    vec![],
                    vec![],
                ),
                domain_id: DomainId::default(),
                domain_tag: "".to_string(),
                qos: DomainParticipantQos::default(),
                builtin_subscriber_list: vec![],
                builtin_publisher_list: vec![],
                user_defined_subscriber_list: vec![],
                user_defined_subscriber_counter: 0,
                default_subscriber_qos: SubscriberQos::default(),
                user_defined_publisher_list: vec![],
                user_defined_publisher_counter: 0,
                default_publisher_qos: PublisherQos::default(),
                topic_list: vec![],
                default_topic_qos: TopicQos::default(),
                manual_liveliness_count: Count(0),
                lease_duration: Duration::new(0, 0),
                metatraffic_unicast_locator_list: vec![],
                metatraffic_multicast_locator_list: vec![],
                enabled: false,
            }
        }
    }

    impl<Rtps: RtpsStructure> Default for SubscriberAttributes<Rtps> {
        fn default() -> Self {
            SubscriberAttributes {
                qos: SubscriberQos::default(),
                rtps_group: RtpsGroupImpl::new(GUID_UNKNOWN),
                data_reader_list: Vec::new(),
                user_defined_data_reader_counter: 0,
                default_data_reader_qos: DataReaderQos::default(),
                parent_domain_participant: RtpsWeak::new(),
            }
        }
    }

    fn make_topic<Rtps: RtpsStructure>(
        type_name: &'static str,
        topic_name: &'static str,
    ) -> TopicAttributes<Rtps> {
        TopicAttributes::new(TopicQos::default(), type_name, topic_name, RtpsWeak::new())
    }

    #[test]
    fn create_datareader() {
        let participant = RtpsShared::new(DomainParticipantAttributes::default());
        let participant_proxy = DomainParticipantProxy::new(participant.downgrade());

        let subscriber = RtpsShared::new(SubscriberAttributes::default());
        let subscriber_proxy = SubscriberProxy::new(participant_proxy, subscriber.downgrade());

        let topic = RtpsShared::new(make_topic(Foo::type_name(), "topic"));
        let topic_proxy = TopicProxy::<Foo, EmptyRtps>::new(topic.downgrade());

        let data_reader = subscriber_proxy.create_datareader(&topic_proxy, None, None, 0);

        assert!(data_reader.is_some());
    }

    #[test]
    fn datareader_factory_create_datareader() {
        let participant = RtpsShared::new(DomainParticipantAttributes::default());
        let participant_proxy = DomainParticipantProxy::new(participant.downgrade());

        let subscriber = RtpsShared::new(SubscriberAttributes::default());
        let subscriber_proxy = SubscriberProxy::new(participant_proxy, subscriber.downgrade());

        let topic = RtpsShared::new(make_topic(Foo::type_name(), "topic"));
        let topic_proxy = TopicProxy::<Foo, EmptyRtps>::new(topic.downgrade());

        let data_reader =
            subscriber_proxy.datareader_factory_create_datareader(&topic_proxy, None, None, 0);

        assert!(data_reader.is_some());
        assert_eq!(1, subscriber.read().unwrap().data_reader_list.len());
    }

    #[test]
    fn datareader_factory_delete_datareader() {
        let participant = RtpsShared::new(DomainParticipantAttributes::default());
        let participant_proxy = DomainParticipantProxy::new(participant.downgrade());

        let subscriber = RtpsShared::new(SubscriberAttributes::default());
        let subscriber_proxy = SubscriberProxy::new(participant_proxy, subscriber.downgrade());

        let topic = RtpsShared::new(make_topic(Foo::type_name(), "topic"));
        let topic_proxy = TopicProxy::<Foo, EmptyRtps>::new(topic.downgrade());

        let data_reader = subscriber_proxy
            .datareader_factory_create_datareader(&topic_proxy, None, None, 0)
            .unwrap();

        assert_eq!(1, subscriber.read().unwrap().data_reader_list.len());

        subscriber_proxy
            .datareader_factory_delete_datareader(&data_reader)
            .unwrap();
        assert_eq!(0, subscriber.read().unwrap().data_reader_list.len());
    }

    #[test]
    fn datareader_factory_delete_datareader_from_other_subscriber() {
        let participant = RtpsShared::new(DomainParticipantAttributes::default());
        let participant_proxy = DomainParticipantProxy::new(participant.downgrade());

        let subscriber = RtpsShared::new(SubscriberAttributes::default());
        let subscriber_proxy =
            SubscriberProxy::new(participant_proxy.clone(), subscriber.downgrade());

        let subscriber2 = RtpsShared::new(SubscriberAttributes::default());
        let subscriber2_proxy =
            SubscriberProxy::new(participant_proxy.clone(), subscriber2.downgrade());

        let topic = RtpsShared::new(make_topic(Foo::type_name(), "topic"));
        let topic_proxy = TopicProxy::<Foo, EmptyRtps>::new(topic.downgrade());

        let data_reader = subscriber_proxy
            .datareader_factory_create_datareader(&topic_proxy, None, None, 0)
            .unwrap();

        assert_eq!(1, subscriber.read().unwrap().data_reader_list.len());
        assert_eq!(0, subscriber2.read().unwrap().data_reader_list.len());

        assert!(matches!(
            subscriber2_proxy.datareader_factory_delete_datareader(&data_reader),
            Err(DDSError::PreconditionNotMet(_))
        ));
    }

    #[test]
    fn datareader_factory_lookup_datareader_when_empty() {
        let participant = RtpsShared::new(DomainParticipantAttributes::default());
        let participant_proxy = DomainParticipantProxy::new(participant.downgrade());

        let subscriber = RtpsShared::new(SubscriberAttributes::default());
        let subscriber_proxy = SubscriberProxy::new(participant_proxy, subscriber.downgrade());

        let topic = RtpsShared::new(make_topic(Foo::type_name(), "topic"));
        let topic_proxy = TopicProxy::<Foo, EmptyRtps>::new(topic.downgrade());

        assert!(subscriber_proxy
            .datareader_factory_lookup_datareader(&topic_proxy)
            .is_none());
    }

    #[test]
    fn datareader_factory_lookup_datareader_when_one_datareader() {
        let participant = RtpsShared::new(DomainParticipantAttributes::default());
        let participant_proxy = DomainParticipantProxy::new(participant.downgrade());

        let subscriber = RtpsShared::new(SubscriberAttributes::default());
        let subscriber_proxy = SubscriberProxy::new(participant_proxy, subscriber.downgrade());

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
        let participant = RtpsShared::new(DomainParticipantAttributes::default());
        let participant_proxy = DomainParticipantProxy::new(participant.downgrade());

        let subscriber = RtpsShared::new(SubscriberAttributes::default());
        let subscriber_proxy = SubscriberProxy::new(participant_proxy, subscriber.downgrade());

        let topic_foo = RtpsShared::new(make_topic(Foo::type_name(), "topic"));
        let topic_foo_proxy = TopicProxy::<Foo, EmptyRtps>::new(topic_foo.downgrade());

        let topic_bar = RtpsShared::new(make_topic(Bar::type_name(), "topic"));
        let topic_bar_proxy = TopicProxy::<Bar, EmptyRtps>::new(topic_bar.downgrade());

        subscriber_proxy
            .datareader_factory_create_datareader(&topic_bar_proxy, None, None, 0)
            .unwrap();

        assert!(subscriber_proxy
            .datareader_factory_lookup_datareader(&topic_foo_proxy)
            .is_none());
    }

    #[test]
    fn datareader_factory_lookup_datareader_when_one_datareader_with_wrong_topic() {
        let participant = RtpsShared::new(DomainParticipantAttributes::default());
        let participant_proxy = DomainParticipantProxy::new(participant.downgrade());

        let subscriber = RtpsShared::new(SubscriberAttributes::default());
        let subscriber_proxy = SubscriberProxy::new(participant_proxy, subscriber.downgrade());

        let topic1 = RtpsShared::new(make_topic(Foo::type_name(), "topic1"));
        let topic1_proxy = TopicProxy::<Foo, EmptyRtps>::new(topic1.downgrade());

        let topic2 = RtpsShared::new(make_topic(Foo::type_name(), "topic2"));
        let topic2_proxy = TopicProxy::<Foo, EmptyRtps>::new(topic2.downgrade());

        subscriber_proxy
            .datareader_factory_create_datareader(&topic2_proxy, None, None, 0)
            .unwrap();

        assert!(subscriber_proxy
            .datareader_factory_lookup_datareader(&topic1_proxy)
            .is_none());
    }

    #[test]
    fn datareader_factory_lookup_datareader_with_two_types() {
        let participant = RtpsShared::new(DomainParticipantAttributes::default());
        let participant_proxy = DomainParticipantProxy::new(participant.downgrade());

        let subscriber = RtpsShared::new(SubscriberAttributes::default());
        let subscriber_proxy = SubscriberProxy::new(participant_proxy, subscriber.downgrade());

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
        let participant = RtpsShared::new(DomainParticipantAttributes::default());
        let participant_proxy = DomainParticipantProxy::new(participant.downgrade());

        let subscriber = RtpsShared::new(SubscriberAttributes::default());
        let subscriber_proxy = SubscriberProxy::new(participant_proxy, subscriber.downgrade());

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
