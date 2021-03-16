use std::{
    cell::RefCell,
    sync::{atomic, Arc, Mutex, Once, Weak},
    thread::JoinHandle,
};

use rust_dds_api::{
    dcps_psm::{DomainId, StatusMask},
    dds_type::DDSType,
    domain::domain_participant_listener::DomainParticipantListener,
    infrastructure::qos::{DomainParticipantQos, PublisherQos, SubscriberQos, TopicQos},
    publication::publisher_listener::PublisherListener,
    return_type::{DDSError, DDSResult},
    subscription::subscriber_listener::SubscriberListener,
    topic::topic_listener::TopicListener,
};
use rust_rtps::{
    discovery::spdp_endpoints::SPDPbuiltinParticipantWriter,
    structure::{RTPSEntity, RTPSParticipant},
    types::{
        constants::{ENTITYID_PARTICIPANT, PROTOCOL_VERSION_2_4, VENDOR_ID},
        GuidPrefix, Locator, ProtocolVersion, VendorId, GUID,
    },
};

use crate::transport::Transport;

use super::{
    data_writer_impl::DataWriterImpl, publisher_impl::PublisherImpl,
    subscriber_impl::SubscriberImpl, topic_impl::TopicImpl,
};

struct RtpsBuiltinParticipantEntities {
    publisher: PublisherImpl,
    subscriber: SubscriberImpl,
    transport: Box<dyn Transport>,
}

struct RtpsParticipantEntities {
    publisher_list: Mutex<Vec<Arc<Mutex<PublisherImpl>>>>,
    subscriber_list: Mutex<Vec<Arc<Mutex<SubscriberImpl>>>>,
    topic_list: Mutex<Vec<Arc<Mutex<TopicImpl>>>>,
    transport: Box<dyn Transport>,
}

impl RtpsParticipantEntities {
    fn new(transport: Box<dyn Transport>) -> Self {
        Self {
            publisher_list: Default::default(),
            subscriber_list: Default::default(),
            topic_list: Default::default(),
            transport,
        }
    }

    pub fn send_data(&self, _participant_guid_prefix: GuidPrefix) {
        // let _transport = &self.transport;
        let publisher_list = self.publisher_list.lock().unwrap();
        for _publisher in publisher_list.iter() {
            // for _writer in publisher.lock().unwrap().writer_list() {
            todo!()
            // let destined_messages = writer.lock().unwrap().produce_messages();
            // RtpsMessageSender::send_cache_change_messages(
            //     participant_guid_prefix,
            //     transport.as_ref(),
            //     destined_messages,
            // );
            // }
        }
    }
}

pub struct DomainParticipantImpl {
    domain_id: DomainId,
    guid_prefix: GuidPrefix,
    qos: DomainParticipantQos,
    publisher_count: usize,
    subscriber_count: usize,
    topic_count: usize,
    default_publisher_qos: PublisherQos,
    default_subscriber_qos: SubscriberQos,
    default_topic_qos: TopicQos,
    builtin_entities: Arc<RtpsBuiltinParticipantEntities>,
    user_defined_entities: Arc<RtpsParticipantEntities>,
    enabled: Arc<atomic::AtomicBool>,
    enabled_function: Once,
    thread_list: RefCell<Vec<JoinHandle<()>>>,
    a_listener: Option<Box<dyn DomainParticipantListener>>,
    mask: StatusMask,
}

