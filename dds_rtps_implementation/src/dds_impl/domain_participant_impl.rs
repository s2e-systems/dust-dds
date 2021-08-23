use std::{
    sync::{
        atomic::{self, AtomicBool},
        Arc,
    },
    thread::JoinHandle,
};

use rust_dds_api::{
    builtin_topics::{ParticipantBuiltinTopicData, TopicBuiltinTopicData},
    dcps_psm::{DomainId, Duration, InstanceHandle, StatusMask, Time},
    domain::domain_participant_listener::DomainParticipantListener,
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
    types::{EntityId, EntityKind, Guid},
    RtpsEntity,
};

use crate::{
    rtps_impl::{rtps_group_impl::RtpsGroupImpl, rtps_participant_impl::RtpsParticipantImpl},
    utils::{
        shared_object::RtpsShared,
        transport::{TransportRead, TransportWrite},
    },
};

use super::{
    publisher_impl::{PublisherImpl, PublisherStorage},
    subscriber_impl::{SubscriberImpl, SubscriberStorage},
    topic_impl::{TopicImpl, TopicStorage},
};

pub trait Transport: TransportRead + TransportWrite + Send {}

impl<T> Transport for T where T: TransportRead + TransportWrite + Send {}

pub struct DomainParticipantStorage {
    rtps_participant: RtpsParticipantImpl,
    domain_participant_qos: DomainParticipantQos,
    builtin_subscriber_storage: Vec<RtpsShared<SubscriberStorage>>,
    builtin_publisher_storage: Vec<RtpsShared<PublisherStorage>>,
    user_defined_subscriber_storage: Vec<RtpsShared<SubscriberStorage>>,
    user_defined_subscriber_counter: u8,
    default_subscriber_qos: SubscriberQos,
    user_defined_publisher_storage: Vec<RtpsShared<PublisherStorage>>,
    user_defined_publisher_counter: u8,
    default_publisher_qos: PublisherQos,
    topic_storage: Vec<RtpsShared<TopicStorage>>,
    default_topic_qos: TopicQos,
    metatraffic_transport: Box<dyn Transport>,
    default_transport: Box<dyn Transport>,
}

impl DomainParticipantStorage {
    pub fn new(
        domain_participant_qos: DomainParticipantQos,
        rtps_participant: RtpsParticipantImpl,
        builtin_subscriber_storage: Vec<RtpsShared<SubscriberStorage>>,
        builtin_publisher_storage: Vec<RtpsShared<PublisherStorage>>,
        metatraffic_transport: Box<dyn Transport>,
        default_transport: Box<dyn Transport>,
    ) -> Self {
        Self {
            rtps_participant,
            domain_participant_qos,
            builtin_subscriber_storage,
            builtin_publisher_storage,
            metatraffic_transport,
            default_transport,
            user_defined_subscriber_storage: Vec::new(),
            user_defined_subscriber_counter: 0,
            default_subscriber_qos: SubscriberQos::default(),
            user_defined_publisher_storage: Vec::new(),
            user_defined_publisher_counter: 0,
            default_publisher_qos: PublisherQos::default(),
            topic_storage: Vec::new(),
            default_topic_qos: TopicQos::default(),
        }
    }

    pub fn send_builtin_data(&mut self) {
        for publisher in &self.builtin_publisher_storage {
            let publisher_lock = publisher.lock();
            for data_writer in publisher_lock.data_writer_storage_list() {
                let mut data_writer_lock = data_writer.lock();
                crate::utils::message_sender::send_data(
                    &self.rtps_participant,
                    &mut data_writer_lock.rtps_data_writer_mut(),
                    &mut *self.metatraffic_transport,
                );
            }
        }
    }

