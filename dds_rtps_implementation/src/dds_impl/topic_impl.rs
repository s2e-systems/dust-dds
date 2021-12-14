use rust_dds_api::{
    dcps_psm::InconsistentTopicStatus,
    infrastructure::{entity::Entity, qos::TopicQos},
    return_type::DDSResult,
    topic::{topic::Topic, topic_description::TopicDescription, topic_listener::TopicListener},
};

// It is not made generic over the type so that it can be stored in the domain participant
pub struct TopicImpl {
    qos: TopicQos,
    type_name: &'static str,
    topic_name: String,
}

impl TopicImpl {
    pub fn new(qos: TopicQos, type_name: &'static str, topic_name: &str) -> Self {
        Self {
            qos,
            type_name,
            topic_name: topic_name.to_string(),
        }
    }

    pub fn set_qos(&mut self, qos: Option<TopicQos>) -> DDSResult<()> {
        let qos = qos.unwrap_or_default();
        qos.is_consistent()?;
        self.qos = qos;
        Ok(())
    }

    pub fn get_qos(&self) -> &TopicQos {
        &self.qos
    }
}

impl<Foo> Topic<Foo> for TopicImpl {
    fn get_inconsistent_topic_status(&self) -> DDSResult<InconsistentTopicStatus> {
        todo!()
    }
}

impl<Foo> TopicDescription<Foo> for TopicImpl {
    fn get_participant(&self) -> &dyn rust_dds_api::domain::domain_participant::DomainParticipant {
        todo!()
    }

    fn get_type_name(&self) -> DDSResult<&'static str> {
        Ok(&self.type_name)
    }

    fn get_name(&self) -> DDSResult<&'static str> {
        todo!()
    }
}

impl Entity for TopicImpl {
    type Qos = TopicQos;
    type Listener = ();

    fn set_qos(&mut self, qos: Option<Self::Qos>) -> DDSResult<()> {
        todo!()
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        todo!()
    }

    fn set_listener(
        &self,
        a_listener: Option<Self::Listener>,
        mask: rust_dds_api::dcps_psm::StatusMask,
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

    fn get_status_changes(&self) -> DDSResult<rust_dds_api::dcps_psm::StatusMask> {
        todo!()
    }

    fn enable(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> DDSResult<rust_dds_api::dcps_psm::InstanceHandle> {
        todo!()
    }
}
