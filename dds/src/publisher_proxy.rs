use dds_api::{
    dcps_psm::{Duration, InstanceHandle, StatusMask},
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::{DataWriterQos, PublisherQos, TopicQos},
    },
    publication::{
        publisher::{Publisher, PublisherDataWriterFactory, PublisherGetParticipant},
        publisher_listener::PublisherListener,
    },
    return_type::DdsResult,
};
use dds_implementation::{
    dds_impl::{
        data_writer_impl::{AnyDataWriterListener, DataWriterImpl},
        publisher_impl::PublisherImpl,
    },
    dds_type::{DdsSerialize, DdsType},
    utils::shared_object::{DdsShared, DdsWeak},
};

use crate::{
    data_writer_proxy::DataWriterProxy, domain_participant_proxy::DomainParticipantProxy,
    topic_proxy::TopicProxy,
};

#[derive(Clone)]
pub struct PublisherProxy {
    publisher_attributes: DdsWeak<PublisherImpl>,
}

impl PublisherProxy {
    pub fn new(publisher_attributes: DdsWeak<PublisherImpl>) -> Self {
        Self {
            publisher_attributes,
        }
    }
}

impl AsRef<DdsWeak<PublisherImpl>> for PublisherProxy {
    fn as_ref(&self) -> &DdsWeak<PublisherImpl> {
        &self.publisher_attributes
    }
}

impl<Foo> PublisherDataWriterFactory<Foo> for PublisherProxy
where
    Foo: DdsType + DdsSerialize + 'static,
{
    type TopicType = TopicProxy<Foo>;
    type DataWriterType = DataWriterProxy<Foo>;

    fn datawriter_factory_create_datawriter(
        &self,
        a_topic: &Self::TopicType,
        qos: Option<DataWriterQos>,
        a_listener: Option<<Self::DataWriterType as Entity>::Listener>,
        mask: StatusMask,
    ) -> DdsResult<Self::DataWriterType> {
        #[allow(clippy::redundant_closure)]
        PublisherDataWriterFactory::<Foo>::datawriter_factory_create_datawriter(
            &self.publisher_attributes.upgrade()?,
            &a_topic.as_ref().upgrade()?,
            qos,
            a_listener
                .map::<Box<dyn AnyDataWriterListener<DdsShared<DataWriterImpl>> + Send + Sync>, _>(
                    |x| Box::new(x),
                ),
            mask,
        )
        .map(|x| DataWriterProxy::new(x.downgrade()))
    }

    fn datawriter_factory_delete_datawriter(
        &self,
        a_datawriter: &Self::DataWriterType,
    ) -> DdsResult<()> {
        PublisherDataWriterFactory::<Foo>::datawriter_factory_delete_datawriter(
            &self.publisher_attributes.upgrade()?,
            &a_datawriter.as_ref().upgrade()?,
        )
    }

    fn datawriter_factory_lookup_datawriter(
        &self,
        topic: &Self::TopicType,
    ) -> DdsResult<Self::DataWriterType> {
        PublisherDataWriterFactory::<Foo>::datawriter_factory_lookup_datawriter(
            &self.publisher_attributes.upgrade()?,
            &topic.as_ref().upgrade()?,
        )
        .map(|x| DataWriterProxy::new(x.downgrade()))
    }
}

impl Publisher for PublisherProxy {
    fn suspend_publications(&self) -> DdsResult<()> {
        self.publisher_attributes.upgrade()?.suspend_publications()
    }

    fn resume_publications(&self) -> DdsResult<()> {
        self.publisher_attributes.upgrade()?.resume_publications()
    }

    fn begin_coherent_changes(&self) -> DdsResult<()> {
        self.publisher_attributes
            .upgrade()?
            .begin_coherent_changes()
    }

    fn end_coherent_changes(&self) -> DdsResult<()> {
        self.publisher_attributes.upgrade()?.end_coherent_changes()
    }

    fn wait_for_acknowledgments(&self, max_wait: Duration) -> DdsResult<()> {
        self.publisher_attributes
            .upgrade()?
            .wait_for_acknowledgments(max_wait)
    }

    fn delete_contained_entities(&self) -> DdsResult<()> {
        self.publisher_attributes
            .upgrade()?
            .delete_contained_entities()
    }

    fn set_default_datawriter_qos(&self, qos: Option<DataWriterQos>) -> DdsResult<()> {
        self.publisher_attributes
            .upgrade()?
            .set_default_datawriter_qos(qos)
    }

    fn get_default_datawriter_qos(&self) -> DdsResult<DataWriterQos> {
        self.publisher_attributes
            .upgrade()?
            .get_default_datawriter_qos()
    }

    fn copy_from_topic_qos(
        &self,
        a_datawriter_qos: &mut DataWriterQos,
        a_topic_qos: &TopicQos,
    ) -> DdsResult<()> {
        self.publisher_attributes
            .upgrade()?
            .copy_from_topic_qos(a_datawriter_qos, a_topic_qos)
    }
}

impl PublisherGetParticipant for PublisherProxy {
    type DomainParticipant = DomainParticipantProxy;

    fn publisher_get_participant(&self) -> DdsResult<Self::DomainParticipant> {
        self.publisher_attributes
            .upgrade()?
            .get_participant()
            .map(|x| DomainParticipantProxy::new(x.downgrade()))
    }
}

impl Entity for PublisherProxy {
    type Qos = PublisherQos;
    type Listener = Box<dyn PublisherListener>;

    fn set_qos(&self, qos: Option<Self::Qos>) -> DdsResult<()> {
        self.publisher_attributes.upgrade()?.set_qos(qos)
    }

    fn get_qos(&self) -> DdsResult<Self::Qos> {
        self.publisher_attributes.upgrade()?.get_qos()
    }

    fn set_listener(&self, a_listener: Option<Self::Listener>, mask: StatusMask) -> DdsResult<()> {
        self.publisher_attributes
            .upgrade()?
            .set_listener(a_listener, mask)
    }

    fn get_listener(&self) -> DdsResult<Option<Self::Listener>> {
        self.publisher_attributes.upgrade()?.get_listener()
    }

    fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        self.publisher_attributes.upgrade()?.get_statuscondition()
    }

    fn get_status_changes(&self) -> DdsResult<StatusMask> {
        self.publisher_attributes.upgrade()?.get_status_changes()
    }

    fn enable(&self) -> DdsResult<()> {
        self.publisher_attributes.upgrade()?.enable()
    }

    fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        self.publisher_attributes.upgrade()?.get_instance_handle()
    }
}