    pub fn send_user_defined_data(&mut self) {
        for publisher in &self.user_defined_publisher_storage {
            let publisher_lock = publisher.lock();
            for data_writer in publisher_lock.data_writer_storage_list() {
                let mut data_writer_lock = data_writer.lock();
                crate::utils::message_sender::send_data(
                    &self.rtps_participant,
                    &mut data_writer_lock.rtps_data_writer_mut(),
                    &mut *self.default_transport,
                );
            }
        }
    }

    pub fn receive_builtin_data(&mut self) {
        if let Some((source_locator, message)) = self.metatraffic_transport.read() {
            crate::utils::message_receiver::MessageReceiver::new().process_message(
                *self.rtps_participant.guid().prefix(),
                &self.builtin_subscriber_storage,
                source_locator,
                &message,
            );
        }
    }

    pub fn receive_user_defined_data(&mut self) {
        if let Some((source_locator, message)) = self.default_transport.read() {
            crate::utils::message_receiver::MessageReceiver::new().process_message(
                *self.rtps_participant.guid().prefix(),
                &self.user_defined_subscriber_storage,
                source_locator,
                &message,
            );
        }
    }

    /// Get a reference to the domain participant storage's builtin subscriber storage.
    pub fn builtin_subscriber_storage(&self) -> &[RtpsShared<SubscriberStorage>] {
        &self.builtin_subscriber_storage
    }

    /// Get a reference to the domain participant storage's rtps participant.
    pub fn rtps_participant(&self) -> &RtpsParticipantImpl {
        &self.rtps_participant
    }

    /// Get a reference to the domain participant storage's builtin publisher storage.
    pub fn builtin_publisher_storage(&self) -> &[RtpsShared<PublisherStorage>] {
        &self.builtin_publisher_storage
    }

    /// Get a reference to the domain participant storage's user defined subscriber storage.
    pub fn user_defined_subscriber_storage(&self) -> &[RtpsShared<SubscriberStorage>] {
        self.user_defined_subscriber_storage.as_slice()
    }

    /// Get a reference to the domain participant storage's user defined publisher storage.
    pub fn user_defined_publisher_storage(&self) -> &[RtpsShared<PublisherStorage>] {
        self.user_defined_publisher_storage.as_slice()
    }
}

pub struct DomainParticipantImpl {
    is_enabled: Arc<AtomicBool>,
    domain_participant_storage: RtpsShared<DomainParticipantStorage>,
    _worker_threads: Vec<JoinHandle<()>>,
}

impl DomainParticipantImpl {
    pub fn new(domain_participant_storage: RtpsShared<DomainParticipantStorage>) -> Self {
        Self {
            is_enabled: Arc::new(AtomicBool::new(false)),
            domain_participant_storage,
            _worker_threads: Vec::new(),
        }
    }
}

impl<'p> rust_dds_api::domain::domain_participant::PublisherFactory<'p> for DomainParticipantImpl {
    type PublisherType = PublisherImpl<'p>;
    fn create_publisher(
        &'p self,
        qos: Option<PublisherQos>,
        _a_listener: Option<&'static dyn PublisherListener>,
        _mask: StatusMask,
    ) -> Option<Self::PublisherType> {
        let mut domain_participant_lock = self.domain_participant_storage.lock();
        let publisher_qos = qos.unwrap_or(domain_participant_lock.default_publisher_qos.clone());
        domain_participant_lock.user_defined_publisher_counter += 1;
        let entity_id = EntityId::new(
            [domain_participant_lock.user_defined_publisher_counter, 0, 0],
            EntityKind::UserDefinedWriterGroup,
        );
        let guid = Guid::new(
            *domain_participant_lock.rtps_participant.guid().prefix(),
            entity_id,
        );
        let rtps_group = RtpsGroupImpl::new(guid);
        let data_writer_storage_list = Vec::new();
        let publisher_storage =
            PublisherStorage::new(publisher_qos, rtps_group, data_writer_storage_list);
        let publisher_storage_shared = RtpsShared::new(publisher_storage);
        let publisher = PublisherImpl::new(self, publisher_storage_shared.downgrade());
        domain_participant_lock
            .user_defined_publisher_storage
            .push(publisher_storage_shared);
        Some(publisher)
    }

