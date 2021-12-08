use std::sync::{
    atomic::{self, AtomicU8},
    Arc, Mutex, RwLock,
};

use rust_dds_api::{
    builtin_topics::{ParticipantBuiltinTopicData, TopicBuiltinTopicData},
    dcps_psm::{BuiltInTopicKey, DomainId, Duration, InstanceHandle, StatusMask, Time},
    domain::{
        domain_participant::{
            DomainParticipant, DomainParticipantPublisherFactory,
            DomainParticipantSubscriberFactory, DomainParticipantTopicFactory,
        },
        domain_participant_listener::DomainParticipantListener,
    },
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::{DomainParticipantQos, PublisherQos, SubscriberQos, TopicQos},
    },
    publication::{publisher::Publisher, publisher_listener::PublisherListener},
    return_type::{DDSError, DDSResult},
    subscription::{
        data_reader::DataReader, subscriber::Subscriber, subscriber_listener::SubscriberListener,
    },
    topic::{topic_description::TopicDescription, topic_listener::TopicListener},
};
use rust_rtps_pim::{
    discovery::{
        participant_discovery::ParticipantDiscovery,
        spdp::participant_proxy::ParticipantProxy,
        types::{BuiltinEndpointQos, BuiltinEndpointSet},
    },
    messages::types::Count,
    structure::{
        entity::RtpsEntity,
        group::RtpsGroup,
        participant::RtpsParticipant,
        types::{
            EntityId, Guid, GuidPrefix, Locator, BUILT_IN_READER_GROUP, BUILT_IN_WRITER_GROUP,
            ENTITYID_PARTICIPANT, PROTOCOLVERSION, USER_DEFINED_WRITER_GROUP, VENDOR_ID_S2E,
        },
    },
};

use crate::{
    data_representation_builtin_endpoints::{
        sedp_discovered_reader_data::SedpDiscoveredReaderData,
        sedp_discovered_topic_data::SedpDiscoveredTopicData,
        sedp_discovered_writer_data::SedpDiscoveredWriterData,
        spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
    },
    rtps_impl::{
        rtps_stateful_writer_impl::RtpsStatefulWriterImpl,
        rtps_stateless_writer_impl::RtpsStatelessWriterImpl,
    },
    utils::{
        clock::StdTimer,
        communication::Communication,
        shared_object::{rtps_shared_downgrade, rtps_shared_new, rtps_weak_upgrade, RtpsShared},
        tasks::Spawner,
        transport::{TransportRead, TransportWrite},
    },
};

use super::{
    data_reader_impl::DataReaderImpl,
    data_writer_impl::DataWriterImpl,
    publisher_impl::{AnyStatefulDataWriter, AnyStatelessDataWriter, PublisherImpl},
    publisher_proxy::PublisherProxy,
    subscriber_impl::{DataReaderObject, SubscriberImpl},
    subscriber_proxy::SubscriberProxy,
    topic_impl::TopicImpl,
    topic_proxy::TopicProxy,
};

pub struct DomainParticipantImpl {
    rtps_participant: RtpsParticipant<Vec<Locator>>,
    domain_id: DomainId,
    domain_tag: String,
    qos: DomainParticipantQos,
    builtin_subscriber: RtpsShared<SubscriberImpl>,
    builtin_publisher: RtpsShared<PublisherImpl>,
    _user_defined_subscriber_list: Arc<Mutex<Vec<RtpsShared<SubscriberImpl>>>>,
    _user_defined_subscriber_counter: u8,
    default_subscriber_qos: SubscriberQos,
    user_defined_publisher_list: Arc<Mutex<Vec<RtpsShared<PublisherImpl>>>>,
    user_defined_publisher_counter: AtomicU8,
    default_publisher_qos: PublisherQos,
    _topic_list: Vec<RtpsShared<TopicImpl>>,
    default_topic_qos: TopicQos,
    manual_liveliness_count: Count,
    lease_duration: rust_rtps_pim::behavior::types::Duration,
    metatraffic_unicast_locator_list: Vec<Locator>,
    metatraffic_multicast_locator_list: Vec<Locator>,
    default_unicast_locator_list: Vec<Locator>,
    default_multicast_locator_list: Vec<Locator>,
    spawner: Spawner,
}

