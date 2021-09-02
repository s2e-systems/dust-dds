use rust_dds_api::{
    builtin_topics::{ParticipantBuiltinTopicData, TopicBuiltinTopicData},
    dcps_psm::{DomainId, Duration, InstanceHandle, StatusMask, Time},
    domain::domain_participant_listener::DomainParticipantListener,
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::{DomainParticipantQos, PublisherQos, SubscriberQos, TopicQos},
    },
    publication::publisher_listener::PublisherListener,
    return_type::DDSResult,
    subscription::subscriber_listener::SubscriberListener,
    topic::{topic_description::TopicDescription, topic_listener::TopicListener},
};
use rust_rtps_pim::structure::{
    types::{EntityId, EntityKind, Guid},
    RtpsEntity,
};

use crate::{
    rtps_impl::{rtps_group_impl::RtpsGroupImpl, rtps_participant_impl::RtpsParticipantImpl},
    utils::{
        shared_object::{RtpsShared, RtpsWeak},
        transport::{TransportRead, TransportWrite},
    },
};

use super::{
    publisher_impl::PublisherImpl, publisher_proxy::PublisherProxy,
    subscriber_impl::SubscriberImpl, subscriber_proxy::SubscriberProxy, topic_impl::TopicImpl,
    topic_proxy::TopicProxy,
};

pub trait Transport: TransportRead + TransportWrite + Send {}

impl<T> Transport for T where T: TransportRead + TransportWrite + Send {}

pub struct DomainParticipantImpl {
    rtps_participant: RtpsParticipantImpl,
    qos: DomainParticipantQos,
    builtin_subscriber_storage: Vec<RtpsShared<SubscriberImpl>>,
    builtin_publisher_storage: Vec<RtpsShared<PublisherImpl>>,
    user_defined_subscriber_storage: Vec<RtpsShared<SubscriberImpl>>,
    user_defined_subscriber_counter: u8,
    default_subscriber_qos: SubscriberQos,
    user_defined_publisher_storage: Vec<RtpsShared<PublisherImpl>>,
    user_defined_publisher_counter: u8,
    default_publisher_qos: PublisherQos,
    topic_storage: Vec<RtpsShared<TopicImpl>>,
    default_topic_qos: TopicQos,
    metatraffic_transport: Box<dyn Transport>,
    default_transport: Box<dyn Transport>,
}

