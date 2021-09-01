use rust_dds_api::{
    dcps_psm::StatusMask,
    infrastructure::qos::{DomainParticipantQos, PublisherQos, SubscriberQos, TopicQos},
    publication::publisher_listener::PublisherListener,
    return_type::DDSResult,
    subscription::subscriber_listener::SubscriberListener,
    topic::topic_listener::TopicListener,
};
use rust_rtps_pim::structure::{
    types::{EntityId, EntityKind, Guid},
    RtpsEntity,
};

use crate::{
    rtps_impl::{rtps_group_impl::RtpsGroupImpl, rtps_participant_impl::RtpsParticipantImpl},
    utils::shared_object::{RtpsShared, RtpsWeak},
};

use super::{
    domain_participant_proxy::Transport, publisher_impl::PublisherImpl,
    subscriber_impl::SubscriberImpl, topic_impl::TopicImpl,
};

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
        for publisher in &self.builtin_publisher_storage {
            let publisher_lock = publisher.lock();
            for data_writer in publisher_lock.data_writer_storage_list() {
                let mut data_writer_lock = data_writer.lock();
                crate::utils::message_sender::send_data(
                    &self.rtps_participant,
                    data_writer_lock.rtps_data_writer_mut(),
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
                    data_writer_lock.rtps_data_writer_mut(),
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
