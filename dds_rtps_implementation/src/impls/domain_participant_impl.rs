use std::{
    cell::RefCell,
    sync::{atomic, Arc, Mutex, Once, Weak},
    thread::JoinHandle,
};

use rust_dds_api::{
    builtin_topics::{ParticipantBuiltinTopicData, TopicBuiltinTopicData},
    dcps_psm::{DomainId, Duration, InstanceHandle, StatusMask, Time},
    dds_type::DDSType,
    domain::domain_participant_listener::DomainParticipantListener,
    infrastructure::{
        entity::StatusCondition,
        qos::{DomainParticipantQos, PublisherQos, SubscriberQos, TopicQos},
    },
    publication::publisher_listener::PublisherListener,
    return_type::{DDSError, DDSResult},
    subscription::subscriber_listener::SubscriberListener,
    topic::{topic_description::TopicDescription, topic_listener::TopicListener},
};
use rust_rtps::{
    structure::{Entity, Participant},
    transport::Transport,
    types::{
        constants::{PROTOCOL_VERSION_2_4, VENDOR_ID},
        GuidPrefix, Locator, ProtocolVersion, VendorId, GUID,
    },
};

use super::{
    publisher_impl::PublisherImpl, subscriber_impl::SubscriberImpl, topic_impl::TopicImpl,
};

struct RtpsParticipantEntities {
    publisher_list: Mutex<Vec<Arc<Mutex<PublisherImpl>>>>,
    subscriber_list: Mutex<Vec<Arc<Mutex<SubscriberImpl>>>>,
    topic_list: Mutex<Vec<Arc<Mutex<TopicImpl>>>>,
    transport: Box<dyn Transport>,
}

impl RtpsParticipantEntities {
    fn new(transport: impl Transport) -> Self {
        Self {
            publisher_list: Default::default(),
            subscriber_list: Default::default(),
            topic_list: Default::default(),
            transport: Box::new(transport),
        }
    }

    pub fn send_data(&self, _participant_guid_prefix: GuidPrefix) {
        let _transport = &self.transport;
        let publisher_list = self.publisher_list.lock().unwrap();
        for publisher in publisher_list.iter() {
            for _writer in publisher.lock().unwrap().writer_list() {
                todo!()
                // let destined_messages = writer.lock().unwrap().produce_messages();
                // RtpsMessageSender::send_cache_change_messages(
                //     participant_guid_prefix,
                //     transport.as_ref(),
                //     destined_messages,
                // );
            }
        }
    }
}

pub struct DomainParticipantImpl {
    domain_id: DomainId,
    guid_prefix: GuidPrefix,
    qos: Mutex<DomainParticipantQos>,
    publisher_count: atomic::AtomicU8,
    subscriber_count: atomic::AtomicU8,
    topic_count: atomic::AtomicU8,
    default_publisher_qos: Mutex<PublisherQos>,
    default_subscriber_qos: Mutex<SubscriberQos>,
    default_topic_qos: Mutex<TopicQos>,
    builtin_entities: Arc<RtpsParticipantEntities>,
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
        userdata_transport: impl Transport,
        metatraffic_transport: impl Transport,
        a_listener: Option<Box<dyn DomainParticipantListener>>,
        mask: StatusMask,
    ) -> Self {
        let guid_prefix = [1; 12];

        let builtin_entities = Arc::new(RtpsParticipantEntities::new(metatraffic_transport));
        let user_defined_entities = Arc::new(RtpsParticipantEntities::new(userdata_transport));

        Self {
            domain_id,
            guid_prefix,
            qos: Mutex::new(qos),
            publisher_count: atomic::AtomicU8::new(0),
            subscriber_count: atomic::AtomicU8::new(0),
            topic_count: atomic::AtomicU8::new(0),
            default_publisher_qos: Mutex::new(PublisherQos::default()),
            default_subscriber_qos: Mutex::new(SubscriberQos::default()),
            default_topic_qos: Mutex::new(TopicQos::default()),
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
        // let qos = qos.unwrap_or(self.get_default_publisher_qos());
        // let publisher = Arc::new(Mutex::new(RtpsPublisherImpl::new(qos, a_listener, mask)));

        // self.user_defined_entities
        //     .publisher_list
        //     .lock()
        //     .unwrap()
        //     .push(publisher.clone());
        todo!()
    }

    pub fn delete_publisher(&self, impl_ref: &Weak<Mutex<PublisherImpl>>) -> DDSResult<()> {
        let publisher_impl = impl_ref.upgrade().ok_or(DDSError::AlreadyDeleted)?;
        if publisher_impl.lock().unwrap().writer_list().is_empty() {
            self.user_defined_entities
                .publisher_list
                .lock()
                .unwrap()
                .retain(|x| !Arc::ptr_eq(x, &publisher_impl));
            Ok(())
        } else {
            Err(DDSError::PreconditionNotMet(
                "Publisher still contains data writers",
            ))
        }
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
        // let qos = qos.unwrap_or(self.get_default_subscriber_qos());
        // let subscriber = Arc::new(Mutex::new(RtpsSubscriberImpl::new(qos, a_listener, mask)));

        // self.user_defined_entities
        //     .subscriber_list
        //     .lock()
        //     .unwrap()
        //     .push(subscriber.clone());

        todo!()
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

    pub fn create_topic<T:DDSType>(
        &self,
        topic_name: &str,
        qos: Option<TopicQos>,
        a_listener: Option<Box<dyn TopicListener>>,
        mask: StatusMask,
    ) -> DDSResult<Weak<Mutex<TopicImpl>>> {
        todo!()
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
        // let qos = qos.unwrap_or(self.get_default_topic_qos());
        // qos.is_consistent().ok()?;
        // let topic = Arc::new(Mutex::new(RtpsTopicImpl::new(
        //     topic_name,
        //     T::type_name(),
        //     qos,
        //     a_listener,
        //     mask,
        // )));

        // self.user_defined_entities
        //     .topic_list
        //     .lock()
        //     .unwrap()
        //     .push(topic.clone());
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
}

impl Entity for DomainParticipantImpl {
    fn guid(&self) -> GUID {
        todo!()
    }
}

impl Participant for DomainParticipantImpl {
    fn default_unicast_locator_list(&self) -> &[Locator] {
        todo!()
    }

    fn default_multicast_locator_list(&self) -> &[Locator] {
        todo!()
    }

    fn protocol_version(&self) -> ProtocolVersion {
        PROTOCOL_VERSION_2_4
    }

    fn vendor_id(&self) -> VendorId {
        VENDOR_ID
    }
}
