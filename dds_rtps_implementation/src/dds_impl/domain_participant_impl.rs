use std::sync::{
    atomic::{self, AtomicBool, AtomicU8},
    Arc, Mutex,
};

use rust_dds_api::{
    builtin_topics::{ParticipantBuiltinTopicData, TopicBuiltinTopicData},
    dcps_psm::{DomainId, Duration, InstanceHandle, StatusMask, Time},
    domain::{
        domain_participant::{DomainParticipant, PublisherGAT, SubscriberGAT, TopicGAT},
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
use rust_rtps_pim::structure::{
    group::RtpsGroup,
    types::{
        EntityId, Guid, GuidPrefix, ProtocolVersion, VendorId, PROTOCOLVERSION,
        USER_DEFINED_WRITER_GROUP, VENDOR_ID_S2E,
    },
};

use crate::{
    data_representation_builtin_endpoints::{
        sedp_discovered_reader_data::SedpDiscoveredReaderData,
        spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
    },
    utils::{
        message_receiver::ProcessDataSubmessage,
        shared_object::{rtps_shared_downgrade, rtps_shared_new, rtps_weak_upgrade, RtpsShared},
        transport::{TransportRead, TransportWrite},
    },
};

use super::{
    publisher_impl::PublisherImpl, publisher_proxy::PublisherProxy,
    subscriber_impl::SubscriberImpl, subscriber_proxy::SubscriberProxy, topic_impl::TopicImpl,
    topic_proxy::TopicProxy,
};

pub trait Transport: TransportRead + TransportWrite + Send + Sync {}

impl<T> Transport for T where T: TransportRead + TransportWrite + Send + Sync {}

pub struct DomainParticipantImpl {
    guid_prefix: GuidPrefix,
    _qos: DomainParticipantQos,
    _builtin_subscriber: RtpsShared<SubscriberImpl>,
    _builtin_publisher: RtpsShared<PublisherImpl>,
    _user_defined_subscriber_list: Arc<Mutex<Vec<RtpsShared<SubscriberImpl>>>>,
    _user_defined_subscriber_counter: u8,
    default_subscriber_qos: SubscriberQos,
    user_defined_publisher_list: Arc<Mutex<Vec<RtpsShared<PublisherImpl>>>>,
    user_defined_publisher_counter: AtomicU8,
    default_publisher_qos: PublisherQos,
    _topic_list: Vec<RtpsShared<TopicImpl>>,
    default_topic_qos: TopicQos,
    is_enabled: Arc<AtomicBool>,
}

impl DomainParticipantImpl {
    pub fn new(
        guid_prefix: GuidPrefix,
        domain_participant_qos: DomainParticipantQos,
        builtin_subscriber: RtpsShared<SubscriberImpl>,
        builtin_publisher: RtpsShared<PublisherImpl>,
        metatraffic_transport: Box<dyn Transport>,
        default_transport: Box<dyn Transport>,
    ) -> Self {
        let protocol_version = PROTOCOLVERSION;
        let vendor_id = VENDOR_ID_S2E;
        let is_enabled = Arc::new(AtomicBool::new(false));
        let is_enabled_arc = is_enabled.clone();
        // let default_transport = default_transport.clone();
        // let metatraffic_transport = metatraffic_transport.clone();
        // let guid_prefix = guid_prefix;
        let builtin_subscriber_arc = builtin_subscriber.clone();
        let builtin_publisher_arc = builtin_publisher.clone();
        let user_defined_subscriber_list = Arc::new(Mutex::new(Vec::new()));
        let user_defined_subscriber_list_arc = user_defined_subscriber_list.clone();
        let user_defined_publisher_list = Arc::new(Mutex::new(Vec::new()));
        let user_defined_publisher_list_arc = user_defined_publisher_list.clone();

        let _option_spdp_builtin_participant_reader =
            builtin_subscriber
                .read()
                .unwrap()
                .lookup_datareader::<SpdpDiscoveredParticipantData>(&());

        let _option_sedp_builtin_publications_reader =
            builtin_subscriber
                .read()
                .unwrap()
                .lookup_datareader::<SedpDiscoveredReaderData>(&());

        struct Communication {
            version: ProtocolVersion,
            vendor_id: VendorId,
            guid_prefix: GuidPrefix,
            transport: Box<dyn Transport>,
        }

        impl Communication {
            fn send(&mut self, list: &[RtpsShared<PublisherImpl>]) {
                for publisher in list {
                    publisher
                        .write()
                        .unwrap()
                        .send_message(self.transport.as_mut());
                }
                // if let Ok((dst_locator, submessages)) =
                //     self.locator_message_channel_receiver.try_recv()
                // {
                //     let message = RtpsMessageWrite::new(
                //         RtpsMessageHeader {
                //             protocol: rust_rtps_pim::messages::types::ProtocolId::PROTOCOL_RTPS,
                //             version: self.version,
                //             vendor_id: self.vendor_id,
                //             guid_prefix: self.guid_prefix,
                //         },
                //         submessages,
                //     );
                //     self.transport.write(&message, &dst_locator);
                // };
            }

            fn receive(&mut self, list: &[RtpsShared<impl ProcessDataSubmessage>]) {
                if let Some((source_locator, message)) = self.transport.read() {
                    crate::utils::message_receiver::MessageReceiver::new().process_message(
                        self.guid_prefix,
                        list,
                        source_locator,
                        &message,
                    );
                }
            }
        }

        std::thread::spawn(move || {
            while !is_enabled_arc.load(atomic::Ordering::SeqCst) {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            let mut communication = Communication {
                version: protocol_version,
                vendor_id,
                guid_prefix,
                transport: metatraffic_transport,
            };

            while is_enabled_arc.load(atomic::Ordering::SeqCst) {
                communication.send(core::slice::from_ref(&builtin_publisher_arc));
                communication.receive(core::slice::from_ref(&builtin_subscriber_arc));
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        });

        let is_enabled_arc = is_enabled.clone();
        std::thread::spawn(move || {
            while !is_enabled_arc.load(atomic::Ordering::SeqCst) {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }

            let mut communication = Communication {
                version: protocol_version,
                vendor_id,
                guid_prefix,
                transport: default_transport,
            };

            while is_enabled_arc.load(atomic::Ordering::SeqCst) {
                communication.send(&user_defined_publisher_list_arc.lock().unwrap());
                communication.receive(&user_defined_subscriber_list_arc.lock().unwrap());
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        });

        //     if let Some(spdp_builtin_participant_reader) =
        //         &option_spdp_builtin_participant_reader
        //     {
        //         if let Ok(discovered_participant) = rtps_shared_write_lock(
        //             &spdp_builtin_participant_reader,
        //         )
        //         .read(1, &[], &[], &[])
        //         {
        //             let local_participant_domain_id = 1;
        //             let local_participant_domain_tag = "ab";

        //             if let Ok(participant_discovery) = ParticipantDiscovery::new(
        //                 &discovered_participant[0].participant_proxy,
        //                 local_participant_domain_id,
        //                 local_participant_domain_tag,
        //             ) {
        //                 if let Some(sedp_builtin_publications_reader) =
        //                     &option_sedp_builtin_publications_reader
        //                 {
        //                     let reader = &mut (rtps_shared_write_lock(
        //                         &sedp_builtin_publications_reader,
        //                     )
        //                     .rtps_reader);
        //                     if let RtpsReaderFlavor::Stateful(stateful_reader) = reader {
        //                         participant_discovery
        //                             .discovered_participant_add_publications_reader(
        //                                 stateful_reader,
        //                             );
        //                     }
        //                 }
        //             }
        //         }
        //     }

        Self {
            guid_prefix,
            _qos: domain_participant_qos,
            _builtin_subscriber: builtin_subscriber,
            _builtin_publisher: builtin_publisher,
            _user_defined_subscriber_list: user_defined_subscriber_list,
            _user_defined_subscriber_counter: 0,
            default_subscriber_qos: SubscriberQos::default(),
            user_defined_publisher_list,
            user_defined_publisher_counter: AtomicU8::new(0),
            default_publisher_qos: PublisherQos::default(),
            _topic_list: Vec::new(),
            default_topic_qos: TopicQos::default(),
            is_enabled,
        }
    }
}

impl<'p> PublisherGAT<'p> for DomainParticipantImpl {
    type PublisherType = PublisherProxy<'p, PublisherImpl>;
    fn create_publisher_gat(
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
        let guid = Guid::new(self.guid_prefix, entity_id);
        let rtps_group = RtpsGroup::new(guid);
        let data_writer_impl_list = Vec::new();
        let publisher_impl = PublisherImpl::new(publisher_qos, rtps_group, data_writer_impl_list);
        let publisher_impl_shared = rtps_shared_new(publisher_impl);
        let publisher_impl_weak = rtps_shared_downgrade(&publisher_impl_shared);
        self.user_defined_publisher_list
            .lock()
            .unwrap()
            .push(publisher_impl_shared);
        let publisher = PublisherProxy::new(self, publisher_impl_weak);

        Some(publisher)
    }

    fn delete_publisher_gat(&self, a_publisher: &Self::PublisherType) -> DDSResult<()> {
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
                "Publisher can only be deleted from its parent participant",
            ))
        }
    }
}

impl<'s> SubscriberGAT<'s> for DomainParticipantImpl {
    type SubscriberType = SubscriberProxy<'s, SubscriberImpl>;

    fn create_subscriber_gat(
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

    fn delete_subscriber_gat(&self, _a_subscriber: &Self::SubscriberType) -> DDSResult<()> {
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

    fn get_builtin_subscriber_gat(&'s self) -> Self::SubscriberType {
        // self.builtin_subscriber_storage[0].clone().downgrade()

        // let subscriber_storage_weak = self
        //     .domain_participant_storage
        //     .lock()
        //     .get_builtin_subscriber();
        // SubscriberProxy::new(self, subscriber_storage_weak)
        todo!()
    }
}

impl<'t, T: 'static> TopicGAT<'t, T> for DomainParticipantImpl {
    type TopicType = TopicProxy<'t, T, TopicImpl>;

    fn create_topic_gat(
        &'t self,
        _topic_name: &str,
        _qos: Option<TopicQos>,
        _a_listener: Option<&'static dyn TopicListener<DataPIM = T>>,
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

    fn delete_topic_gat(&self, _a_topic: &Self::TopicType) -> DDSResult<()> {
        todo!()
    }

    fn find_topic_gat(&self, _topic_name: &str, _timeout: Duration) -> Option<Self::TopicType> {
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
        self.is_enabled.store(true, atomic::Ordering::SeqCst);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_dds_api::return_type::DDSError;
    use rust_rtps_pim::structure::types::{Locator, GUID_UNKNOWN};
    use rust_rtps_psm::messages::overall_structure::{RtpsMessageRead, RtpsMessageWrite};

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
        let builtin_subscriber = rtps_shared_new(SubscriberImpl::new(
            SubscriberQos::default(),
            RtpsGroup::new(GUID_UNKNOWN),
            vec![],
        ));
        let builtin_publisher = rtps_shared_new(PublisherImpl::new(
            PublisherQos::default(),
            RtpsGroup::new(GUID_UNKNOWN),
            vec![],
        ));
        let mut domain_participant = DomainParticipantImpl::new(
            GuidPrefix([3; 12]),
            DomainParticipantQos::default(),
            builtin_subscriber,
            builtin_publisher,
            Box::new(MockTransport),
            Box::new(MockTransport),
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
        let builtin_subscriber = rtps_shared_new(SubscriberImpl::new(
            SubscriberQos::default(),
            RtpsGroup::new(GUID_UNKNOWN),
            vec![],
        ));
        let builtin_publisher = rtps_shared_new(PublisherImpl::new(
            PublisherQos::default(),
            RtpsGroup::new(GUID_UNKNOWN),
            vec![],
        ));
        let mut domain_participant = DomainParticipantImpl::new(
            GuidPrefix([0; 12]),
            DomainParticipantQos::default(),
            builtin_subscriber,
            builtin_publisher,
            Box::new(MockTransport),
            Box::new(MockTransport),
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
        let builtin_subscriber = rtps_shared_new(SubscriberImpl::new(
            SubscriberQos::default(),
            RtpsGroup::new(GUID_UNKNOWN),
            vec![],
        ));
        let builtin_publisher = rtps_shared_new(PublisherImpl::new(
            PublisherQos::default(),
            RtpsGroup::new(GUID_UNKNOWN),
            vec![],
        ));
        let mut domain_participant = DomainParticipantImpl::new(
            GuidPrefix([1; 12]),
            DomainParticipantQos::default(),
            builtin_subscriber,
            builtin_publisher,
            Box::new(MockTransport),
            Box::new(MockTransport),
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
        let builtin_subscriber = rtps_shared_new(SubscriberImpl::new(
            SubscriberQos::default(),
            RtpsGroup::new(GUID_UNKNOWN),
            vec![],
        ));
        let builtin_publisher = rtps_shared_new(PublisherImpl::new(
            PublisherQos::default(),
            RtpsGroup::new(GUID_UNKNOWN),
            vec![],
        ));
        let mut domain_participant = DomainParticipantImpl::new(
            GuidPrefix([1; 12]),
            DomainParticipantQos::default(),
            builtin_subscriber,
            builtin_publisher,
            Box::new(MockTransport),
            Box::new(MockTransport),
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
        let builtin_subscriber = rtps_shared_new(SubscriberImpl::new(
            SubscriberQos::default(),
            RtpsGroup::new(GUID_UNKNOWN),
            vec![],
        ));
        let builtin_publisher = rtps_shared_new(PublisherImpl::new(
            PublisherQos::default(),
            RtpsGroup::new(GUID_UNKNOWN),
            vec![],
        ));
        let mut domain_participant = DomainParticipantImpl::new(
            GuidPrefix([1; 12]),
            DomainParticipantQos::default(),
            builtin_subscriber,
            builtin_publisher,
            Box::new(MockTransport),
            Box::new(MockTransport),
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
        let builtin_subscriber = rtps_shared_new(SubscriberImpl::new(
            SubscriberQos::default(),
            RtpsGroup::new(GUID_UNKNOWN),
            vec![],
        ));
        let builtin_publisher = rtps_shared_new(PublisherImpl::new(
            PublisherQos::default(),
            RtpsGroup::new(GUID_UNKNOWN),
            vec![],
        ));
        let mut domain_participant = DomainParticipantImpl::new(
            GuidPrefix([1; 12]),
            DomainParticipantQos::default(),
            builtin_subscriber,
            builtin_publisher,
            Box::new(MockTransport),
            Box::new(MockTransport),
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
        let builtin_subscriber = rtps_shared_new(SubscriberImpl::new(
            SubscriberQos::default(),
            RtpsGroup::new(GUID_UNKNOWN),
            vec![],
        ));
        let builtin_publisher = rtps_shared_new(PublisherImpl::new(
            PublisherQos::default(),
            RtpsGroup::new(GUID_UNKNOWN),
            vec![],
        ));
        let mut domain_participant = DomainParticipantImpl::new(
            GuidPrefix([1; 12]),
            DomainParticipantQos::default(),
            builtin_subscriber,
            builtin_publisher,
            Box::new(MockTransport),
            Box::new(MockTransport),
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
        let builtin_subscriber = rtps_shared_new(SubscriberImpl::new(
            SubscriberQos::default(),
            RtpsGroup::new(GUID_UNKNOWN),
            vec![],
        ));
        let builtin_publisher = rtps_shared_new(PublisherImpl::new(
            PublisherQos::default(),
            RtpsGroup::new(GUID_UNKNOWN),
            vec![],
        ));
        let domain_participant = DomainParticipantImpl::new(
            GuidPrefix([1; 12]),
            DomainParticipantQos::default(),
            builtin_subscriber,
            builtin_publisher,
            Box::new(MockTransport),
            Box::new(MockTransport),
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
        let builtin_subscriber = rtps_shared_new(SubscriberImpl::new(
            SubscriberQos::default(),
            RtpsGroup::new(GUID_UNKNOWN),
            vec![],
        ));
        let builtin_publisher = rtps_shared_new(PublisherImpl::new(
            PublisherQos::default(),
            RtpsGroup::new(GUID_UNKNOWN),
            vec![],
        ));
        let domain_participant = DomainParticipantImpl::new(
            GuidPrefix([1; 12]),
            DomainParticipantQos::default(),
            builtin_subscriber,
            builtin_publisher,
            Box::new(MockTransport),
            Box::new(MockTransport),
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
}