impl DomainParticipantImpl {
    pub fn new(
        guid_prefix: GuidPrefix,
        domain_id: DomainId,
        domain_tag: String,
        domain_participant_qos: DomainParticipantQos,
        metatraffic_transport: impl TransportRead + TransportWrite + Send + Sync + 'static,
        default_transport: impl TransportRead + TransportWrite + Send + Sync + 'static,
        metatraffic_unicast_locator_list: Vec<Locator>,
        metatraffic_multicast_locator_list: Vec<Locator>,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
        mut spdp_builtin_participant_data_reader: Option<DataReaderImpl<SpdpDiscoveredParticipantData>>,
        spdp_builtin_participant_data_writer: Option<
            DataWriterImpl<SpdpDiscoveredParticipantData, RtpsStatelessWriterImpl, StdTimer>,
        >,
        sedp_builtin_publications_data_reader: Option<DataReaderImpl<SedpDiscoveredWriterData>>,
        mut sedp_builtin_publications_data_writer: Option<
            DataWriterImpl<SedpDiscoveredWriterData, RtpsStatefulWriterImpl, StdTimer>,
        >,
        sedp_builtin_subscriptions_data_reader: Option<DataReaderImpl<SedpDiscoveredReaderData>>,
        sedp_builtin_subscriptions_data_writer: Option<
            DataWriterImpl<SedpDiscoveredReaderData, RtpsStatefulWriterImpl, StdTimer>,
        >,
        sedp_builtin_topics_data_reader: Option<DataReaderImpl<SedpDiscoveredTopicData>>,
        sedp_builtin_topics_data_writer: Option<
            DataWriterImpl<SedpDiscoveredTopicData, RtpsStatefulWriterImpl, StdTimer>,
        >,
        spawner: Spawner,
    ) -> Self {
        let lease_duration = rust_rtps_pim::behavior::types::Duration::new(100, 0);
        let protocol_version = PROTOCOLVERSION;
        let vendor_id = VENDOR_ID_S2E;
        let rtps_participant = RtpsParticipant {
            entity: RtpsEntity {
                guid: Guid::new(guid_prefix, ENTITYID_PARTICIPANT),
            },
            protocol_version,
            vendor_id,
            default_unicast_locator_list: vec![],
            default_multicast_locator_list: vec![],
        };

        {
            if let Some(spdp_builtin_participant_reader) = &mut spdp_builtin_participant_data_reader
            {
                let samples = spdp_builtin_participant_reader
                    .read(1, &[], &[], &[])
                    .unwrap_or(vec![]);
                for discovered_participant in samples {
                    if let Ok(participant_discovery) = ParticipantDiscovery::new(
                        &discovered_participant.participant_proxy,
                        domain_id as u32,
                        domain_tag.as_ref(),
                    ) {
                        if let Some(sedp_builtin_publications_writer) =
                            &mut sedp_builtin_publications_data_writer
                        {
                            participant_discovery.discovered_participant_add_publications_writer(
                                sedp_builtin_publications_writer.as_mut(),
                            );
                        }
                    }
                }
            }
        }

        let mut data_reader_list: Vec<Arc<dyn DataReaderObject + Send + Sync>> = Vec::new();
        if let Some(spdp_builtin_participant_data_reader) = spdp_builtin_participant_data_reader {
            data_reader_list.push(Arc::new(RwLock::new(spdp_builtin_participant_data_reader)));
        }
        if let Some(sedp_builtin_publications_data_reader) = sedp_builtin_publications_data_reader {
            data_reader_list.push(Arc::new(RwLock::new(sedp_builtin_publications_data_reader)));
        }
        if let Some(sedp_builtin_subscriptions_data_reader) = sedp_builtin_subscriptions_data_reader
        {
            data_reader_list.push(Arc::new(RwLock::new(
                sedp_builtin_subscriptions_data_reader,
            )));
        }
        if let Some(sedp_builtin_topics_data_reader) = sedp_builtin_topics_data_reader {
            data_reader_list.push(Arc::new(RwLock::new(sedp_builtin_topics_data_reader)));
        }
        let builtin_subscriber = rtps_shared_new(SubscriberImpl::new(
            SubscriberQos::default(),
            RtpsGroup::new(Guid::new(
                guid_prefix,
                EntityId::new([0, 0, 0], BUILT_IN_READER_GROUP),
            )),
            data_reader_list,
        ));

        let mut stateless_data_writer_list: Vec<Arc<dyn AnyStatelessDataWriter + Send + Sync>> =
            Vec::new();
        let mut stateful_data_writer_list: Vec<Arc<dyn AnyStatefulDataWriter + Send + Sync>> =
            Vec::new();
        if let Some(spdp_builtin_participant_data_writer) = spdp_builtin_participant_data_writer {
            stateless_data_writer_list
                .push(Arc::new(RwLock::new(spdp_builtin_participant_data_writer)));
        };
        if let Some(sedp_builtin_publications_data_writer) = sedp_builtin_publications_data_writer {
            stateful_data_writer_list
                .push(Arc::new(RwLock::new(sedp_builtin_publications_data_writer)))
        }
        if let Some(sedp_builtin_subscriptions_data_writer) = sedp_builtin_subscriptions_data_writer
        {
            stateful_data_writer_list.push(Arc::new(RwLock::new(
                sedp_builtin_subscriptions_data_writer,
            )))
        }
        if let Some(sedp_builtin_topics_data_writer) = sedp_builtin_topics_data_writer {
            stateful_data_writer_list.push(Arc::new(RwLock::new(sedp_builtin_topics_data_writer)))
        }
        let builtin_publisher = rtps_shared_new(PublisherImpl::new(
            PublisherQos::default(),
            RtpsGroup::new(Guid::new(
                guid_prefix,
                EntityId::new([0, 0, 0], BUILT_IN_WRITER_GROUP),
            )),
            stateless_data_writer_list,
            stateful_data_writer_list,
        ));

        let builtin_subscriber_arc = builtin_subscriber.clone();
        let builtin_publisher_arc = builtin_publisher.clone();
        let user_defined_subscriber_list = Arc::new(Mutex::new(Vec::new()));
        let user_defined_subscriber_list_arc = user_defined_subscriber_list.clone();
        let user_defined_publisher_list = Arc::new(Mutex::new(Vec::new()));
        let user_defined_publisher_list_arc = user_defined_publisher_list.clone();

        let mut communication = Communication {
            version: protocol_version,
            vendor_id,
            guid_prefix,
            transport: metatraffic_transport,
        };
        spawner.spawn_enabled_periodic_task(
            "builtin communication",
            move || {
                communication.send(core::slice::from_ref(&builtin_publisher_arc));
                communication.receive(core::slice::from_ref(&builtin_subscriber_arc));
            },
            std::time::Duration::from_millis(100),
        );
        let mut communication = Communication {
            version: protocol_version,
            vendor_id,
            guid_prefix,
            transport: default_transport,
        };
        spawner.spawn_enabled_periodic_task(
            "user-defined communication",
            move || {
                communication.send(&user_defined_publisher_list_arc.lock().unwrap());
                communication.receive(&user_defined_subscriber_list_arc.lock().unwrap());
            },
            std::time::Duration::from_millis(100),
        );

        Self {
            rtps_participant,
            domain_id,
            domain_tag,
            qos: domain_participant_qos,
            builtin_subscriber,
            builtin_publisher,
            _user_defined_subscriber_list: user_defined_subscriber_list,
            _user_defined_subscriber_counter: 0,
            default_subscriber_qos: SubscriberQos::default(),
            user_defined_publisher_list,
            user_defined_publisher_counter: AtomicU8::new(0),
            default_publisher_qos: PublisherQos::default(),
            _topic_list: Vec::new(),
            default_topic_qos: TopicQos::default(),
            manual_liveliness_count: Count(0),
            lease_duration,
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
            default_unicast_locator_list,
            default_multicast_locator_list,
            spawner,
        }
    }