impl DomainParticipantImpl {
    pub fn new(
        domain_participant_qos: DomainParticipantQos,
        rtps_participant: RtpsParticipantImpl,
        builtin_subscriber_storage: Vec<RtpsShared<SubscriberImpl>>,
        builtin_publisher_storage: Vec<RtpsShared<PublisherImpl>>,
        metatraffic_transport: Box<dyn Transport>,
        default_transport: Box<dyn Transport>,
    ) -> Self {
        Self {
            rtps_participant,
            qos: domain_participant_qos,
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

    pub fn create_publisher(
        &mut self,
        qos: Option<PublisherQos>,
        _a_listener: Option<&'static dyn PublisherListener>,
        _mask: StatusMask,
    ) -> Option<RtpsWeak<PublisherImpl>> {
        let publisher_qos = qos.unwrap_or(self.default_publisher_qos.clone());
        self.user_defined_publisher_counter += 1;
        let entity_id = EntityId::new(
            [self.user_defined_publisher_counter, 0, 0],
            EntityKind::UserDefinedWriterGroup,
        );
        let guid = Guid::new(*self.rtps_participant.guid().prefix(), entity_id);
        let rtps_group = RtpsGroupImpl::new(guid);
        let data_writer_storage_list = Vec::new();
        let publisher_storage =
            PublisherImpl::new(publisher_qos, rtps_group, data_writer_storage_list);
        let publisher_storage_shared = RtpsShared::new(publisher_storage);
        let publisher_storage_weak = publisher_storage_shared.downgrade();
        self.user_defined_publisher_storage
            .push(publisher_storage_shared);
        Some(publisher_storage_weak)
    }

    pub fn delete_publisher(&mut self, a_publisher: &RtpsWeak<PublisherImpl>) -> DDSResult<()> {
        let publisher_storage = a_publisher.upgrade()?;
        self.user_defined_publisher_storage
            .retain(|x| x != &publisher_storage);
        Ok(())
    }

    pub fn create_subscriber(
        &mut self,
        qos: Option<SubscriberQos>,
        _a_listener: Option<&'static dyn SubscriberListener>,
        _mask: StatusMask,
    ) -> Option<RtpsWeak<SubscriberImpl>> {
        let subscriber_qos = qos.unwrap_or(self.default_subscriber_qos.clone());
        self.user_defined_subscriber_counter += 1;
        let entity_id = EntityId::new(
            [self.user_defined_subscriber_counter, 0, 0],
            EntityKind::UserDefinedWriterGroup,
        );
        let guid = Guid::new(*self.rtps_participant.guid().prefix(), entity_id);
        let rtps_group = RtpsGroupImpl::new(guid);
        let data_reader_storage_list = Vec::new();
        let subscriber_storage =
            SubscriberImpl::new(subscriber_qos, rtps_group, data_reader_storage_list);
        let subscriber_storage_shared = RtpsShared::new(subscriber_storage);
        let subscriber_storage_weak = subscriber_storage_shared.downgrade();
        self.user_defined_subscriber_storage
            .push(subscriber_storage_shared);
        Some(subscriber_storage_weak)
    }

    pub fn delete_subscriber(&mut self, a_subscriber: &RtpsWeak<SubscriberImpl>) -> DDSResult<()> {
        let subscriber_storage = a_subscriber.upgrade()?;
        self.user_defined_subscriber_storage
            .retain(|x| x != &subscriber_storage);
        Ok(())
    }

    pub fn get_builtin_subscriber(&self) -> RtpsWeak<SubscriberImpl> {
        self.builtin_subscriber_storage[0].clone().downgrade()
    }

    pub fn create_topic<T>(
        &mut self,
        _topic_name: &str,
        qos: Option<TopicQos>,
        _a_listener: Option<&'static dyn TopicListener<DataPIM = T>>,
        _mask: StatusMask,
    ) -> Option<RtpsWeak<TopicImpl>> {
        let topic_qos = qos.unwrap_or(self.default_topic_qos.clone());
        let topic_storage = TopicImpl::new(topic_qos);
        let topic_storage_shared = RtpsShared::new(topic_storage);
        let topic_storage_weak = topic_storage_shared.downgrade();
        self.topic_storage.push(topic_storage_shared);
        Some(topic_storage_weak)
    }

    pub fn set_default_publisher_qos(&mut self, qos: Option<PublisherQos>) -> DDSResult<()> {
        self.default_publisher_qos = qos.unwrap_or_default();
        Ok(())
    }

    pub fn get_default_publisher_qos(&self) -> &PublisherQos {
        &self.default_publisher_qos
    }

    pub fn set_default_subscriber_qos(&mut self, qos: Option<SubscriberQos>) -> DDSResult<()> {
        self.default_subscriber_qos = qos.unwrap_or_default();
        Ok(())
    }

    pub fn get_default_subscriber_qos(&self) -> &SubscriberQos {
        &self.default_subscriber_qos
    }

    pub fn set_default_topic_qos(&mut self, qos: Option<TopicQos>) -> DDSResult<()> {
        let topic_qos = qos.unwrap_or_default();
        topic_qos.is_consistent()?;
        self.default_topic_qos = topic_qos;
        Ok(())
    }

    pub fn get_default_topic_qos(&self) -> &TopicQos {
        &self.default_topic_qos
    }

    pub fn set_qos(&mut self, qos: Option<DomainParticipantQos>) -> DDSResult<()> {
        self.qos = qos.unwrap_or_default();
        Ok(())
    }

    pub fn get_qos(&self) -> &DomainParticipantQos {
        &self.qos
    }

    pub fn send_builtin_data(&mut self) {
        // for publisher in &self.builtin_publisher_storage {
        //     let publisher_lock = publisher.lock();
        //     for data_writer in publisher_lock.data_writer_storage_list() {
        //         let mut data_writer_lock = data_writer.lock();
        //         crate::utils::message_sender::send_data(
        //             &self.rtps_participant,
        //             data_writer_lock.rtps_data_writer_mut(),
        //             &mut *self.metatraffic_transport,
        //         );
        //     }
        // }
        todo!()
    }

    pub fn send_user_defined_data(&mut self) {
        todo!()
        // for publisher in &self.user_defined_publisher_storage {
        //     let publisher_lock = publisher.lock();
        //     for data_writer in publisher_lock.data_writer_storage_list() {
        //         let mut data_writer_lock = data_writer.lock();
        //         crate::utils::message_sender::send_data(
        //             &self.rtps_participant,
        //             data_writer_lock.rtps_data_writer_mut(),
        //             &mut *self.default_transport,
        //         );
        //     }
        // }
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
    pub fn builtin_subscriber_storage(&self) -> &[RtpsShared<SubscriberImpl>] {
        &self.builtin_subscriber_storage
    }

    /// Get a reference to the domain participant storage's rtps participant.
    pub fn rtps_participant(&self) -> &RtpsParticipantImpl {
        &self.rtps_participant
    }

    /// Get a reference to the domain participant storage's builtin publisher storage.
    pub fn builtin_publisher_storage(&self) -> &[RtpsShared<PublisherImpl>] {
        &self.builtin_publisher_storage
    }

    /// Get a reference to the domain participant storage's user defined subscriber storage.
    pub fn user_defined_subscriber_storage(&self) -> &[RtpsShared<SubscriberImpl>] {
        self.user_defined_subscriber_storage.as_slice()
    }

    /// Get a reference to the domain participant storage's user defined publisher storage.
    pub fn user_defined_publisher_storage(&self) -> &[RtpsShared<PublisherImpl>] {
        self.user_defined_publisher_storage.as_slice()
    }
}

impl<'p> rust_dds_api::domain::domain_participant::PublisherGAT<'p> for DomainParticipantImpl {
    type PublisherType = PublisherProxy<'p>;
    fn create_publisher_gat(
        &'p self,
        _qos: Option<PublisherQos>,
        _a_listener: Option<&'static dyn PublisherListener>,
        _mask: StatusMask,
    ) -> Option<Self::PublisherType> {
        // let publisher_storage_weak = self
        //     .domain_participant_storage
        //     .lock()
        //     .create_publisher(qos, a_listener, mask)?;
        // let publisher = PublisherProxy::new(self, publisher_storage_weak);

        // Some(publisher)
        todo!()
    }

    fn delete_publisher_gat(&self, _a_publisher: &Self::PublisherType) -> DDSResult<()> {
        // if std::ptr::eq(a_publisher.get_participant(), self) {
        //     self.domain_participant_storage
        //         .lock()
        //         .delete_publisher(a_publisher.publisher_storage())
        // } else {
        //     Err(DDSError::PreconditionNotMet(
        //         "Publisher can only be deleted from its parent participant",
        //     ))
        // }
        todo!()
    }
}

impl<'s> rust_dds_api::domain::domain_participant::SubscriberGAT<'s> for DomainParticipantImpl {
    type SubscriberType = SubscriberProxy<'s>;

    fn create_subscriber_gat(
        &'s self,
        _qos: Option<SubscriberQos>,
        _a_listener: Option<&'static dyn SubscriberListener>,
        _mask: StatusMask,
    ) -> Option<Self::SubscriberType> {
        // let subscriber_storage_weak = self
        //     .domain_participant_storage
        //     .lock()
        //     .create_subscriber(qos, a_listener, mask)?;
        // let subscriber = SubscriberProxy::new(self, subscriber_storage_weak);
        // Some(subscriber)
        todo!()
    }

    fn delete_subscriber_gat(&self, _a_subscriber: &Self::SubscriberType) -> DDSResult<()> {
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
        // let subscriber_storage_weak = self
        //     .domain_participant_storage
        //     .lock()
        //     .get_builtin_subscriber();
        // SubscriberProxy::new(self, subscriber_storage_weak)
        todo!()
    }
}

impl<'t, T: 'static> rust_dds_api::domain::domain_participant::TopicGAT<'t, T>
    for DomainParticipantImpl
{
    type TopicType = TopicProxy<'t, T>;

    fn create_topic_gat(
        &'t self,
        _topic_name: &str,
        _qos: Option<TopicQos>,
        _a_listener: Option<&'static dyn TopicListener<DataPIM = T>>,
        _mask: StatusMask,
    ) -> Option<Self::TopicType> {
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

    fn set_default_publisher_qos(&self, _qos: Option<PublisherQos>) -> DDSResult<()> {
        // self.domain_participant_storage
        //     .lock()
        //     .set_default_publisher_qos(qos)
        todo!()
    }

    fn get_default_publisher_qos(&self) -> PublisherQos {
        // self.domain_participant_storage
        //     .lock()
        //     .get_default_publisher_qos()
        //     .clone()
        todo!()
    }

    fn set_default_subscriber_qos(&self, _qos: Option<SubscriberQos>) -> DDSResult<()> {
        // self.domain_participant_storage
        //     .lock()
        //     .set_default_subscriber_qos(qos)
        todo!()
    }

    fn get_default_subscriber_qos(&self) -> SubscriberQos {
        // self.domain_participant_storage
        //     .lock()
        //     .get_default_subscriber_qos()
        //     .clone()
        todo!()
    }

    fn set_default_topic_qos(&self, _qos: Option<TopicQos>) -> DDSResult<()> {
        // self.domain_participant_storage
        // .lock()
        // .set_default_topic_qos(qos)
        todo!()
    }

    fn get_default_topic_qos(&self) -> TopicQos {
        todo!()
        // self.domain_participant_storage
        //     .lock()
        //     .get_default_topic_qos()
        //     .clone()
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

    fn set_qos(&self, _qos: Option<Self::Qos>) -> DDSResult<()> {
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
        // self.is_enabled.store(true, atomic::Ordering::Release);
        // let is_enabled = self.is_enabled.clone();
        // let domain_participant_storage = self.domain_participant_storage.clone();
        // std::thread::spawn(move || {
        //     while is_enabled.load(atomic::Ordering::Relaxed) {
        //         domain_participant_storage.lock().send_builtin_data();
        //         domain_participant_storage.lock().receive_builtin_data();
        //         domain_participant_storage.lock().send_user_defined_data();
        //         domain_participant_storage
        //             .lock()
        //             .receive_user_defined_data();
        //         std::thread::sleep(std::time::Duration::from_millis(100));
        //     }
        // });
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::{
//         rtps_impl::rtps_participant_impl::RtpsParticipantImpl, utils::transport::RtpsMessageRead,
//     };

//     use super::*;

//     use rust_dds_api::domain::domain_participant::DomainParticipant;
//     use rust_rtps_pim::structure::types::Locator;

//     struct MockDDSType;

//     struct MockTransport;

//     impl TransportRead for MockTransport {
//         fn read(&mut self) -> Option<(Locator, RtpsMessageRead)> {
//             todo!()
//         }
//     }

//     impl TransportWrite for MockTransport {
//         fn write(
//             &mut self,
//             _message: &crate::utils::transport::RtpsMessageWrite<'_>,
//             _destination_locator: &Locator,
//         ) {
//             todo!()
//         }
//     }

//     #[test]
//     fn set_default_publisher_qos_some_value() {
//         let rtps_participant = RtpsParticipantImpl::new([1; 12]);
//         let domain_participant_storage = DomainParticipantImpl::new(
//             DomainParticipantQos::default(),
//             rtps_participant,
//             vec![],
//             vec![],
//             Box::new(MockTransport),
//             Box::new(MockTransport),
//         );
//         let domain_participant_impl =
//             DomainParticipantProxy::new(RtpsShared::new(domain_participant_storage));
//         let mut qos = PublisherQos::default();
//         qos.group_data.value = &[1, 2, 3, 4];
//         domain_participant_impl
//             .set_default_publisher_qos(Some(qos.clone()))
//             .unwrap();
//         assert!(domain_participant_impl.get_default_publisher_qos() == qos);
//     }

//     #[test]
//     fn set_default_publisher_qos_none() {
//         let rtps_participant = RtpsParticipantImpl::new([1; 12]);
//         let domain_participant_storage = DomainParticipantImpl::new(
//             DomainParticipantQos::default(),
//             rtps_participant,
//             vec![],
//             vec![],
//             Box::new(MockTransport),
//             Box::new(MockTransport),
//         );
//         let domain_participant_impl =
//             DomainParticipantProxy::new(RtpsShared::new(domain_participant_storage));
//         let mut qos = PublisherQos::default();
//         qos.group_data.value = &[1, 2, 3, 4];
//         domain_participant_impl
//             .set_default_publisher_qos(Some(qos.clone()))
//             .unwrap();

//         domain_participant_impl
//             .set_default_publisher_qos(None)
//             .unwrap();
//         assert!(domain_participant_impl.get_default_publisher_qos() == PublisherQos::default());
//     }

//     #[test]
//     fn set_default_subscriber_qos_some_value() {
//         let rtps_participant = RtpsParticipantImpl::new([1; 12]);
//         let domain_participant_storage = DomainParticipantImpl::new(
//             DomainParticipantQos::default(),
//             rtps_participant,
//             vec![],
//             vec![],
//             Box::new(MockTransport),
//             Box::new(MockTransport),
//         );
//         let domain_participant_impl =
//             DomainParticipantProxy::new(RtpsShared::new(domain_participant_storage));
//         let mut qos = SubscriberQos::default();
//         qos.group_data.value = &[1, 2, 3, 4];
//         domain_participant_impl
//             .set_default_subscriber_qos(Some(qos.clone()))
//             .unwrap();
//         assert_eq!(domain_participant_impl.get_default_subscriber_qos(), qos);
//     }

//     #[test]
//     fn set_default_subscriber_qos_none() {
//         let rtps_participant = RtpsParticipantImpl::new([1; 12]);
//         let domain_participant_storage = DomainParticipantImpl::new(
//             DomainParticipantQos::default(),
//             rtps_participant,
//             vec![],
//             vec![],
//             Box::new(MockTransport),
//             Box::new(MockTransport),
//         );
//         let domain_participant_impl =
//             DomainParticipantProxy::new(RtpsShared::new(domain_participant_storage));
//         let mut qos = SubscriberQos::default();
//         qos.group_data.value = &[1, 2, 3, 4];
//         domain_participant_impl
//             .set_default_subscriber_qos(Some(qos.clone()))
//             .unwrap();

//         domain_participant_impl
//             .set_default_subscriber_qos(None)
//             .unwrap();
//         assert_eq!(
//             domain_participant_impl.get_default_subscriber_qos(),
//             SubscriberQos::default()
//         );
//     }

//     #[test]
//     fn set_default_topic_qos_some_value() {
//         let rtps_participant = RtpsParticipantImpl::new([1; 12]);
//         let domain_participant_storage = DomainParticipantImpl::new(
//             DomainParticipantQos::default(),
//             rtps_participant,
//             vec![],
//             vec![],
//             Box::new(MockTransport),
//             Box::new(MockTransport),
//         );
//         let domain_participant_impl =
//             DomainParticipantProxy::new(RtpsShared::new(domain_participant_storage));
//         let mut qos = TopicQos::default();
//         qos.topic_data.value = &[1, 2, 3, 4];
//         domain_participant_impl
//             .set_default_topic_qos(Some(qos.clone()))
//             .unwrap();
//         assert_eq!(domain_participant_impl.get_default_topic_qos(), qos);
//     }

//     #[test]
//     fn set_default_topic_qos_inconsistent() {
//         let rtps_participant = RtpsParticipantImpl::new([1; 12]);
//         let domain_participant_storage = DomainParticipantImpl::new(
//             DomainParticipantQos::default(),
//             rtps_participant,
//             vec![],
//             vec![],
//             Box::new(MockTransport),
//             Box::new(MockTransport),
//         );
//         let domain_participant_impl =
//             DomainParticipantProxy::new(RtpsShared::new(domain_participant_storage));
//         let mut qos = TopicQos::default();
//         qos.resource_limits.max_samples_per_instance = 2;
//         qos.resource_limits.max_samples = 1;
//         let set_default_topic_qos_result =
//             domain_participant_impl.set_default_topic_qos(Some(qos.clone()));
//         assert!(set_default_topic_qos_result == Err(DDSError::InconsistentPolicy));
//     }

//     #[test]
//     fn set_default_topic_qos_none() {
//         let rtps_participant = RtpsParticipantImpl::new([1; 12]);
//         let domain_participant_storage = DomainParticipantImpl::new(
//             DomainParticipantQos::default(),
//             rtps_participant,
//             vec![],
//             vec![],
//             Box::new(MockTransport),
//             Box::new(MockTransport),
//         );
//         let domain_participant_impl =
//             DomainParticipantProxy::new(RtpsShared::new(domain_participant_storage));
//         let mut qos = TopicQos::default();
//         qos.topic_data.value = &[1, 2, 3, 4];
//         domain_participant_impl
//             .set_default_topic_qos(Some(qos.clone()))
//             .unwrap();

//         domain_participant_impl.set_default_topic_qos(None).unwrap();
//         assert_eq!(
//             domain_participant_impl.get_default_topic_qos(),
//             TopicQos::default()
//         );
//     }

//     #[test]
//     fn create_publisher() {
//         let rtps_participant = RtpsParticipantImpl::new([1; 12]);
//         let domain_participant_storage = DomainParticipantImpl::new(
//             DomainParticipantQos::default(),
//             rtps_participant,
//             vec![],
//             vec![],
//             Box::new(MockTransport),
//             Box::new(MockTransport),
//         );
//         let domain_participant_impl =
//             DomainParticipantProxy::new(RtpsShared::new(domain_participant_storage));
//         let publisher = domain_participant_impl.create_publisher(None, None, 0);

//         assert!(publisher.is_some())
//     }

//     #[test]
//     fn create_subscriber() {
//         let rtps_participant = RtpsParticipantImpl::new([1; 12]);
//         let domain_participant_storage = DomainParticipantImpl::new(
//             DomainParticipantQos::default(),
//             rtps_participant,
//             vec![],
//             vec![],
//             Box::new(MockTransport),
//             Box::new(MockTransport),
//         );
//         let domain_participant_impl =
//             DomainParticipantProxy::new(RtpsShared::new(domain_participant_storage));
//         let subscriber = domain_participant_impl.create_subscriber(None, None, 0);

//         assert!(subscriber.is_some())
//     }

//     #[test]
//     fn create_topic() {
//         let rtps_participant = RtpsParticipantImpl::new([1; 12]);
//         let domain_participant_storage = DomainParticipantImpl::new(
//             DomainParticipantQos::default(),
//             rtps_participant,
//             vec![],
//             vec![],
//             Box::new(MockTransport),
//             Box::new(MockTransport),
//         );
//         let domain_participant_impl =
//             DomainParticipantProxy::new(RtpsShared::new(domain_participant_storage));
//         let topic =
//             domain_participant_impl.create_topic::<MockDDSType>("topic_name", None, None, 0);
//         assert!(topic.is_some());
//     }
// }