impl DomainParticipantImpl {
    pub fn new(
        domain_id: DomainId,
        qos: DomainParticipantQos,
        userdata_transport: Box<dyn Transport>,
        metatraffic_transport: Box<dyn Transport>,
        a_listener: Option<Box<dyn DomainParticipantListener>>,
        mask: StatusMask,
        spdp_locator_list: &[Locator],
    ) -> Self {
        let guid_prefix = [1; 12];

        let spdp_builtin_participant_writer = SPDPbuiltinParticipantWriter::create(
            guid_prefix,
            metatraffic_transport.unicast_locator_list(),
            metatraffic_transport.multicast_locator_list(),
            spdp_locator_list,
        );

        let mut builtin_publisher = PublisherImpl::new(PublisherQos::default(), None, 0);
        let spdp_announcer = Arc::new(Mutex::new(DataWriterImpl::new(
            spdp_builtin_participant_writer,
        )));
        builtin_publisher.add_datawriter(spdp_announcer);

        let builtin_subscriber = SubscriberImpl::new(SubscriberQos::default(), None, 0);
        // spdp_announcer.write_w_timestamp(data, handle, timestamp);
        let builtin_entities = Arc::new(RtpsBuiltinParticipantEntities {
            publisher: builtin_publisher,
            subscriber: builtin_subscriber,
            transport: metatraffic_transport,
        });

        let user_defined_entities = Arc::new(RtpsParticipantEntities::new(userdata_transport));

        Self {
            domain_id,
            guid_prefix,
            qos,
            publisher_count: 0,
            subscriber_count: 0,
            topic_count: 0,
            default_publisher_qos: PublisherQos::default(),
            default_subscriber_qos: SubscriberQos::default(),
            default_topic_qos: TopicQos::default(),
            builtin_entities,
            user_defined_entities,
            enabled: Arc::new(atomic::AtomicBool::new(false)),
            enabled_function: Once::new(),
            thread_list: RefCell::new(Vec::new()),
            a_listener,
            mask,
        }
    }

    pub fn create_publisher(
        &self,
        qos: Option<PublisherQos>,
        a_listener: Option<Box<dyn PublisherListener>>,
        mask: StatusMask,
    ) -> DDSResult<Weak<Mutex<PublisherImpl>>> {
        // let guid_prefix = self.participant.entity.guid.prefix();
        let qos = qos.unwrap_or(self.get_default_publisher_qos());
        let publisher = Arc::new(Mutex::new(PublisherImpl::new(qos, a_listener, mask)));

        self.user_defined_entities
            .publisher_list
            .lock()
            .unwrap()
            .push(publisher.clone());

        Ok(Arc::downgrade(&publisher))
    }

    pub fn delete_publisher(&self, _impl_ref: &Weak<Mutex<PublisherImpl>>) -> DDSResult<()> {
        todo!()
        // let publisher_impl = impl_ref.upgrade().ok_or(DDSError::AlreadyDeleted)?;
        // if publisher_impl.lock().unwrap().writer_list().is_empty() {
        //     self.user_defined_entities
        //         .publisher_list
        //         .lock()
        //         .unwrap()
        //         .retain(|x| !Arc::ptr_eq(x, &publisher_impl));
        //     Ok(())
        // } else {
        //     Err(DDSError::PreconditionNotMet(
        //         "Publisher still contains data writers",
        //     ))
        // }
    }

    pub fn create_subscriber(
        &self,
        qos: Option<SubscriberQos>,
        a_listener: Option<Box<dyn SubscriberListener>>,
        mask: StatusMask,
    ) -> DDSResult<Weak<Mutex<SubscriberImpl>>> {
        // let guid_prefix = self.participant.entity.guid.prefix();
        // let qos = qos.unwrap_or(self.get_default_publisher_qos());
        // let publisher = Arc::new(Mutex::new(RtpsPublisherImpl::new(qos, a_listener, mask)));

        // self.user_defined_entities
        //     .publisher_list
        //     .lock()
        //     .unwrap()
        //     .push(publisher.clone());

        // let guid_prefix = self.participant.entity.guid.prefix();
        // let entity_key = [
        //     0,
        //     self.subscriber_count
        //         .fetch_add(1, atomic::Ordering::Relaxed),
        //     0,
        // ];
        // let entity_kind = ENTITY_KIND_USER_DEFINED_READER_GROUP;
        // let entity_id = EntityId::new(entity_key, entity_kind);
        // let guid = GUID::new(guid_prefix, entity_id);
        // let group = rust_rtps::structure::Group::new(guid);
        let qos = qos.unwrap_or(self.get_default_subscriber_qos().clone());
        let subscriber = Arc::new(Mutex::new(SubscriberImpl::new(qos, a_listener, mask)));

        self.user_defined_entities
            .subscriber_list
            .lock()
            .unwrap()
            .push(subscriber.clone());

        Ok(Arc::downgrade(&subscriber))
    }