    fn as_spdp_discovered_participant_data(&self) -> SpdpDiscoveredParticipantData {
        SpdpDiscoveredParticipantData {
            dds_participant_data: ParticipantBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: self.rtps_participant.entity.guid.into(),
                },
                user_data: self.qos.user_data.clone(),
            },
            participant_proxy: ParticipantProxy {
                domain_id: self.domain_id as u32,
                domain_tag: self.domain_tag.clone(),
                protocol_version: self.rtps_participant.protocol_version,
                guid_prefix: self.rtps_participant.entity.guid.prefix,
                vendor_id: self.rtps_participant.vendor_id,
                expects_inline_qos: false,
                metatraffic_unicast_locator_list: self.metatraffic_unicast_locator_list.clone(),
                metatraffic_multicast_locator_list: self.metatraffic_multicast_locator_list.clone(),
                default_unicast_locator_list: self.default_unicast_locator_list.clone(),
                default_multicast_locator_list: self.default_multicast_locator_list.clone(),
                available_builtin_endpoints: BuiltinEndpointSet::default(),
                manual_liveliness_count: self.manual_liveliness_count,
                builtin_endpoint_qos: BuiltinEndpointQos::default(),
            },
            lease_duration: self.lease_duration,
        }
    }
}

impl<'p> DomainParticipantPublisherFactory<'p> for DomainParticipantImpl {
    type PublisherType = PublisherProxy<'p, PublisherImpl>;

    fn publisher_factory_create_publisher(
        &'p self,
        qos: Option<PublisherQos>,
        _a_listener: Option<&'static dyn PublisherListener>,
        _mask: StatusMask,
    ) -> Option<Self::PublisherType> {
        let publisher_qos = qos.unwrap_or(self.default_publisher_qos.clone());
        let user_defined_publisher_counter = self
            .user_defined_publisher_counter
            .fetch_add(1, atomic::Ordering::SeqCst);
        let entity_id = EntityId::new(
            [user_defined_publisher_counter, 0, 0],
            USER_DEFINED_WRITER_GROUP,
        );
        let guid = Guid::new(self.rtps_participant.entity.guid.prefix, entity_id);
        let rtps_group = RtpsGroup::new(guid);
        let data_writer_impl_list = Vec::new();
        let publisher_impl =
            PublisherImpl::new(publisher_qos, rtps_group, data_writer_impl_list, vec![]);
        let publisher_impl_shared = rtps_shared_new(publisher_impl);
        let publisher_impl_weak = rtps_shared_downgrade(&publisher_impl_shared);
        self.user_defined_publisher_list
            .lock()
            .unwrap()
            .push(publisher_impl_shared);
        let publisher = PublisherProxy::new(self, publisher_impl_weak);

        Some(publisher)
    }

    fn publisher_factory_delete_publisher(
        &self,
        a_publisher: &Self::PublisherType,
    ) -> DDSResult<()> {
        // let publisher = a_publisher.upgrade()?;

        if std::ptr::eq(a_publisher.get_participant(), self) {
            let publisher_impl_shared = rtps_weak_upgrade(&a_publisher.publisher_impl())?;
            self.user_defined_publisher_list
                .lock()
                .unwrap()
                .retain(|x| !Arc::ptr_eq(&x, &publisher_impl_shared));
            Ok(())
        } else {
            Err(DDSError::PreconditionNotMet(
                "Publisher can only be deleted from its parent participant".to_string(),
            ))
        }
    }
}

