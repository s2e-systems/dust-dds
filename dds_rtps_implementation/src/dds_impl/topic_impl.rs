use rust_dds_api::{
    infrastructure::qos::TopicQos,
    return_type::DDSResult,
    topic::{topic::Topic, topic_description::TopicDescription},
};

pub struct TopicImpl {
    qos: TopicQos,
}

impl TopicImpl {
    pub fn new(qos: TopicQos) -> Self {
        Self { qos }
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

impl<T> Topic<T> for TopicImpl {
    fn get_inconsistent_topic_status(
        &self,
        _status: &mut rust_dds_api::dcps_psm::InconsistentTopicStatus,
    ) -> DDSResult<()> {
        todo!()
    }
}

impl<T> TopicDescription<T> for TopicImpl {
    fn get_participant(&self) -> &dyn rust_dds_api::domain::domain_participant::DomainParticipant {
        todo!()
    }

    fn get_type_name(&self) -> DDSResult<&'static str> {
        todo!()
    }

    fn get_name(&self) -> DDSResult<&'static str> {
        todo!()
    }
}
