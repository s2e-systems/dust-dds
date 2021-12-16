use rust_dds_api::{
    builtin_topics::{ParticipantBuiltinTopicData, TopicBuiltinTopicData},
    dcps_psm::{DomainId, Duration, InstanceHandle, StatusMask, Time},
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
    subscription::{subscriber::Subscriber, subscriber_listener::SubscriberListener},
    topic::{topic_description::TopicDescription, topic_listener::TopicListener},
};

use crate::{
    dds_type::DdsType,
    utils::shared_object::{rtps_shared_downgrade, rtps_weak_upgrade},
};

use super::{
    domain_participant_impl::DomainParticipantImpl, publisher_impl::PublisherImpl,
    publisher_proxy::PublisherProxy, subscriber_impl::SubscriberImpl,
    subscriber_proxy::SubscriberProxy, topic_proxy::TopicProxy,
};

pub struct DomainParticipantProxy {
    domain_participant: DomainParticipantImpl<SubscriberImpl, PublisherImpl>,
}

impl DomainParticipantProxy {
    pub fn new(domain_participant: DomainParticipantImpl<SubscriberImpl, PublisherImpl>) -> Self {
        Self { domain_participant }
    }
}

impl<'p> DomainParticipantPublisherFactory<'p> for DomainParticipantProxy {
    type PublisherType = PublisherProxy<'p>;

    fn publisher_factory_create_publisher(
        &'p self,
        qos: Option<PublisherQos>,
        a_listener: Option<&'static dyn PublisherListener>,
        mask: StatusMask,
    ) -> Option<Self::PublisherType> {
        let publisher_shared = self
            .domain_participant
            .publisher_factory_create_publisher(qos, a_listener, mask)?;
        let publisher_weak = rtps_shared_downgrade(&publisher_shared);

        Some(PublisherProxy::new(self, publisher_weak))
    }

    fn publisher_factory_delete_publisher(
        &self,
        a_publisher: &Self::PublisherType,
    ) -> DDSResult<()> {
        let publisher_shared = rtps_weak_upgrade(a_publisher.as_ref())?;
        if std::ptr::eq(a_publisher.get_participant(), self) {
            self.domain_participant
                .publisher_factory_delete_publisher(&publisher_shared)
        } else {
            Err(DDSError::PreconditionNotMet(
                "Publisher can only be deleted from its parent participant".to_string(),
            ))
        }
    }
}

impl<'s> DomainParticipantSubscriberFactory<'s> for DomainParticipantProxy {
    type SubscriberType = SubscriberProxy<'s>;

    fn subscriber_factory_create_subscriber(
        &'s self,
        qos: Option<SubscriberQos>,
        a_listener: Option<&'static dyn SubscriberListener>,
        mask: StatusMask,
    ) -> Option<Self::SubscriberType> {
        let subscriber_shared = self
            .domain_participant
            .subscriber_factory_create_subscriber(qos, a_listener, mask)?;
        let subscriber_weak = rtps_shared_downgrade(&subscriber_shared);
        Some(SubscriberProxy::new(self, subscriber_weak))
    }

    fn subscriber_factory_delete_subscriber(
        &self,
        a_subscriber: &Self::SubscriberType,
    ) -> DDSResult<()> {
        let subscriber_shared = rtps_weak_upgrade(a_subscriber.as_ref())?;
        if std::ptr::eq(a_subscriber.get_participant(), self) {
            self.domain_participant
                .subscriber_factory_delete_subscriber(&subscriber_shared)
        } else {
            Err(DDSError::PreconditionNotMet(
                "Subscriber can only be deleted from its parent participant".to_string(),
            ))
        }
    }

    fn subscriber_factory_get_builtin_subscriber(&'s self) -> Self::SubscriberType {
        let subscriber_shared = self
            .domain_participant
            .subscriber_factory_get_builtin_subscriber();
        let subscriber_weak = rtps_shared_downgrade(&subscriber_shared);
        SubscriberProxy::new(self, subscriber_weak)
    }
}

impl<'t, Foo> DomainParticipantTopicFactory<'t, Foo> for DomainParticipantProxy
where
    Foo: DdsType + 'static,
{
    type TopicType = TopicProxy<'t, Foo>;

    fn topic_factory_create_topic(
        &'t self,
        topic_name: &str,
        qos: Option<TopicQos>,
        a_listener: Option<Box<dyn TopicListener<DataType = Foo>>>,
        mask: StatusMask,
    ) -> Option<Self::TopicType> {
        let topic_shared = self
            .domain_participant
            .topic_factory_create_topic(topic_name, qos, a_listener, mask)?;
        let topic_weak = rtps_shared_downgrade(&topic_shared);
        Some(TopicProxy::new(self, topic_weak))
    }

    fn topic_factory_delete_topic(&self, a_topic: &Self::TopicType) -> DDSResult<()> {
        let topic_shared = rtps_weak_upgrade(a_topic.as_ref())?;
        if std::ptr::eq(a_topic.get_participant(), self) {
            // Explicit call with the complete function path otherwise the generic type can't be infered.
            // This happens because TopicImpl has no generic type information.
            DomainParticipantTopicFactory::<'t, Foo>::topic_factory_delete_topic(
                &self.domain_participant,
                &topic_shared,
            )
        } else {
            Err(DDSError::PreconditionNotMet(
                "Subscriber can only be deleted from its parent participant".to_string(),
            ))
        }
    }

    fn topic_factory_find_topic(
        &'t self,
        topic_name: &'t str,
        timeout: Duration,
    ) -> Option<Self::TopicType> {
        // Explicit call with the complete function path otherwise the generic type can't be infered.
        // This happens because TopicImpl has no generic type information.
        let topic_shared = DomainParticipantTopicFactory::<'t, Foo>::topic_factory_find_topic(
            &self.domain_participant,
            topic_name,
            timeout,
        )?;
        let topic_weak = rtps_shared_downgrade(&topic_shared);
        Some(TopicProxy::new(self, topic_weak))
    }
}