impl<'s> DomainParticipantSubscriberFactory<'s> for DomainParticipantImpl {
    type SubscriberType = SubscriberProxy<'s, SubscriberImpl>;

    fn subscriber_factory_create_subscriber(
        &'s self,
        _qos: Option<SubscriberQos>,
        _a_listener: Option<&'static dyn SubscriberListener>,
        _mask: StatusMask,
    ) -> Option<Self::SubscriberType> {
        // let subscriber_qos = qos.unwrap_or(self.default_subscriber_qos.clone());
        // self.user_defined_subscriber_counter += 1;
        // let entity_id = EntityId::new(
        //     [self.user_defined_subscriber_counter, 0, 0],
        //     USER_DEFINED_WRITER_GROUP,
        // );
        // let guid = Guid::new(*self.rtps_participant.guid().prefix(), entity_id);
        // let rtps_group = RtpsGroupImpl::new(guid);
        // let data_reader_storage_list = Vec::new();
        // let subscriber_storage =
        //     SubscriberImpl::new(subscriber_qos, rtps_group, data_reader_storage_list);
        // let subscriber_storage_shared = RtpsShared::new(subscriber_storage);
        // let subscriber_storage_weak = subscriber_storage_shared.downgrade();
        // self.user_defined_subscriber_storage
        //     .push(subscriber_storage_shared);
        // Some(subscriber_storage_weak)

        // let subscriber_storage_weak = self
        //     .domain_participant_storage
        //     .lock()
        //     .create_subscriber(qos, a_listener, mask)?;
        // let subscriber = SubscriberProxy::new(self, subscriber_storage_weak);
        // Some(subscriber)
        todo!()
    }

    fn subscriber_factory_delete_subscriber(
        &self,
        _a_subscriber: &Self::SubscriberType,
    ) -> DDSResult<()> {
        // let subscriber_storage = a_subscriber.upgrade()?;
        // self.user_defined_subscriber_storage
        //     .retain(|x| x != &subscriber_storage);
        // Ok(())

        // if std::ptr::eq(a_subscriber.get_participant(), self) {
        //     self.domain_participant_storage
        //         .lock()
        //         .delete_subscriber(a_subscriber.subscriber_storage())
        // } else {
        //     Err(DDSError::PreconditionNotMet(
        //         "Subscriber can only be deleted from its parent participant",
        //     ))
        // }
        todo!()
    }

    fn subscriber_factory_get_builtin_subscriber(&'s self) -> Self::SubscriberType {
        // self.builtin_subscriber_storage[0].clone().downgrade()

        // let subscriber_storage_weak = self
        //     .domain_participant_storage
        //     .lock()
        //     .get_builtin_subscriber();
        // SubscriberProxy::new(self, subscriber_storage_weak)
        todo!()
    }
}

impl<'t, T: 'static> DomainParticipantTopicFactory<'t, T> for DomainParticipantImpl {
    type TopicType = TopicProxy<'t, T, TopicImpl>;

    fn topic_factory_create_topic(
        &'t self,
        _topic_name: &str,
        _qos: Option<TopicQos>,
        _a_listener: Option<&'static dyn TopicListener<DataType = T>>,
        _mask: StatusMask,
    ) -> Option<Self::TopicType> {
        // let topic_qos = qos.unwrap_or(self.default_topic_qos.clone());
        // let topic_storage = TopicImpl::new(topic_qos);
        // let topic_storage_shared = RtpsShared::new(topic_storage);
        // let topic_storage_weak = topic_storage_shared.downgrade();
        // self.topic_storage.push(topic_storage_shared);
        // Some(topic_storage_weak)

        // let topic_storage_weak = self
        //     .domain_participant_storage
        //     .lock()
        //     .create_topic(topic_name, qos, a_listener, mask)?;
        // let topic = TopicProxy::new(self, topic_storage_weak);
        // Some(topic)
        todo!()
    }

    fn topic_factory_delete_topic(&self, _a_topic: &Self::TopicType) -> DDSResult<()> {
        todo!()
    }

    fn topic_factory_find_topic(
        &self,
        _topic_name: &str,
        _timeout: Duration,
    ) -> Option<Self::TopicType> {
        todo!()
    }
}

