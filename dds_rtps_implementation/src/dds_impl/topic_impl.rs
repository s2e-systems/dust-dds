use rust_dds_api::{infrastructure::qos::TopicQos, return_type::DDSResult};

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
