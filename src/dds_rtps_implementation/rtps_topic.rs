use crate::dds_infrastructure::qos::TopicQos;
use crate::dds_rtps_implementation::discovery::sedp::SimpleEndpointDiscoveryProtocol;
use crate::dds_rtps_implementation::rtps_object::RtpsObject;
use crate::rtps::structure::Entity;
use crate::rtps::types::GUID;
use crate::types::{InstanceHandle, ReturnCode, TopicKind};
use std::sync::{Arc, Mutex, RwLockReadGuard};

pub struct RtpsTopicInner {
    pub entity: Entity,
    pub topic_name: String,
    pub type_name: &'static str,
    pub topic_kind: TopicKind,
    pub qos: Mutex<TopicQos>,
}

impl RtpsTopicInner {
    pub fn new(
        guid: GUID,
        topic_name: String,
        type_name: &'static str,
        topic_kind: TopicKind,
        qos: TopicQos,
    ) -> Self {
        Self {
            entity: Entity { guid },
            topic_name,
            type_name,
            topic_kind,
            qos: Mutex::new(qos),
        }
    }
}

pub type RtpsTopic<'a> = RwLockReadGuard<'a, RtpsObject<Arc<RtpsTopicInner>>>;

impl RtpsObject<Arc<RtpsTopicInner>> {
    pub fn get_name(&self) -> ReturnCode<String> {
        Ok(self.value()?.topic_name.clone())
    }

    pub fn get_type_name(&self) -> ReturnCode<&str> {
        Ok(self.value()?.type_name)
    }

    pub fn get_qos(&self) -> ReturnCode<TopicQos> {
        Ok(self.value()?.qos.lock().unwrap().clone())
    }

    pub fn set_qos(
        &self,
        qos: TopicQos,
        discovery: &SimpleEndpointDiscoveryProtocol,
    ) -> ReturnCode<()> {
        let topic = self.value()?;
        qos.is_consistent()?;
        *topic.qos.lock().unwrap() = qos;
        discovery.update_topic(topic)?;
        Ok(())
    }

    pub fn enable(&self) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_instance_handle(&self) -> ReturnCode<InstanceHandle> {
        Ok(self.value()?.entity.guid.into())
    }
}