impl DomainParticipant for DomainParticipantProxy {
    fn lookup_topicdescription<'t, T>(
        &'t self,
        name: &'t str,
    ) -> Option<&'t dyn TopicDescription<T>>
    where
        Self: Sized,
    {
        self.domain_participant.lookup_topicdescription(name)
    }

    fn ignore_participant(&self, handle: InstanceHandle) -> DDSResult<()> {
        self.domain_participant.ignore_participant(handle)
    }

    fn ignore_topic(&self, handle: InstanceHandle) -> DDSResult<()> {
        self.domain_participant.ignore_topic(handle)
    }

    fn ignore_publication(&self, handle: InstanceHandle) -> DDSResult<()> {
        self.domain_participant.ignore_publication(handle)
    }

    fn ignore_subscription(&self, handle: InstanceHandle) -> DDSResult<()> {
        self.domain_participant.ignore_subscription(handle)
    }

    fn get_domain_id(&self) -> DomainId {
        self.domain_participant.get_domain_id()
    }

    fn delete_contained_entities(&self) -> DDSResult<()> {
        self.domain_participant.delete_contained_entities()
    }

    fn assert_liveliness(&self) -> DDSResult<()> {
        self.domain_participant.assert_liveliness()
    }

    fn set_default_publisher_qos(&mut self, qos: Option<PublisherQos>) -> DDSResult<()> {
        self.domain_participant.set_default_publisher_qos(qos)
    }

    fn get_default_publisher_qos(&self) -> PublisherQos {
        self.domain_participant.get_default_publisher_qos()
    }

    fn set_default_subscriber_qos(&mut self, qos: Option<SubscriberQos>) -> DDSResult<()> {
        self.domain_participant.set_default_subscriber_qos(qos)
    }

    fn get_default_subscriber_qos(&self) -> SubscriberQos {
        self.domain_participant.get_default_subscriber_qos()
    }

    fn set_default_topic_qos(&mut self, qos: Option<TopicQos>) -> DDSResult<()> {
        self.domain_participant.set_default_topic_qos(qos)
    }

    fn get_default_topic_qos(&self) -> TopicQos {
        self.domain_participant.get_default_topic_qos()
    }

    fn get_discovered_participants(
        &self,
        participant_handles: &mut [InstanceHandle],
    ) -> DDSResult<()> {
        self.domain_participant
            .get_discovered_participants(participant_handles)
    }

    fn get_discovered_participant_data(
        &self,
        participant_data: ParticipantBuiltinTopicData,
        participant_handle: InstanceHandle,
    ) -> DDSResult<()> {
        self.domain_participant
            .get_discovered_participant_data(participant_data, participant_handle)
    }

    fn get_discovered_topics(&self, topic_handles: &mut [InstanceHandle]) -> DDSResult<()> {
        self.domain_participant.get_discovered_topics(topic_handles)
    }

    fn get_discovered_topic_data(
        &self,
        topic_data: TopicBuiltinTopicData,
        topic_handle: InstanceHandle,
    ) -> DDSResult<()> {
        self.domain_participant
            .get_discovered_topic_data(topic_data, topic_handle)
    }

    fn contains_entity(&self, a_handle: InstanceHandle) -> bool {
        self.domain_participant.contains_entity(a_handle)
    }

    fn get_current_time(&self) -> DDSResult<Time> {
        self.domain_participant.get_current_time()
    }
}

impl Entity for DomainParticipantProxy {
    type Qos = DomainParticipantQos;
    type Listener = &'static dyn DomainParticipantListener;

    fn set_qos(&mut self, qos: Option<Self::Qos>) -> DDSResult<()> {
        self.domain_participant.set_qos(qos)
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        self.domain_participant.get_qos()
    }

    fn set_listener(&self, a_listener: Option<Self::Listener>, mask: StatusMask) -> DDSResult<()> {
        self.domain_participant.set_listener(a_listener, mask)
    }

    fn get_listener(&self) -> DDSResult<Option<Self::Listener>> {
        self.domain_participant.get_listener()
    }

    fn get_statuscondition(&self) -> DDSResult<StatusCondition> {
        self.domain_participant.get_statuscondition()
    }

    fn get_status_changes(&self) -> DDSResult<StatusMask> {
        self.domain_participant.get_status_changes()
    }

    fn enable(&self) -> DDSResult<()> {
        self.domain_participant.enable()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        self.domain_participant.get_instance_handle()
    }
}
