use std::marker::PhantomData;

use rust_dds_api::{
    dcps_psm::{InconsistentTopicStatus, StatusMask},
    infrastructure::{entity::Entity, qos::TopicQos},
    return_type::DDSResult,
    topic::{topic::Topic, topic_description::TopicDescription, topic_listener::TopicListener},
};

pub struct TopicImpl<Foo> {
    _qos: TopicQos,
    type_name: &'static str,
    topic_name: String,
    phantom: PhantomData<Foo>,
}

impl<Foo> TopicImpl<Foo> {
    pub fn new(qos: TopicQos, type_name: &'static str, topic_name: &str) -> Self {
        Self {
            _qos: qos,
            type_name,
            topic_name: topic_name.to_string(),
            phantom: PhantomData,
        }
    }

    // pub fn set_qos(&mut self, qos: Option<TopicQos>) -> DDSResult<()> {
    //     let qos = qos.unwrap_or_default();
    //     qos.is_consistent()?;
    //     self.qos = qos;
    //     Ok(())
    // }

    // pub fn get_qos(&self) -> &TopicQos {
    //     &self.qos
    // }
}

impl<Foo> Topic for TopicImpl<Foo> {
    fn get_inconsistent_topic_status(&self) -> DDSResult<InconsistentTopicStatus> {
        todo!()
    }
}

impl<Foo> TopicDescription for TopicImpl<Foo> {
    type DomainParticipant = ();

    fn get_type_name(&self) -> DDSResult<&'static str> {
        Ok(&self.type_name)
    }

    fn get_name(&self) -> DDSResult<String> {
        Ok(self.topic_name.clone())
    }

    fn get_participant(&self) -> Self::DomainParticipant {
        todo!()
    }
}

impl<Foo> Entity for TopicImpl<Foo> {
    type Qos = TopicQos;
    type Listener = Box<dyn TopicListener>;

    fn set_qos(&mut self, _qos: Option<Self::Qos>) -> DDSResult<()> {
        todo!()
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        todo!()
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

    fn get_statuscondition(
        &self,
    ) -> DDSResult<rust_dds_api::infrastructure::entity::StatusCondition> {
        todo!()
    }

    fn get_status_changes(&self) -> DDSResult<StatusMask> {
        todo!()
    }

    fn enable(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> DDSResult<rust_dds_api::dcps_psm::InstanceHandle> {
        todo!()
    }
}