impl DomainParticipant for DomainParticipantImpl {
    fn lookup_topicdescription<'t, T>(
        &'t self,
        _name: &'t str,
    ) -> Option<&'t (dyn TopicDescription<T> + 't)> {
        todo!()
    }

    fn ignore_participant(&self, _handle: InstanceHandle) -> DDSResult<()> {
        todo!()
    }

    fn ignore_topic(&self, _handle: InstanceHandle) -> DDSResult<()> {
        todo!()
    }

    fn ignore_publication(&self, _handle: InstanceHandle) -> DDSResult<()> {
        todo!()
    }

    fn ignore_subscription(&self, _handle: InstanceHandle) -> DDSResult<()> {
        todo!()
    }

    fn get_domain_id(&self) -> DomainId {
        // self.domain_id
        todo!()
    }

    fn delete_contained_entities(&self) -> DDSResult<()> {
        todo!()
    }

    fn assert_liveliness(&self) -> DDSResult<()> {
        todo!()
    }

    fn set_default_publisher_qos(&mut self, qos: Option<PublisherQos>) -> DDSResult<()> {
        self.default_publisher_qos = qos.unwrap_or_default();
        Ok(())
    }

    fn get_default_publisher_qos(&self) -> PublisherQos {
        self.default_publisher_qos.clone()
    }

    fn set_default_subscriber_qos(&mut self, qos: Option<SubscriberQos>) -> DDSResult<()> {
        self.default_subscriber_qos = qos.unwrap_or_default();
        Ok(())
    }

    fn get_default_subscriber_qos(&self) -> SubscriberQos {
        self.default_subscriber_qos.clone()
    }

    fn set_default_topic_qos(&mut self, qos: Option<TopicQos>) -> DDSResult<()> {
        let topic_qos = qos.unwrap_or_default();
        topic_qos.is_consistent()?;
        self.default_topic_qos = topic_qos;
        Ok(())
    }

    fn get_default_topic_qos(&self) -> TopicQos {
        self.default_topic_qos.clone()
    }

    fn get_discovered_participants(
        &self,
        _participant_handles: &mut [InstanceHandle],
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_discovered_participant_data(
        &self,
        _participant_data: ParticipantBuiltinTopicData,
        _participant_handle: InstanceHandle,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_discovered_topics(&self, _topic_handles: &mut [InstanceHandle]) -> DDSResult<()> {
        todo!()
    }

    fn get_discovered_topic_data(
        &self,
        _topic_data: TopicBuiltinTopicData,
        _topic_handle: InstanceHandle,
    ) -> DDSResult<()> {
        todo!()
    }

    fn contains_entity(&self, _a_handle: InstanceHandle) -> bool {
        todo!()
    }

    fn get_current_time(&self) -> DDSResult<Time> {
        todo!()
    }
}

impl Entity for DomainParticipantImpl {
    type Qos = DomainParticipantQos;
    type Listener = &'static dyn DomainParticipantListener;

    fn set_qos(&mut self, _qos: Option<Self::Qos>) -> DDSResult<()> {
        // self.qos = qos.unwrap_or_default();
        // Ok(())
        todo!()
        // self.domain_participant_storage.lock().set_qos(qos)
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        todo!()
        // Ok(self.domain_participant_storage.lock().get_qos().clone())
    }

    fn set_listener(
        &self,
        _a_listener: Option<Self::Listener>,
        _mask: StatusMask,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_listener(&self) -> DDSResult<Option<Self::Listener>> {
        todo!()
    }

    fn get_statuscondition(&self) -> DDSResult<StatusCondition> {
        todo!()
    }

    fn get_status_changes(&self) -> DDSResult<StatusMask> {
        todo!()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        todo!()
        // Ok(crate::utils::instance_handle_from_guid(
        //     &self.rtps_participant_impl.lock().guid(),
        // ))
    }

    fn enable(&self) -> DDSResult<()> {
        self.spawner.enable_tasks();

        let builtin_publisher_lock = self.builtin_publisher.write().unwrap();
        if let Some(spdp_builtin_participant_writer) =
            builtin_publisher_lock.lookup_datawriter::<SpdpDiscoveredParticipantData>(&())
        {
            let spdp_discovered_participant_data = self.as_spdp_discovered_participant_data();
            spdp_builtin_participant_writer
                .write()
                .unwrap()
                .write_w_timestamp(
                    &spdp_discovered_participant_data,
                    None,
                    Time { sec: 0, nanosec: 0 },
                )
                .unwrap();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::dds_type::{DdsSerialize, LittleEndian};

    use super::*;
    use rust_dds_api::{
        infrastructure::{
            qos::{DataReaderQos, DataWriterQos},
            qos_policy::UserDataQosPolicy,
        },
        return_type::DDSError,
    };
    use rust_rtps_pim::{
        behavior::writer::{
            reader_locator::RtpsReaderLocator, stateless_writer::RtpsStatelessWriterOperations,
        },
        discovery::spdp::builtin_endpoints::{
            SpdpBuiltinParticipantReader, SpdpBuiltinParticipantWriter,
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER,
        },
        structure::{
            cache_change::RtpsCacheChange,
            history_cache::RtpsHistoryCacheAddChange,
            types::{ChangeKind, LOCATOR_KIND_UDPv4, Locator, ENTITYID_UNKNOWN},
        },
    };
    use rust_rtps_psm::messages::overall_structure::{
        RtpsMessageRead, RtpsMessageWrite, RtpsSubmessageTypeWrite,
    };

    struct MockTransport;

    impl TransportRead for MockTransport {
        fn read(&mut self) -> Option<(Locator, RtpsMessageRead)> {
            todo!()
        }
    }

    impl TransportWrite for MockTransport {
        fn write(&mut self, _message: &RtpsMessageWrite, _destination_locator: &Locator) {
            todo!()
        }
    }

    #[test]
    fn set_default_publisher_qos_some_value() {
        let (sender, _receiver) = std::sync::mpsc::sync_channel(10);
        let spawner = Spawner::new(sender);
        let mut domain_participant = DomainParticipantImpl::new(
            GuidPrefix([3; 12]),
            1,
            "".to_string(),
            DomainParticipantQos::default(),
            MockTransport,
            MockTransport,
            vec![],
            vec![],
            vec![],
            vec![],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            spawner,
        );
        let mut qos = PublisherQos::default();
        qos.group_data.value = vec![1, 2, 3, 4];
        domain_participant
            .set_default_publisher_qos(Some(qos.clone()))
            .unwrap();
        assert!(domain_participant.get_default_publisher_qos() == qos);
    }

    #[test]
    fn set_default_publisher_qos_none() {
        let (sender, _receiver) = std::sync::mpsc::sync_channel(10);
        let spawner = Spawner::new(sender);
        let mut domain_participant = DomainParticipantImpl::new(
            GuidPrefix([0; 12]),
            1,
            "".to_string(),
            DomainParticipantQos::default(),
            MockTransport,
            MockTransport,
            vec![],
            vec![],
            vec![],
            vec![],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            spawner,
        );
        let mut qos = PublisherQos::default();
        qos.group_data.value = vec![1, 2, 3, 4];
        domain_participant
            .set_default_publisher_qos(Some(qos.clone()))
            .unwrap();

        domain_participant.set_default_publisher_qos(None).unwrap();
        assert!(domain_participant.get_default_publisher_qos() == PublisherQos::default());
    }

    #[test]
    fn set_default_subscriber_qos_some_value() {
        let (sender, _receiver) = std::sync::mpsc::sync_channel(10);
        let spawner = Spawner::new(sender);
        let mut domain_participant = DomainParticipantImpl::new(
            GuidPrefix([1; 12]),
            1,
            "".to_string(),
            DomainParticipantQos::default(),
            MockTransport,
            MockTransport,
            vec![],
            vec![],
            vec![],
            vec![],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            spawner,
        );
        let mut qos = SubscriberQos::default();
        qos.group_data.value = vec![1, 2, 3, 4];
        domain_participant
            .set_default_subscriber_qos(Some(qos.clone()))
            .unwrap();
        assert_eq!(domain_participant.get_default_subscriber_qos(), qos);
    }

    #[test]
    fn set_default_subscriber_qos_none() {
        let (sender, _receiver) = std::sync::mpsc::sync_channel(10);
        let spawner = Spawner::new(sender);
        let mut domain_participant = DomainParticipantImpl::new(
            GuidPrefix([1; 12]),
            1,
            "".to_string(),
            DomainParticipantQos::default(),
            MockTransport,
            MockTransport,
            vec![],
            vec![],
            vec![],
            vec![],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            spawner,
        );
        let mut qos = SubscriberQos::default();
        qos.group_data.value = vec![1, 2, 3, 4];
        domain_participant
            .set_default_subscriber_qos(Some(qos.clone()))
            .unwrap();

        domain_participant.set_default_subscriber_qos(None).unwrap();
        assert_eq!(
            domain_participant.get_default_subscriber_qos(),
            SubscriberQos::default()
        );
    }

    #[test]
    fn set_default_topic_qos_some_value() {
        let (sender, _receiver) = std::sync::mpsc::sync_channel(10);
        let spawner = Spawner::new(sender);
        let mut domain_participant = DomainParticipantImpl::new(
            GuidPrefix([1; 12]),
            1,
            "".to_string(),
            DomainParticipantQos::default(),
            MockTransport,
            MockTransport,
            vec![],
            vec![],
            vec![],
            vec![],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            spawner,
        );
        let mut qos = TopicQos::default();
        qos.topic_data.value = vec![1, 2, 3, 4];
        domain_participant
            .set_default_topic_qos(Some(qos.clone()))
            .unwrap();
        assert_eq!(domain_participant.get_default_topic_qos(), qos);
    }

    #[test]
    fn set_default_topic_qos_inconsistent() {
        let (sender, _receiver) = std::sync::mpsc::sync_channel(10);
        let spawner = Spawner::new(sender);
        let mut domain_participant = DomainParticipantImpl::new(
            GuidPrefix([1; 12]),
            1,
            "".to_string(),
            DomainParticipantQos::default(),
            MockTransport,
            MockTransport,
            vec![],
            vec![],
            vec![],
            vec![],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            spawner,
        );
        let mut qos = TopicQos::default();
        qos.resource_limits.max_samples_per_instance = 2;
        qos.resource_limits.max_samples = 1;
        let set_default_topic_qos_result =
            domain_participant.set_default_topic_qos(Some(qos.clone()));
        assert!(set_default_topic_qos_result == Err(DDSError::InconsistentPolicy));
    }

    #[test]
    fn set_default_topic_qos_none() {
        let (sender, _receiver) = std::sync::mpsc::sync_channel(10);
        let spawner = Spawner::new(sender);
        let mut domain_participant = DomainParticipantImpl::new(
            GuidPrefix([1; 12]),
            1,
            "".to_string(),
            DomainParticipantQos::default(),
            MockTransport,
            MockTransport,
            vec![],
            vec![],
            vec![],
            vec![],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            spawner,
        );
        let mut qos = TopicQos::default();
        qos.topic_data.value = vec![1, 2, 3, 4];
        domain_participant
            .set_default_topic_qos(Some(qos.clone()))
            .unwrap();

        domain_participant.set_default_topic_qos(None).unwrap();
        assert_eq!(
            domain_participant.get_default_topic_qos(),
            TopicQos::default()
        );
    }

    #[test]
    fn create_publisher() {
        let (sender, _receiver) = std::sync::mpsc::sync_channel(10);
        let spawner = Spawner::new(sender);
        let domain_participant = DomainParticipantImpl::new(
            GuidPrefix([1; 12]),
            1,
            "".to_string(),
            DomainParticipantQos::default(),
            MockTransport,
            MockTransport,
            vec![],
            vec![],
            vec![],
            vec![],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            spawner,
        );

        let publisher_counter_before = domain_participant
            .user_defined_publisher_counter
            .load(atomic::Ordering::Relaxed);
        let publisher = domain_participant.create_publisher(None, None, 0);

        let publisher_counter_after = domain_participant
            .user_defined_publisher_counter
            .load(atomic::Ordering::Relaxed);

        assert_eq!(
            domain_participant
                .user_defined_publisher_list
                .lock()
                .unwrap()
                .len(),
            1
        );

        assert_ne!(publisher_counter_before, publisher_counter_after);
        assert!(publisher.is_some());
    }

    #[test]
    fn delete_publisher() {
        let (sender, _receiver) = std::sync::mpsc::sync_channel(10);
        let spawner = Spawner::new(sender);
        let domain_participant = DomainParticipantImpl::new(
            GuidPrefix([1; 12]),
            1,
            "".to_string(),
            DomainParticipantQos::default(),
            MockTransport,
            MockTransport,
            vec![],
            vec![],
            vec![],
            vec![],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            spawner,
        );
        let a_publisher = domain_participant.create_publisher(None, None, 0).unwrap();

        domain_participant.delete_publisher(&a_publisher).unwrap();
        assert_eq!(
            domain_participant
                .user_defined_publisher_list
                .lock()
                .unwrap()
                .len(),
            0
        );
    }

    #[test]
    fn domain_participant_as_spdp_discovered_participant_data() {
        let (sender, _receiver) = std::sync::mpsc::sync_channel(10);
        let spawner = Spawner::new(sender);
        let domain_participant = DomainParticipantImpl::new(
            GuidPrefix([1; 12]),
            1,
            "".to_string(),
            DomainParticipantQos::default(),
            MockTransport,
            MockTransport,
            vec![],
            vec![],
            vec![],
            vec![],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            spawner,
        );
        let spdp_discovered_participant_data =
            domain_participant.as_spdp_discovered_participant_data();
        let expected_spdp_discovered_participant_data = SpdpDiscoveredParticipantData {
            dds_participant_data: ParticipantBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0xc1],
                },
                user_data: UserDataQosPolicy { value: vec![] },
            },
            participant_proxy: ParticipantProxy {
                domain_id: 1,
                domain_tag: "".to_string(),
                protocol_version: PROTOCOLVERSION,
                guid_prefix: GuidPrefix([1; 12]),
                vendor_id: VENDOR_ID_S2E,
                expects_inline_qos: false,
                metatraffic_unicast_locator_list: vec![],
                metatraffic_multicast_locator_list: vec![],
                default_unicast_locator_list: vec![],
                default_multicast_locator_list: vec![],
                available_builtin_endpoints: BuiltinEndpointSet::default(),
                manual_liveliness_count: Count(0),
                builtin_endpoint_qos: BuiltinEndpointQos::default(),
            },
            lease_duration: rust_rtps_pim::behavior::types::Duration::new(100, 0),
        };

        assert_eq!(
            spdp_discovered_participant_data,
            expected_spdp_discovered_participant_data
        );
    }

    #[test]
    fn spdp_data_sent() {
        const SPDP_TEST_LOCATOR: Locator = Locator {
            kind: LOCATOR_KIND_UDPv4,
            port: 7400,
            address: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0, 1],
        };
        struct TestTransport;
        impl TransportRead for TestTransport {
            fn read(&mut self) -> Option<(Locator, RtpsMessageRead)> {
                None
            }
        }
        impl TransportWrite for TestTransport {
            fn write(&mut self, message: &RtpsMessageWrite, destination_locator: &Locator) {
                assert_eq!(message.submessages.len(), 1);
                match &message.submessages[0] {
                    RtpsSubmessageTypeWrite::Data(data_submessage) => {
                        assert_eq!(data_submessage.reader_id.value, ENTITYID_UNKNOWN);
                        assert_eq!(
                            data_submessage.writer_id.value,
                            ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER
                        );
                    }
                    _ => assert!(false),
                };
                assert_eq!(destination_locator, &SPDP_TEST_LOCATOR);
                println!("Writing {:?}, to {:?}", message, destination_locator);
            }
        }

        let (sender, receiver) = std::sync::mpsc::sync_channel(10);
        let spawner = Spawner::new(sender);

        let guid_prefix = GuidPrefix([1; 12]);
        let mut spdp_builtin_participant_rtps_writer = RtpsStatelessWriterImpl::new(
            SpdpBuiltinParticipantWriter::create(guid_prefix, vec![], vec![]),
        );

        let spdp_discovery_locator = RtpsReaderLocator::new(SPDP_TEST_LOCATOR, false);

        spdp_builtin_participant_rtps_writer.reader_locator_add(spdp_discovery_locator);

        let spdp_builtin_participant_data_writer =
            Some(DataWriterImpl::<SpdpDiscoveredParticipantData, _, _>::new(
                DataWriterQos::default(),
                spdp_builtin_participant_rtps_writer,
                StdTimer::new(),
            ));

        let domain_participant = DomainParticipantImpl::new(
            guid_prefix,
            1,
            "".to_string(),
            DomainParticipantQos::default(),
            TestTransport,
            TestTransport,
            vec![],
            vec![],
            vec![],
            vec![],
            None,
            spdp_builtin_participant_data_writer,
            None,
            None,
            None,
            None,
            None,
            None,
            spawner,
        );
        let mut tasks = Vec::new();
        tasks.push(receiver.recv().unwrap());
        tasks.push(receiver.recv().unwrap());

        domain_participant.enable().unwrap();

        let builtin_communication_task = tasks
            .iter_mut()
            .find(|x| x.name == "builtin communication")
            .unwrap();

        (builtin_communication_task.task)()
    }

    #[test]
    fn spdp_discovery_read() {
        const SPDP_TEST_LOCATOR: Locator = Locator {
            kind: LOCATOR_KIND_UDPv4,
            port: 7400,
            address: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0, 1],
        };
        struct TestTransport;
        impl TransportRead for TestTransport {
            fn read(&mut self) -> Option<(Locator, RtpsMessageRead)> {
                None
            }
        }
        impl TransportWrite for TestTransport {
            fn write(&mut self, message: &RtpsMessageWrite, destination_locator: &Locator) {
                assert_eq!(message.submessages.len(), 1);
                match &message.submessages[0] {
                    RtpsSubmessageTypeWrite::Data(data_submessage) => {
                        assert_eq!(data_submessage.reader_id.value, ENTITYID_UNKNOWN);
                        assert_eq!(
                            data_submessage.writer_id.value,
                            ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER
                        );
                    }
                    _ => assert!(false),
                };
                assert_eq!(destination_locator, &SPDP_TEST_LOCATOR);
                println!("Writing {:?}, to {:?}", message, destination_locator);
            }
        }

        let (sender, _receiver) = std::sync::mpsc::sync_channel(10);
        let spawner = Spawner::new(sender);

        let guid_prefix = GuidPrefix([1; 12]);
        let spdp_builtin_participant_rtps_reader =
            SpdpBuiltinParticipantReader::create(guid_prefix, vec![], vec![]);

        let mut spdp_builtin_participant_data_reader =
            DataReaderImpl::<SpdpDiscoveredParticipantData>::new(
                DataReaderQos::default(),
                spdp_builtin_participant_rtps_reader,
            );

        let spdp_discovered_participant_data = SpdpDiscoveredParticipantData {
            dds_participant_data: ParticipantBuiltinTopicData {
                key: BuiltInTopicKey { value: [2; 16] },
                user_data: UserDataQosPolicy { value: vec![] },
            },
            participant_proxy: ParticipantProxy {
                domain_id: 1,
                domain_tag: "".to_string(),
                protocol_version: PROTOCOLVERSION,
                guid_prefix: GuidPrefix([2; 12]),
                vendor_id: VENDOR_ID_S2E,
                expects_inline_qos: false,
                metatraffic_unicast_locator_list: vec![],
                metatraffic_multicast_locator_list: vec![],
                default_unicast_locator_list: vec![],
                default_multicast_locator_list: vec![],
                available_builtin_endpoints: BuiltinEndpointSet::default(),
                manual_liveliness_count: Count(1),
                builtin_endpoint_qos: BuiltinEndpointQos::default(),
            },
            lease_duration: rust_rtps_pim::behavior::types::Duration::new(100, 0),
        };

        let mut serialized_data = Vec::new();
        spdp_discovered_participant_data
            .serialize::<_, LittleEndian>(&mut serialized_data)
            .unwrap();

        spdp_builtin_participant_data_reader
            .rtps_reader
            .reader_cache
            .add_change(RtpsCacheChange {
                kind: ChangeKind::Alive,
                writer_guid: Guid::new(
                    GuidPrefix([2; 12]),
                    ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER,
                ),
                instance_handle: 1,
                sequence_number: 1,
                data_value: &serialized_data,
                inline_qos: &[],
            });

        let domain_participant = DomainParticipantImpl::new(
            guid_prefix,
            1,
            "".to_string(),
            DomainParticipantQos::default(),
            TestTransport,
            TestTransport,
            vec![],
            vec![],
            vec![],
            vec![],
            Some(spdp_builtin_participant_data_reader),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            spawner,
        );
        // let mut tasks = Vec::new();
        // tasks.push(receiver.recv().unwrap());
        // tasks.push(receiver.recv().unwrap());

        domain_participant.enable().unwrap();

        // let builtin_communication_task = tasks
        //     .iter_mut()
        //     .find(|x| x.name == "builtin communication")
        //     .unwrap();

        // (builtin_communication_task.task)()
    }
}
