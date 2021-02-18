use std::sync::Mutex;

use rust_dds_api::{
    dcps_psm::StatusMask, dds_type::DDSType, infrastructure::qos::TopicQos, return_type::DDSResult,
    topic::topic_listener::TopicListener,
};
use rust_rtps::{
    structure::Entity,
    types::{constants::ENTITY_KIND_USER_DEFINED_UNKNOWN, EntityId, GuidPrefix, TopicKind, GUID},
};

use super::mask_listener::MaskListener;

pub struct RtpsTopicImpl {
    entity: Entity,
    topic_name: String,
    type_name: &'static str,
    qos: Mutex<TopicQos>,
    listener: Mutex<MaskListener<Box<dyn TopicListener>>>,
}

impl RtpsTopicImpl {
    pub fn new(
        entity: Entity,
        type_name: &'static str,
        topic_name: &str,
        qos: TopicQos,
        listener: Option<Box<dyn TopicListener>>,
        status_mask: StatusMask,
    ) -> Self {
        Self {
            entity,
            topic_name: topic_name.to_string(),
            type_name,
            qos: Mutex::new(qos),
            listener: Mutex::new(MaskListener::new(listener, status_mask)),
        }
    }

    pub fn get_type_name(&self) -> &str {
        self.type_name
    }

    pub fn get_name(&self) -> &str {
        &self.topic_name
    }

    pub fn set_qos(&self, qos: Option<TopicQos>) -> DDSResult<()> {
        let qos = qos.unwrap_or_default();
        qos.is_consistent()?;
        *self.qos.lock().unwrap() = qos;
        Ok(())
    }

    pub fn get_qos(&self) -> TopicQos {
        self.qos.lock().unwrap().clone()
    }

    pub fn get_listener(&self) -> Option<Box<dyn TopicListener>> {
        self.listener.lock().unwrap().take()
    }

    pub fn set_listener(&self, a_listener: Option<Box<dyn TopicListener>>, mask: StatusMask) {
        self.listener.lock().unwrap().set(a_listener, mask)
    }
}

fn topic_kind_from_dds_type<T: DDSType>() -> TopicKind {
    match T::has_key() {
        false => TopicKind::NoKey,
        true => TopicKind::WithKey,
    }
}

pub struct RtpsTopicInner {
    rtps_entity: rust_rtps::structure::Entity,
    topic_name: String,
    type_name: &'static str,
    topic_kind: TopicKind,
    qos: Mutex<TopicQos>,
    listener: Option<Box<dyn TopicListener>>,
    status_mask: StatusMask,
}

impl RtpsTopicInner {
    pub fn new(
        guid_prefix: GuidPrefix,
        entity_key: [u8; 3],
        topic_name: String,
        type_name: &'static str,
        topic_kind: TopicKind,
        qos: TopicQos,
        listener: Option<Box<dyn TopicListener>>,
        status_mask: StatusMask,
    ) -> Self {
        let guid = GUID::new(
            guid_prefix,
            EntityId::new(entity_key, ENTITY_KIND_USER_DEFINED_UNKNOWN),
        );
        Self {
            rtps_entity: rust_rtps::structure::Entity { guid },
            topic_name,
            type_name,
            topic_kind,
            qos: Mutex::new(qos),
            listener,
            status_mask,
        }
    }

    pub fn topic_kind(&self) -> TopicKind {
        self.topic_kind
    }

    // pub fn delete(&self) -> DDSResult<()> {
    //     if Arc::strong_count(self.get()?) == 1 {
    //         MaybeValid::delete(self);
    //         Ok(())
    //     } else {
    //         Err(DDSError::PreconditionNotMet(
    //             "Topic still attached to some data reader or data writer",
    //         ))
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use rust_dds_api::{infrastructure::listener::Listener, return_type::DDSError};

    use super::*;

    #[test]
    fn get_type_name() {
        let entity_id = EntityId::new([1; 3], ENTITY_KIND_USER_DEFINED_UNKNOWN);
        let guid = GUID::new([1; 12], entity_id);
        let entity = Entity::new(guid);
        let type_name = "TestType";
        let topic_name = "TestTopic";
        let qos = TopicQos::default();
        let listener = None;
        let status_mask = 0;
        let topic = RtpsTopicImpl::new(entity, type_name, topic_name, qos, listener, status_mask);

        assert_eq!(topic.get_type_name(), type_name);
    }