    pub fn delete_subscriber(&self, impl_ref: &Weak<Mutex<SubscriberImpl>>) -> DDSResult<()> {
        let subscriber_impl = impl_ref.upgrade().ok_or(DDSError::AlreadyDeleted)?;
        if subscriber_impl.lock().unwrap().reader_list().is_empty() {
            self.user_defined_entities
                .subscriber_list
                .lock()
                .unwrap()
                .retain(|x| !Arc::ptr_eq(x, &subscriber_impl));
            Ok(())
        } else {
            Err(DDSError::PreconditionNotMet(
                "Subscriber still contains data readers",
            ))
        }
    }

    pub fn create_topic<T: DDSType>(
        &self,
        topic_name: &str,
        qos: Option<TopicQos>,
        a_listener: Option<Box<dyn TopicListener>>,
        mask: StatusMask,
    ) -> DDSResult<Weak<Mutex<TopicImpl>>> {
        // let guid_prefix = self.participant.entity.guid.prefix();
        // let entity_key = [
        //     0,
        //     self.topic_count.fetch_add(1, atomic::Ordering::Relaxed),
        //     0,
        // ];
        // let entity_kind = ENTITY_KIND_USER_DEFINED_UNKNOWN;
        // let entity_id = EntityId::new(entity_key, entity_kind);
        // let guid = GUID::new(guid_prefix, entity_id);
        // let entity = rust_rtps::structure::Entity::new(guid);
        let qos = qos.unwrap_or(self.get_default_topic_qos());
        qos.is_consistent()?;
        let topic = Arc::new(Mutex::new(TopicImpl::new(
            topic_name,
            T::type_name(),
            qos,
            a_listener,
            mask,
        )));

        self.user_defined_entities
            .topic_list
            .lock()
            .unwrap()
            .push(topic.clone());

        Ok(Arc::downgrade(&topic))
    }

    pub fn delete_topic(&self, impl_ref: &Weak<Mutex<TopicImpl>>) -> DDSResult<()> {
        impl_ref.upgrade().ok_or(DDSError::AlreadyDeleted)?; // Just to check if already deleted
        if Weak::strong_count(impl_ref) == 1 {
            self.user_defined_entities
                .topic_list
                .lock()
                .unwrap()
                .retain(|x| !Weak::ptr_eq(&Arc::downgrade(x), &impl_ref));
            Ok(())
        } else {
            Err(DDSError::PreconditionNotMet(
                "Topic still attached to some data reader or data writer",
            ))
        }
    }

    pub fn set_qos(&mut self, qos: Option<DomainParticipantQos>) -> DDSResult<()> {
        let qos = qos.unwrap_or_default();
        self.qos = qos;
        Ok(())
    }

    pub fn get_qos(&self) -> DomainParticipantQos {
        self.qos.clone()
    }

    pub fn set_default_publisher_qos(&mut self, qos: Option<PublisherQos>) -> DDSResult<()> {
        let qos = qos.unwrap_or_default();
        self.default_publisher_qos = qos;
        Ok(())
    }

    pub fn get_default_publisher_qos(&self) -> PublisherQos {
        self.default_publisher_qos.clone()
    }

    pub fn set_default_subscriber_qos(&mut self, qos: Option<SubscriberQos>) -> DDSResult<()> {
        let qos = qos.unwrap_or_default();
        self.default_subscriber_qos = qos;
        Ok(())
    }

    pub fn get_default_subscriber_qos(&self) -> SubscriberQos {
        self.default_subscriber_qos.clone()
    }

    pub fn set_default_topic_qos(&mut self, qos: Option<TopicQos>) -> DDSResult<()> {
        let qos = qos.unwrap_or_default();
        qos.is_consistent()?;
        self.default_topic_qos = qos;
        Ok(())
    }

    pub fn get_default_topic_qos(&self) -> TopicQos {
        self.default_topic_qos.clone()
    }

