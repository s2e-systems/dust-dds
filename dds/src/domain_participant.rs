use rust_dds_rtps_implementation::dds_impl::{
    domain_participant_impl::DomainParticipantImpl, publisher_impl::PublisherImpl,
    subscriber_impl::SubscriberImpl, topic_impl::TopicImpl,
};

pub enum DomainParticipant {
    Rtps(DomainParticipantImpl),
}

impl<'p> rust_dds_api::domain::domain_participant::PublisherFactory<'p> for DomainParticipant {
    type PublisherType = PublisherImpl<'p>;

    fn create_publisher(
        &'p self,
        qos: Option<rust_dds_api::infrastructure::qos::PublisherQos>,
        a_listener: Option<
            &'static dyn rust_dds_api::publication::publisher_listener::PublisherListener,
        >,
        mask: rust_dds_api::dcps_psm::StatusMask,
    ) -> Option<Self::PublisherType> {
        match self {
            DomainParticipant::Rtps(dp) => dp.create_publisher(qos, a_listener, mask),
        }
    }

    fn delete_publisher(
        &self,
        a_publisher: &Self::PublisherType,
    ) -> rust_dds_api::return_type::DDSResult<()> {
        match self {
            DomainParticipant::Rtps(dp) => dp.delete_publisher(a_publisher),
        }
    }
}

impl<'s> rust_dds_api::domain::domain_participant::SubscriberFactory<'s> for DomainParticipant {
    type SubscriberType = SubscriberImpl<'s>;

    fn create_subscriber(
        &'s self,
        qos: Option<rust_dds_api::infrastructure::qos::SubscriberQos>,
        a_listener: Option<
            &'static dyn rust_dds_api::subscription::subscriber_listener::SubscriberListener,
        >,
        mask: rust_dds_api::dcps_psm::StatusMask,
    ) -> Option<Self::SubscriberType> {
        todo!()
    }

    fn delete_subscriber(
        &self,
        a_subscriber: &Self::SubscriberType,
    ) -> rust_dds_api::return_type::DDSResult<()> {
        todo!()
    }

    fn get_builtin_subscriber(&'s self) -> Self::SubscriberType {
        todo!()
    }
}

impl<'t, T: 'static> rust_dds_api::domain::domain_participant::TopicFactory<'t, T>
    for DomainParticipant
{
    type TopicType = TopicImpl<'t, T>;

    fn create_topic(
        &'t self,
        topic_name: &str,
        qos: Option<rust_dds_api::infrastructure::qos::TopicQos>,
        a_listener: Option<
            &'static dyn rust_dds_api::topic::topic_listener::TopicListener<DataPIM = T>,
        >,
        mask: rust_dds_api::dcps_psm::StatusMask,
    ) -> Option<Self::TopicType> {
        todo!()
    }

    fn delete_topic(&self, a_topic: &Self::TopicType) -> rust_dds_api::return_type::DDSResult<()> {
        todo!()
    }

    fn find_topic(
        &'t self,
        topic_name: &'t str,
        timeout: rust_dds_api::dcps_psm::Duration,
    ) -> Option<Self::TopicType> {
        todo!()
    }
}