    fn delete_publisher(&self, a_publisher: &Self::PublisherType) -> DDSResult<()> {
        if std::ptr::eq(a_publisher.get_participant(), self) {
            let publisher_storage = a_publisher.publisher_storage().upgrade()?;
            let mut domain_participant_lock = self.domain_participant_storage.lock();
            domain_participant_lock
                .user_defined_publisher_storage
                .retain(|x| x != &publisher_storage);
            Ok(())
        } else {
            Err(DDSError::PreconditionNotMet(
                "Publisher can only be deleted from its parent participant",
            ))
        }
    }
}

impl<'s> rust_dds_api::domain::domain_participant::SubscriberFactory<'s> for DomainParticipantImpl {
    type SubscriberType = SubscriberImpl<'s>;

    fn create_subscriber(
        &'s self,
        qos: Option<SubscriberQos>,
        _a_listener: Option<&'static dyn SubscriberListener>,
        _mask: StatusMask,
    ) -> Option<Self::SubscriberType> {
        let mut domain_participant_lock = self.domain_participant_storage.lock();
        let subscriber_qos = qos.unwrap_or(domain_participant_lock.default_subscriber_qos.clone());
        domain_participant_lock.user_defined_subscriber_counter += 1;
        let entity_id = EntityId::new(
            [
                domain_participant_lock.user_defined_subscriber_counter,
                0,
                0,
            ],
            EntityKind::UserDefinedWriterGroup,
        );
        let guid = Guid::new(
            *domain_participant_lock.rtps_participant.guid().prefix(),
            entity_id,
        );
        let rtps_group = RtpsGroupImpl::new(guid);
        let data_reader_storage_list = Vec::new();
        let subscriber_storage =
            SubscriberStorage::new(subscriber_qos, rtps_group, data_reader_storage_list);
        let subscriber_storage_shared = RtpsShared::new(subscriber_storage);
        let subscriber = SubscriberImpl::new(self, subscriber_storage_shared.downgrade());
        domain_participant_lock
            .user_defined_subscriber_storage
            .push(subscriber_storage_shared);
        Some(subscriber)
    }

    fn delete_subscriber(&self, a_subscriber: &Self::SubscriberType) -> DDSResult<()> {
        if std::ptr::eq(a_subscriber.get_participant(), self) {
            let subscriber_storage = a_subscriber.subscriber_storage().upgrade()?;
            let mut domain_participant_lock = self.domain_participant_storage.lock();
            domain_participant_lock
                .user_defined_subscriber_storage
                .retain(|x| x != &subscriber_storage);
            Ok(())
        } else {
            Err(DDSError::PreconditionNotMet(
                "Subscriber can only be deleted from its parent participant",
            ))
        }
    }

    fn get_builtin_subscriber(&'s self) -> Self::SubscriberType {
        let domain_participant_lock = self.domain_participant_storage.lock();
        let subscriber_storage_shared =
            domain_participant_lock.builtin_subscriber_storage[0].clone();
        SubscriberImpl::new(self, subscriber_storage_shared.downgrade())
    }
}

impl<'t, T: 'static> rust_dds_api::domain::domain_participant::TopicFactory<'t, T>
    for DomainParticipantImpl
{
    type TopicType = TopicImpl<'t, T>;

    fn create_topic(
        &'t self,
        _topic_name: &str,
        qos: Option<TopicQos>,
        _a_listener: Option<&'static dyn TopicListener<DataPIM = T>>,
        _mask: StatusMask,
    ) -> Option<Self::TopicType> {
        let topic_qos = qos.unwrap_or(
            self.domain_participant_storage
                .lock()
                .default_topic_qos
                .clone(),
        );
        let topic_storage = TopicStorage::new(topic_qos);
        let topic_storage_shared = RtpsShared::new(topic_storage);
        let topic = TopicImpl::new(self, topic_storage_shared.downgrade());
        self.domain_participant_storage
            .lock()
            .topic_storage
            .push(topic_storage_shared);
        Some(topic)
    }

    fn delete_topic(&self, _a_topic: &Self::TopicType) -> DDSResult<()> {
        todo!()
    }

    fn find_topic(&self, _topic_name: &str, _timeout: Duration) -> Option<Self::TopicType> {
        todo!()
    }
}