    pub fn enable(&mut self) -> DDSResult<()> {
        let enabled = self.enabled.clone();
        let mut thread_list = self.thread_list.borrow_mut();
        self.enabled.store(true, atomic::Ordering::Release);

        self.enabled_function.call_once(|| {
            thread_list.push(std::thread::spawn(move || {
                while enabled.load(atomic::Ordering::Acquire) {
                    Self::send();
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            }));
        });
        Ok(())
    }

    fn send() {

        // let writer = Writer::new();
        // let spdp_announcer = StatelessWriter::new(writer);
        // builtin_entities.send_data(guid_prefix);

        //     use rust_rtps::structure::Entity;
        //     let guid_prefix = self.guid().prefix();
        //     let builtin_publisher = RtpsPublisherInner::new_builtin(
        //         guid_prefix,
        //         [0, 0, 0],
        //         PublisherQos::default(),
        //         None,
        //         0,
        //     );

        //     let spdp_topic_qos = TopicQos::default();
        //     let spdp_topic = Arc::new(RtpsTopicInner::new(
        //         guid_prefix,
        //         ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER.entity_key(),
        //         "SPDP".to_string(),
        //         SpdpDiscoveredParticipantData::type_name(),
        //         rust_rtps::types::TopicKind::WithKey,
        //         spdp_topic_qos,
        //         None,
        //         0,
        //     ));
        //     // // let _spdp_topic_ref = self
        //     // //     .builtin_entities
        //     // //     .topic_list
        //     // //     .add(spdp_topic.clone())
        //     // //     .expect("Error creating SPDP topic");

        //     let spdp_unicast_locator_list = self.builtin_entities.transport.unicast_locator_list().clone();
        //     let spdp_multicast_locator_list = self.builtin_entities.transport.multicast_locator_list().clone();
        //     let spdp_resend_period = rust_rtps::behavior::types::Duration::from_secs(30);
        //     let spdp_reader_locators = vec![ReaderLocator::new(Locator::new_udpv4(7400, [239, 255, 0, 0]))];

        //     let mut spdp_announcer_qos = DataWriterQos::default();
        //     spdp_announcer_qos.reliability.kind = rust_dds_api::infrastructure::qos_policy::ReliabilityQosPolicyKind::BestEffortReliabilityQos;
        //     let spdp_announcer = RtpsDataWriterImpl::new::<SpdpDiscoveredParticipantData>(RtpsWriterFlavor::Stateless(SPDPbuiltinParticipantWriter::new(guid_prefix, spdp_unicast_locator_list, spdp_multicast_locator_list, spdp_resend_period, spdp_reader_locators)), &spdp_topic, spdp_announcer_qos, None, 0);

        //     {
        //         let spdp_announcer_ref = builtin_publisher.writer_list().add(spdp_announcer).expect("Error adding SPDP writer to built_in publisher");
        //         spdp_announcer_ref.write_w_timestamp::<SpdpDiscoveredParticipantData>(SpdpDiscoveredParticipantData{value:5}, None, Time{sec:10, nanosec:0}).expect("Error announcing participant");
        //     }

        //     self
        //         .builtin_entities
        //         .publisher_list
        //         .add(Box::new(builtin_publisher))
        //         .expect("Error creating built-in publisher");

        //
        //     let enabled = self.enabled.clone();
        //     let builtin_entities = self.builtin_entities.clone();
    }
}

impl RTPSEntity for DomainParticipantImpl {
    fn guid(&self) -> GUID {
        GUID::new(self.guid_prefix, ENTITYID_PARTICIPANT)
    }
}

impl RTPSParticipant for DomainParticipantImpl {
    fn default_unicast_locator_list(&self) -> &[Locator] {
        todo!()
        // self.user_defined_entities.transport.unicast_locator_list()
    }

    fn default_multicast_locator_list(&self) -> &[Locator] {
        todo!()
        // self.user_defined_entities
        //     .transport
        //     .multicast_locator_list()
    }