impl rust_dds_api::domain::domain_participant::DomainParticipant for DomainParticipant {
    fn lookup_topicdescription<'t, T>(
        &'t self,
        _name: &'t str,
    ) -> Option<&'t dyn rust_dds_api::topic::topic_description::TopicDescription<T>>
    where
        Self: Sized,
    {
        todo!()
    }

    fn ignore_participant(
        &self,
        handle: rust_dds_api::dcps_psm::InstanceHandle,
    ) -> rust_dds_api::return_type::DDSResult<()> {
        todo!()
    }

    fn ignore_topic(
        &self,
        handle: rust_dds_api::dcps_psm::InstanceHandle,
    ) -> rust_dds_api::return_type::DDSResult<()> {
        todo!()
    }

    fn ignore_publication(
        &self,
        handle: rust_dds_api::dcps_psm::InstanceHandle,
    ) -> rust_dds_api::return_type::DDSResult<()> {
        todo!()
    }

    fn ignore_subscription(
        &self,
        handle: rust_dds_api::dcps_psm::InstanceHandle,
    ) -> rust_dds_api::return_type::DDSResult<()> {
        todo!()
    }

    fn get_domain_id(&self) -> rust_dds_api::dcps_psm::DomainId {
        todo!()
    }

    fn delete_contained_entities(&self) -> rust_dds_api::return_type::DDSResult<()> {
        todo!()
    }

    fn assert_liveliness(&self) -> rust_dds_api::return_type::DDSResult<()> {
        todo!()
    }

    fn set_default_publisher_qos(
        &self,
        qos: Option<rust_dds_api::infrastructure::qos::PublisherQos>,
    ) -> rust_dds_api::return_type::DDSResult<()> {
        todo!()
    }

    fn get_default_publisher_qos(&self) -> rust_dds_api::infrastructure::qos::PublisherQos {
        todo!()
    }

    fn set_default_subscriber_qos(
        &self,
        qos: Option<rust_dds_api::infrastructure::qos::SubscriberQos>,
    ) -> rust_dds_api::return_type::DDSResult<()> {
        todo!()
    }

    fn get_default_subscriber_qos(&self) -> rust_dds_api::infrastructure::qos::SubscriberQos {
        todo!()
    }

    fn set_default_topic_qos(
        &self,
        qos: Option<rust_dds_api::infrastructure::qos::TopicQos>,
    ) -> rust_dds_api::return_type::DDSResult<()> {
        todo!()
    }

    fn get_default_topic_qos(&self) -> rust_dds_api::infrastructure::qos::TopicQos {
        todo!()
    }

    fn get_discovered_participants(
        &self,
        participant_handles: &mut [rust_dds_api::dcps_psm::InstanceHandle],
    ) -> rust_dds_api::return_type::DDSResult<()> {
        todo!()
    }

    fn get_discovered_participant_data(
        &self,
        participant_data: rust_dds_api::builtin_topics::ParticipantBuiltinTopicData,
        participant_handle: rust_dds_api::dcps_psm::InstanceHandle,
    ) -> rust_dds_api::return_type::DDSResult<()> {
        todo!()
    }

    fn get_discovered_topics(
        &self,
        topic_handles: &mut [rust_dds_api::dcps_psm::InstanceHandle],
    ) -> rust_dds_api::return_type::DDSResult<()> {
        todo!()
    }

    fn get_discovered_topic_data(
        &self,
        topic_data: rust_dds_api::builtin_topics::TopicBuiltinTopicData,
        topic_handle: rust_dds_api::dcps_psm::InstanceHandle,
    ) -> rust_dds_api::return_type::DDSResult<()> {
        todo!()
    }

    fn contains_entity(&self, a_handle: rust_dds_api::dcps_psm::InstanceHandle) -> bool {
        todo!()
    }

    fn get_current_time(
        &self,
    ) -> rust_dds_api::return_type::DDSResult<rust_dds_api::dcps_psm::Time> {
        todo!()
    }
}

impl rust_dds_api::infrastructure::entity::Entity for DomainParticipant {
    type Qos = rust_dds_api::infrastructure::qos::DomainParticipantQos;
    type Listener =
        &'static dyn rust_dds_api::domain::domain_participant_listener::DomainParticipantListener;

    fn set_qos(&self, qos: Option<Self::Qos>) -> rust_dds_api::return_type::DDSResult<()> {
        todo!()
    }

    fn get_qos(&self) -> rust_dds_api::return_type::DDSResult<Self::Qos> {
        todo!()
    }

    fn set_listener(
        &self,
        a_listener: Option<Self::Listener>,
        mask: rust_dds_api::dcps_psm::StatusMask,
    ) -> rust_dds_api::return_type::DDSResult<()> {
        todo!()
    }

    fn get_listener(&self) -> rust_dds_api::return_type::DDSResult<Option<Self::Listener>> {
        todo!()
    }

    fn get_statuscondition(&self) -> rust_dds_api::infrastructure::entity::StatusCondition {
        todo!()
    }

    fn get_status_changes(&self) -> rust_dds_api::dcps_psm::StatusMask {
        todo!()
    }

    fn enable(&self) -> rust_dds_api::return_type::DDSResult<()> {
        match self {
            DomainParticipant::Rtps(dp) => dp.enable(),
        }
    }

    fn get_instance_handle(
        &self,
    ) -> rust_dds_api::return_type::DDSResult<rust_dds_api::dcps_psm::InstanceHandle> {
        todo!()
    }
}
