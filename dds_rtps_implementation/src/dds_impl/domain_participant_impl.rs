use rust_dds_api::infrastructure::qos::{
    DomainParticipantQos, PublisherQos, SubscriberQos, TopicQos,
};
use rust_rtps_pim::structure::RtpsEntity;

use crate::{
    rtps_impl::rtps_participant_impl::RtpsParticipantImpl, utils::shared_object::RtpsShared,
};

use super::{
    domain_participant_proxy::Transport, publisher_proxy::PublisherImpl,
    subscriber_proxy::SubscriberImpl, topic_proxy::TopicImpl,
};

pub struct DomainParticipantImpl {
    pub rtps_participant: RtpsParticipantImpl,
    pub domain_participant_qos: DomainParticipantQos,
    pub builtin_subscriber_storage: Vec<RtpsShared<SubscriberImpl>>,
    pub builtin_publisher_storage: Vec<RtpsShared<PublisherImpl>>,
    pub user_defined_subscriber_storage: Vec<RtpsShared<SubscriberImpl>>,
    pub user_defined_subscriber_counter: u8,
    pub default_subscriber_qos: SubscriberQos,
    pub user_defined_publisher_storage: Vec<RtpsShared<PublisherImpl>>,
    pub user_defined_publisher_counter: u8,
    pub default_publisher_qos: PublisherQos,
    pub topic_storage: Vec<RtpsShared<TopicImpl>>,
    pub default_topic_qos: TopicQos,
    pub metatraffic_transport: Box<dyn Transport>,
    pub default_transport: Box<dyn Transport>,
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