    fn protocol_version(&self) -> ProtocolVersion {
        PROTOCOL_VERSION_2_4
    }

    fn vendor_id(&self) -> VendorId {
        VENDOR_ID
    }
}

impl Drop for DomainParticipantImpl {
    fn drop(&mut self) {
        self.enabled.store(false, atomic::Ordering::Release);
        for thread in self.thread_list.borrow_mut().drain(..) {
            thread.join().ok();
        }
    }
}

#[cfg(test)]
mod tests {
    use rust_rtps::types::Locator;

    use crate::transport::Transport;

    use super::*;

    struct TestType;

    impl DDSType for TestType {
        fn type_name() -> &'static str {
            "TestType"
        }

        fn has_key() -> bool {
            todo!()
        }

        fn key(&self) -> Vec<u8> {
            todo!()
        }

        fn serialize(&self) -> Vec<u8> {
            todo!()
        }

        fn deserialize(_data: Vec<u8>) -> Self {
            todo!()
        }
    }

    #[derive(Default)]
    struct MockTransport {
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
    }

    impl Transport for MockTransport {
        fn write(
            &self,
            _message: rust_rtps::messages::RtpsMessage,
            _destination_locator: &Locator,
        ) {
            todo!()
        }

        fn read<'a>(
            &'a self,
        ) -> crate::transport::TransportResult<
            Option<(rust_rtps::messages::RtpsMessage<'a>, Locator)>,
        > {
            todo!()
        }

        fn unicast_locator_list(&self) -> &Vec<Locator> {
            todo!()
        }

        fn multicast_locator_list(&self) -> &Vec<Locator> {
            todo!()
        }
    }