impl rust_dds_api::domain::domain_participant::DomainParticipant for DomainParticipantImpl {
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

    fn set_default_publisher_qos(&self, qos: Option<PublisherQos>) -> DDSResult<()> {
        self.domain_participant_storage.lock().default_publisher_qos = qos.unwrap_or_default();
        Ok(())
    }

    fn get_default_publisher_qos(&self) -> PublisherQos {
        self.domain_participant_storage
            .lock()
            .default_publisher_qos
            .clone()
    }

    fn set_default_subscriber_qos(&self, qos: Option<SubscriberQos>) -> DDSResult<()> {
        self.domain_participant_storage
            .lock()
            .default_subscriber_qos = qos.unwrap_or_default();
        Ok(())
    }

    fn get_default_subscriber_qos(&self) -> SubscriberQos {
        self.domain_participant_storage
            .lock()
            .default_subscriber_qos
            .clone()
    }

    fn set_default_topic_qos(&self, qos: Option<TopicQos>) -> DDSResult<()> {
        let topic_qos = qos.unwrap_or_default();
        topic_qos.is_consistent()?;
        self.domain_participant_storage.lock().default_topic_qos = topic_qos;
        Ok(())
    }

    fn get_default_topic_qos(&self) -> TopicQos {
        self.domain_participant_storage
            .lock()
            .default_topic_qos
            .clone()
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

    fn set_qos(&self, qos: Option<Self::Qos>) -> DDSResult<()> {
        self.domain_participant_storage
            .lock()
            .domain_participant_qos = qos.unwrap_or_default();
        Ok(())
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        Ok(self
            .domain_participant_storage
            .lock()
            .domain_participant_qos
            .clone())
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

    fn get_statuscondition(&self) -> StatusCondition {
        todo!()
    }

    fn get_status_changes(&self) -> StatusMask {
        todo!()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        todo!()
        // Ok(crate::utils::instance_handle_from_guid(
        //     &self.rtps_participant_impl.lock().guid(),
        // ))
    }

    fn enable(&self) -> DDSResult<()> {
        self.is_enabled.store(true, atomic::Ordering::Release);
        let is_enabled = self.is_enabled.clone();
        let domain_participant_storage = self.domain_participant_storage.clone();
        std::thread::spawn(move || {
            while is_enabled.load(atomic::Ordering::Relaxed) {
                domain_participant_storage.lock().send_builtin_data();
                domain_participant_storage.lock().receive_builtin_data();
                domain_participant_storage.lock().send_user_defined_data();
                domain_participant_storage
                    .lock()
                    .receive_user_defined_data();
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rust_dds_api::domain::domain_participant::DomainParticipant;

    struct MockDDSType;

    struct MockTransport;

    impl TransportRead for MockTransport {
        fn read(&mut self) -> Option<(rust_rtps_pim::structure::types::Locator, rust_rtps_pim::messages::RtpsMessage<Vec<rust_rtps_pim::messages::submessages::RtpsSubmessageType<'_, Vec<rust_rtps_pim::structure::types::SequenceNumber>, Vec<rust_rtps_pim::messages::submessage_elements::Parameter<'_>>, (), ()>>>)> {
        todo!()
    }
    }

    impl TransportWrite for MockTransport {
        fn write(
            &mut self,
            _message: &crate::utils::transport::TransportMessage<'_>,
            _destination_locator: &rust_rtps_pim::structure::types::Locator,
        ) {
            todo!()
        }
    }

    #[test]
    fn set_default_publisher_qos_some_value() {
        let rtps_participant = RtpsParticipantImpl::new([1; 12]);
        let domain_participant_storage = DomainParticipantStorage::new(
            DomainParticipantQos::default(),
            rtps_participant,
            vec![],
            vec![],
            Box::new(MockTransport),
            Box::new(MockTransport),
        );
        let domain_participant_impl =
            DomainParticipantImpl::new(RtpsShared::new(domain_participant_storage));
        let mut qos = PublisherQos::default();
        qos.group_data.value = &[1, 2, 3, 4];
        domain_participant_impl
            .set_default_publisher_qos(Some(qos.clone()))
            .unwrap();
        assert!(domain_participant_impl.get_default_publisher_qos() == qos);
    }

    #[test]
    fn set_default_publisher_qos_none() {
        let rtps_participant = RtpsParticipantImpl::new([1; 12]);
        let domain_participant_storage = DomainParticipantStorage::new(
            DomainParticipantQos::default(),
            rtps_participant,
            vec![],
            vec![],
            Box::new(MockTransport),
            Box::new(MockTransport),
        );
        let domain_participant_impl =
            DomainParticipantImpl::new(RtpsShared::new(domain_participant_storage));
        let mut qos = PublisherQos::default();
        qos.group_data.value = &[1, 2, 3, 4];
        domain_participant_impl
            .set_default_publisher_qos(Some(qos.clone()))
            .unwrap();

        domain_participant_impl
            .set_default_publisher_qos(None)
            .unwrap();
        assert!(domain_participant_impl.get_default_publisher_qos() == PublisherQos::default());
    }

    #[test]
    fn set_default_subscriber_qos_some_value() {
        let rtps_participant = RtpsParticipantImpl::new([1; 12]);
        let domain_participant_storage = DomainParticipantStorage::new(
            DomainParticipantQos::default(),
            rtps_participant,
            vec![],
            vec![],
            Box::new(MockTransport),
            Box::new(MockTransport),
        );
        let domain_participant_impl =
            DomainParticipantImpl::new(RtpsShared::new(domain_participant_storage));
        let mut qos = SubscriberQos::default();
        qos.group_data.value = &[1, 2, 3, 4];
        domain_participant_impl
            .set_default_subscriber_qos(Some(qos.clone()))
            .unwrap();
        assert_eq!(domain_participant_impl.get_default_subscriber_qos(), qos);
    }

    #[test]
    fn set_default_subscriber_qos_none() {
        let rtps_participant = RtpsParticipantImpl::new([1; 12]);
        let domain_participant_storage = DomainParticipantStorage::new(
            DomainParticipantQos::default(),
            rtps_participant,
            vec![],
            vec![],
            Box::new(MockTransport),
            Box::new(MockTransport),
        );
        let domain_participant_impl =
            DomainParticipantImpl::new(RtpsShared::new(domain_participant_storage));
        let mut qos = SubscriberQos::default();
        qos.group_data.value = &[1, 2, 3, 4];
        domain_participant_impl
            .set_default_subscriber_qos(Some(qos.clone()))
            .unwrap();

        domain_participant_impl
            .set_default_subscriber_qos(None)
            .unwrap();
        assert_eq!(
            domain_participant_impl.get_default_subscriber_qos(),
            SubscriberQos::default()
        );
    }

    #[test]
    fn set_default_topic_qos_some_value() {
        let rtps_participant = RtpsParticipantImpl::new([1; 12]);
        let domain_participant_storage = DomainParticipantStorage::new(
            DomainParticipantQos::default(),
            rtps_participant,
            vec![],
            vec![],
            Box::new(MockTransport),
            Box::new(MockTransport),
        );
        let domain_participant_impl =
            DomainParticipantImpl::new(RtpsShared::new(domain_participant_storage));
        let mut qos = TopicQos::default();
        qos.topic_data.value = &[1, 2, 3, 4];
        domain_participant_impl
            .set_default_topic_qos(Some(qos.clone()))
            .unwrap();
        assert_eq!(domain_participant_impl.get_default_topic_qos(), qos);
    }

    #[test]
    fn set_default_topic_qos_inconsistent() {
        let rtps_participant = RtpsParticipantImpl::new([1; 12]);
        let domain_participant_storage = DomainParticipantStorage::new(
            DomainParticipantQos::default(),
            rtps_participant,
            vec![],
            vec![],
            Box::new(MockTransport),
            Box::new(MockTransport),
        );
        let domain_participant_impl =
            DomainParticipantImpl::new(RtpsShared::new(domain_participant_storage));
        let mut qos = TopicQos::default();
        qos.resource_limits.max_samples_per_instance = 2;
        qos.resource_limits.max_samples = 1;
        let set_default_topic_qos_result =
            domain_participant_impl.set_default_topic_qos(Some(qos.clone()));
        assert!(set_default_topic_qos_result == Err(DDSError::InconsistentPolicy));
    }

    #[test]
    fn set_default_topic_qos_none() {
        let rtps_participant = RtpsParticipantImpl::new([1; 12]);
        let domain_participant_storage = DomainParticipantStorage::new(
            DomainParticipantQos::default(),
            rtps_participant,
            vec![],
            vec![],
            Box::new(MockTransport),
            Box::new(MockTransport),
        );
        let domain_participant_impl =
            DomainParticipantImpl::new(RtpsShared::new(domain_participant_storage));
        let mut qos = TopicQos::default();
        qos.topic_data.value = &[1, 2, 3, 4];
        domain_participant_impl
            .set_default_topic_qos(Some(qos.clone()))
            .unwrap();

        domain_participant_impl.set_default_topic_qos(None).unwrap();
        assert_eq!(
            domain_participant_impl.get_default_topic_qos(),
            TopicQos::default()
        );
    }

    #[test]
    fn create_publisher() {
        let rtps_participant = RtpsParticipantImpl::new([1; 12]);
        let domain_participant_storage = DomainParticipantStorage::new(
            DomainParticipantQos::default(),
            rtps_participant,
            vec![],
            vec![],
            Box::new(MockTransport),
            Box::new(MockTransport),
        );
        let domain_participant_impl =
            DomainParticipantImpl::new(RtpsShared::new(domain_participant_storage));
        let publisher = domain_participant_impl.create_publisher(None, None, 0);

        assert!(publisher.is_some())
    }

    #[test]
    fn create_subscriber() {
        let rtps_participant = RtpsParticipantImpl::new([1; 12]);
        let domain_participant_storage = DomainParticipantStorage::new(
            DomainParticipantQos::default(),
            rtps_participant,
            vec![],
            vec![],
            Box::new(MockTransport),
            Box::new(MockTransport),
        );
        let domain_participant_impl =
            DomainParticipantImpl::new(RtpsShared::new(domain_participant_storage));
        let subscriber = domain_participant_impl.create_subscriber(None, None, 0);

        assert!(subscriber.is_some())
    }

    #[test]
    fn create_topic() {
        let rtps_participant = RtpsParticipantImpl::new([1; 12]);
        let domain_participant_storage = DomainParticipantStorage::new(
            DomainParticipantQos::default(),
            rtps_participant,
            vec![],
            vec![],
            Box::new(MockTransport),
            Box::new(MockTransport),
        );
        let domain_participant_impl =
            DomainParticipantImpl::new(RtpsShared::new(domain_participant_storage));
        let topic =
            domain_participant_impl.create_topic::<MockDDSType>("topic_name", None, None, 0);
        assert!(topic.is_some());
    }
}