    #[test]
    fn get_name() {
        let entity_id = EntityId::new([1; 3], ENTITY_KIND_USER_DEFINED_UNKNOWN);
        let guid = GUID::new([1; 12], entity_id);
        let entity = Entity::new(guid);
        let type_name = "TestType";
        let topic_name = "TestTopic";
        let qos = TopicQos::default();
        let listener = None;
        let status_mask = 0;
        let topic = RtpsTopicImpl::new(entity, type_name, topic_name, qos, listener, status_mask);

        assert_eq!(topic.get_name(), topic_name);
    }

    #[test]
    fn get_qos() {
        let entity_id = EntityId::new([1; 3], ENTITY_KIND_USER_DEFINED_UNKNOWN);
        let guid = GUID::new([1; 12], entity_id);
        let entity = Entity::new(guid);
        let type_name = "TestType";
        let topic_name = "TestTopic";
        let mut qos = TopicQos::default();
        qos.topic_data.value = vec![1, 2, 3, 4];
        let listener = None;
        let status_mask = 0;
        let topic = RtpsTopicImpl::new(
            entity,
            type_name,
            topic_name,
            qos.clone(),
            listener,
            status_mask,
        );

        assert_eq!(topic.get_qos(), qos);
    }

    #[test]
    fn set_qos() {
        let entity_id = EntityId::new([1; 3], ENTITY_KIND_USER_DEFINED_UNKNOWN);
        let guid = GUID::new([1; 12], entity_id);
        let entity = Entity::new(guid);
        let type_name = "TestType";
        let topic_name = "TestTopic";
        let mut qos = TopicQos::default();
        qos.topic_data.value = vec![1, 2, 3, 4];
        let listener = None;
        let status_mask = 0;
        let topic = RtpsTopicImpl::new(
            entity,
            type_name,
            topic_name,
            TopicQos::default(),
            listener,
            status_mask,
        );

        topic
            .set_qos(Some(qos.clone()))
            .expect("Error setting Topic QoS");
        assert_eq!(topic.get_qos(), qos);
    }

    #[test]
    fn set_inconsistent_qos() {
        let entity_id = EntityId::new([1; 3], ENTITY_KIND_USER_DEFINED_UNKNOWN);
        let guid = GUID::new([1; 12], entity_id);
        let entity = Entity::new(guid);
        let type_name = "TestType";
        let topic_name = "TestTopic";
        let mut inconsistent_qos = TopicQos::default();
        inconsistent_qos.resource_limits.max_samples_per_instance = 10;
        inconsistent_qos.resource_limits.max_samples = 5;
        let listener = None;
        let status_mask = 0;
        let topic = RtpsTopicImpl::new(
            entity,
            type_name,
            topic_name,
            TopicQos::default(),
            listener,
            status_mask,
        );

        let result = topic.set_qos(Some(inconsistent_qos));
        assert_eq!(result, Err(DDSError::InconsistentPolicy));
    }

    #[test]
    fn set_and_get_listener() {
        struct TestListener;

        impl Listener for TestListener {}

        impl TopicListener for TestListener {
            fn on_inconsistent_topic(
                &self,
                _the_topic: &dyn rust_dds_api::topic::topic::Topic,
                _status: rust_dds_api::dcps_psm::InconsistentTopicStatus,
            ) {
                todo!()
            }
        }

        let entity_id = EntityId::new([1; 3], ENTITY_KIND_USER_DEFINED_UNKNOWN);
        let guid = GUID::new([1; 12], entity_id);
        let entity = Entity::new(guid);
        let type_name = "TestType";
        let topic_name = "TestTopic";
        let qos = TopicQos::default();
        let listener = Box::new(TestListener);
        let status_mask = 0;
        let topic = RtpsTopicImpl::new(
            entity,
            type_name,
            topic_name,
            qos.clone(),
            Some(listener),
            status_mask,
        );

        assert!(topic.get_listener().is_some());
        // Get listener is a take operation so it leaves no listener behind
        assert!(topic.get_listener().is_none());

        topic.set_listener(Some(Box::new(TestListener)), 10);
        assert!(topic.get_listener().is_some());
    }
}
