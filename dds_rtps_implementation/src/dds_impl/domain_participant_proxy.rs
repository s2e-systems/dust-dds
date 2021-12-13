use rust_dds_api::{
    builtin_topics::{ParticipantBuiltinTopicData, TopicBuiltinTopicData},
    dcps_psm::{InstanceHandle, StatusMask, Time},
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
    subscription::subscriber_listener::SubscriberListener,
    topic::topic_listener::TopicListener,
};

use crate::utils::shared_object::{rtps_shared_downgrade, rtps_weak_upgrade};

use super::{
    domain_participant_impl::DomainParticipantImpl, publisher_impl::PublisherImpl,
    publisher_proxy::PublisherProxy, subscriber_impl::SubscriberImpl,
    subscriber_proxy::SubscriberProxy, topic_impl::TopicImpl, topic_proxy::TopicProxy,
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
    type PublisherType = PublisherProxy<'p, PublisherImpl>;

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
    type SubscriberType = SubscriberProxy<'s, SubscriberImpl>;

    fn subscriber_factory_create_subscriber(
        &'s self,
        qos: Option<SubscriberQos>,
        a_listener: Option<&'static dyn SubscriberListener>,
        mask: StatusMask,
    ) -> Option<Self::SubscriberType> {
        todo!()
    }

    fn subscriber_factory_delete_subscriber(
        &self,
        a_subscriber: &Self::SubscriberType,
    ) -> DDSResult<()> {
        todo!()
    }

    fn subscriber_factory_get_builtin_subscriber(&'s self) -> Self::SubscriberType {
        todo!()
    }
}

impl<'t, Foo> DomainParticipantTopicFactory<'t, Foo> for DomainParticipantProxy
where
    Foo: 't,
{
    type TopicType = TopicProxy<'t, Foo, TopicImpl>;

    fn topic_factory_create_topic(
        &'t self,
        topic_name: &str,
        qos: Option<TopicQos>,
        a_listener: Option<Box<dyn TopicListener<DataType = Foo>>>,
        mask: StatusMask,
    ) -> Option<Self::TopicType> {
        todo!()
    }

    fn topic_factory_delete_topic(&self, a_topic: &Self::TopicType) -> DDSResult<()> {
        todo!()
    }

    fn topic_factory_find_topic(
        &'t self,
        topic_name: &'t str,
        timeout: rust_dds_api::dcps_psm::Duration,
    ) -> Option<Self::TopicType> {
        todo!()
    }
}

impl DomainParticipant for DomainParticipantProxy {
    fn lookup_topicdescription<'t, T>(
        &'t self,
        _name: &'t str,
    ) -> Option<&'t dyn rust_dds_api::topic::topic_description::TopicDescription<T>>
    where
        Self: Sized,
    {
        todo!()
    }

    fn ignore_participant(&self, handle: InstanceHandle) -> DDSResult<()> {
        todo!()
    }

    fn ignore_topic(&self, handle: InstanceHandle) -> DDSResult<()> {
        todo!()
    }

    fn ignore_publication(&self, handle: InstanceHandle) -> DDSResult<()> {
        todo!()
    }

    fn ignore_subscription(&self, handle: InstanceHandle) -> DDSResult<()> {
        todo!()
    }

    fn get_domain_id(&self) -> rust_dds_api::dcps_psm::DomainId {
        todo!()
    }

    fn delete_contained_entities(&self) -> DDSResult<()> {
        todo!()
    }

    fn assert_liveliness(&self) -> DDSResult<()> {
        todo!()
    }

    fn set_default_publisher_qos(&mut self, qos: Option<PublisherQos>) -> DDSResult<()> {
        todo!()
    }

    fn get_default_publisher_qos(&self) -> PublisherQos {
        todo!()
    }

    fn set_default_subscriber_qos(&mut self, qos: Option<SubscriberQos>) -> DDSResult<()> {
        todo!()
    }

    fn get_default_subscriber_qos(&self) -> SubscriberQos {
        todo!()
    }

    fn set_default_topic_qos(&mut self, qos: Option<TopicQos>) -> DDSResult<()> {
        todo!()
    }

    fn get_default_topic_qos(&self) -> TopicQos {
        todo!()
    }

    fn get_discovered_participants(
        &self,
        participant_handles: &mut [InstanceHandle],
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_discovered_participant_data(
        &self,
        participant_data: ParticipantBuiltinTopicData,
        participant_handle: InstanceHandle,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_discovered_topics(&self, topic_handles: &mut [InstanceHandle]) -> DDSResult<()> {
        todo!()
    }

    fn get_discovered_topic_data(
        &self,
        topic_data: TopicBuiltinTopicData,
        topic_handle: InstanceHandle,
    ) -> DDSResult<()> {
        todo!()
    }

    fn contains_entity(&self, a_handle: InstanceHandle) -> bool {
        todo!()
    }

    fn get_current_time(&self) -> DDSResult<Time> {
        todo!()
    }
}

impl Entity for DomainParticipantProxy {
    type Qos = DomainParticipantQos;

    type Listener = Box<dyn DomainParticipantListener>;

    fn set_qos(&mut self, qos: Option<Self::Qos>) -> DDSResult<()> {
        todo!()
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        todo!()
    }

    fn set_listener(&self, a_listener: Option<Self::Listener>, mask: StatusMask) -> DDSResult<()> {
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

    fn enable(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        todo!()
    }
}