    #[test]
    fn create_publisher() {
        let participant = DomainParticipantImpl::new(
            0,
            DomainParticipantQos::default(),
            Box::new(MockTransport::default()),
            Box::new(MockTransport::default()),
            None,
            0,
            &[],
        );

        let qos = Some(PublisherQos::default());
        let a_listener = None;
        let mask = 0;
        participant
            .create_publisher(qos, a_listener, mask)
            .expect("Error creating publisher");

        assert_eq!(
            participant
                .user_defined_entities
                .publisher_list
                .lock()
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn create_delete_publisher() {
        let participant = DomainParticipantImpl::new(
            0,
            DomainParticipantQos::default(),
            Box::new(MockTransport::default()),
            Box::new(MockTransport::default()),
            None,
            0,
            &[],
        );

        let qos = Some(PublisherQos::default());
        let a_listener = None;
        let mask = 0;
        let a_publisher = participant.create_publisher(qos, a_listener, mask).unwrap();

        participant
            .delete_publisher(&a_publisher)
            .expect("Error deleting publisher");
        assert_eq!(
            participant
                .user_defined_entities
                .publisher_list
                .lock()
                .unwrap()
                .len(),
            0
        );
    }

    #[test]
    fn create_subscriber() {
        let participant = DomainParticipantImpl::new(
            0,
            DomainParticipantQos::default(),
            Box::new(MockTransport::default()),
            Box::new(MockTransport::default()),
            None,
            0,
            &[],
        );

        let qos = Some(SubscriberQos::default());
        let a_listener = None;
        let mask = 0;
        participant
            .create_subscriber(qos, a_listener, mask)
            .expect("Error creating subscriber");
        assert_eq!(
            participant
                .user_defined_entities
                .subscriber_list
                .lock()
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn create_delete_subscriber() {
        let participant = DomainParticipantImpl::new(
            0,
            DomainParticipantQos::default(),
            Box::new(MockTransport::default()),
            Box::new(MockTransport::default()),
            None,
            0,
            &[],
        );

        let qos = Some(SubscriberQos::default());
        let a_listener = None;
        let mask = 0;
        let a_subscriber = participant
            .create_subscriber(qos, a_listener, mask)
            .unwrap();

        participant
            .delete_subscriber(&a_subscriber)
            .expect("Error deleting subscriber");
        assert_eq!(
            participant
                .user_defined_entities
                .subscriber_list
                .lock()
                .unwrap()
                .len(),
            0
        );
    }

    #[test]
    fn create_topic() {
        let participant = DomainParticipantImpl::new(
            0,
            DomainParticipantQos::default(),
            Box::new(MockTransport::default()),
            Box::new(MockTransport::default()),
            None,
            0,
            &[],
        );

        let topic_name = "Test";
        let qos = Some(TopicQos::default());
        let a_listener = None;
        let mask = 0;
        participant
            .create_topic::<TestType>(topic_name, qos, a_listener, mask)
            .expect("Error creating topic");
    }

    #[test]
    fn create_delete_topic() {
        let participant = DomainParticipantImpl::new(
            0,
            DomainParticipantQos::default(),
            Box::new(MockTransport::default()),
            Box::new(MockTransport::default()),
            None,
            0,
            &[],
        );

        let topic_name = "Test";
        let qos = Some(TopicQos::default());
        let a_listener = None;
        let mask = 0;
        let a_topic = participant
            .create_topic::<TestType>(topic_name, qos, a_listener, mask)
            .unwrap();

        participant
            .delete_topic(&a_topic)
            .expect("Error deleting topic")
    }

    #[test]
    fn set_get_default_publisher_qos() {
        let mut participant = DomainParticipantImpl::new(
            0,
            DomainParticipantQos::default(),
            Box::new(MockTransport::default()),
            Box::new(MockTransport::default()),
            None,
            0,
            &[],
        );

        let mut publisher_qos = PublisherQos::default();
        publisher_qos.group_data.value = vec![b'a', b'b', b'c'];
        participant
            .set_default_publisher_qos(Some(publisher_qos.clone()))
            .expect("Error setting default publisher qos");

        assert_eq!(publisher_qos, participant.get_default_publisher_qos())
    }

    #[test]
    fn set_get_default_subscriber_qos() {
        let mut participant = DomainParticipantImpl::new(
            0,
            DomainParticipantQos::default(),
            Box::new(MockTransport::default()),
            Box::new(MockTransport::default()),
            None,
            0,
            &[],
        );

        let mut subscriber_qos = SubscriberQos::default();
        subscriber_qos.group_data.value = vec![b'a', b'b', b'c'];
        participant
            .set_default_subscriber_qos(Some(subscriber_qos.clone()))
            .expect("Error setting default subscriber qos");

        assert_eq!(subscriber_qos, participant.get_default_subscriber_qos())
    }

    #[test]
    fn set_get_default_topic_qos() {
        let mut participant = DomainParticipantImpl::new(
            0,
            DomainParticipantQos::default(),
            Box::new(MockTransport::default()),
            Box::new(MockTransport::default()),
            None,
            0,
            &[],
        );

        let mut topic_qos = TopicQos::default();
        topic_qos.topic_data.value = vec![b'a', b'b', b'c'];
        participant
            .set_default_topic_qos(Some(topic_qos.clone()))
            .expect("Error setting default subscriber qos");

        assert_eq!(topic_qos, participant.get_default_topic_qos())
    }

    #[test]
    fn set_default_publisher_qos_to_default_value() {
        let mut participant = DomainParticipantImpl::new(
            0,
            DomainParticipantQos::default(),
            Box::new(MockTransport::default()),
            Box::new(MockTransport::default()),
            None,
            0,
            &[],
        );

        let mut publisher_qos = PublisherQos::default();
        publisher_qos.group_data.value = vec![b'a', b'b', b'c'];
        participant
            .set_default_publisher_qos(Some(publisher_qos.clone()))
            .unwrap();

        participant
            .set_default_publisher_qos(None)
            .expect("Error setting default publisher qos");

        assert_eq!(
            PublisherQos::default(),
            participant.get_default_publisher_qos()
        )
    }

    #[test]
    fn set_default_subscriber_qos_to_default_value() {
        let mut participant = DomainParticipantImpl::new(
            0,
            DomainParticipantQos::default(),
            Box::new(MockTransport::default()),
            Box::new(MockTransport::default()),
            None,
            0,
            &[],
        );

        let mut subscriber_qos = SubscriberQos::default();
        subscriber_qos.group_data.value = vec![b'a', b'b', b'c'];
        participant
            .set_default_subscriber_qos(Some(subscriber_qos.clone()))
            .unwrap();

        participant
            .set_default_subscriber_qos(None)
            .expect("Error setting default subscriber qos");

        assert_eq!(
            SubscriberQos::default(),
            participant.get_default_subscriber_qos()
        )
    }

    #[test]
    fn set_default_topic_qos_to_default_value() {
        let mut participant = DomainParticipantImpl::new(
            0,
            DomainParticipantQos::default(),
            Box::new(MockTransport::default()),
            Box::new(MockTransport::default()),
            None,
            0,
            &[],
        );

        let mut topic_qos = TopicQos::default();
        topic_qos.topic_data.value = vec![b'a', b'b', b'c'];
        participant
            .set_default_topic_qos(Some(topic_qos.clone()))
            .unwrap();

        participant
            .set_default_topic_qos(None)
            .expect("Error setting default subscriber qos");

        assert_eq!(TopicQos::default(), participant.get_default_topic_qos())
    }

    #[test]
    fn enable() {
        let mut participant = DomainParticipantImpl::new(
            0,
            DomainParticipantQos::default(),
            Box::new(MockTransport::default()),
            Box::new(MockTransport::default()),
            None,
            0,
            &[],
        );

        participant.enable().expect("Failed to enable");
        assert_eq!(participant.thread_list.borrow().len(), 1);
    }

    // #[test]
    // fn create_publisher_factory_default_qos() {
    //     let participant = DomainParticipantImpl::new(
    //         0,
    //         DomainParticipantQos::default(),
    //         MockTransport::default(),
    //         MockTransport::default(),
    //         None,
    //         0,
    //     );

    //     let mut publisher_qos = PublisherQos::default();
    //     publisher_qos.group_data.value = vec![b'a', b'b', b'c'];
    //     participant
    //         .set_default_publisher_qos(Some(publisher_qos.clone()))
    //         .unwrap();

    //     let qos = None;
    //     let a_listener = None;
    //     let mask = 0;
    //     let publisher = participant
    //         .create_publisher(qos, a_listener, mask)
    //         .expect("Error creating publisher");

    //     assert_eq!(publisher.get_qos().unwrap(), publisher_qos);
    // }

    // #[test]
    // fn create_subscriber_factory_default_qos() {
    //     let participant = DomainParticipantImpl::new(
    //         0,
    //         DomainParticipantQos::default(),
    //         MockTransport::default(),
    //         MockTransport::default(),
    //         None,
    //         0,
    //     );

    //     let mut subscriber_qos = SubscriberQos::default();
    //     subscriber_qos.group_data.value = vec![b'a', b'b', b'c'];
    //     participant
    //         .set_default_subscriber_qos(Some(subscriber_qos.clone()))
    //         .unwrap();

    //     let qos = None;
    //     let a_listener = None;
    //     let mask = 0;
    //     let subscriber = participant
    //         .create_subscriber(qos, a_listener, mask)
    //         .expect("Error creating publisher");

    //     assert_eq!(subscriber.get_qos().unwrap(), subscriber_qos);
    // }

    // #[test]
    // fn create_topic_factory_default_qos() {
    //     let participant = DomainParticipantImpl::new(
    //         0,
    //         DomainParticipantQos::default(),
    //         MockTransport::default(),
    //         MockTransport::default(),
    //         None,
    //         0,
    //     );

    //     let mut topic_qos = TopicQos::default();
    //     topic_qos.topic_data.value = vec![b'a', b'b', b'c'];
    //     participant
    //         .set_default_topic_qos(Some(topic_qos.clone()))
    //         .unwrap();

    //     let qos = None;
    //     let a_listener = None;
    //     let mask = 0;
    //     let topic = participant
    //         .create_topic::<TestType>("name", qos, a_listener, mask)
    //         .expect("Error creating publisher");

    //     assert_eq!(topic.get_qos().unwrap(), topic_qos);
    // }
}
