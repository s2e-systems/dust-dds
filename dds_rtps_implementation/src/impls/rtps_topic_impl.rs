use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use rust_dds_api::{
    dcps_psm::StatusMask,
    dds_type::DDSType,
    infrastructure::qos::TopicQos,
    return_type::{DDSError, DDSResult},
    topic::topic_listener::TopicListener,
};
use rust_rtps::{structure::Entity, types::{
    constants::ENTITY_KIND_USER_DEFINED_UNKNOWN, EntityId, GuidPrefix, TopicKind, GUID,
}};

use crate::utils::maybe_valid::{MaybeValid, MaybeValidRef};

pub struct RtpsTopicImpl {
    entity: Entity,
    topic_name: String,
    type_name: &'static str,
    qos: TopicQos,
    listener: Option<Box<dyn TopicListener>>,
    status_mask: StatusMask,
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
            qos,
            listener,
            status_mask,
        }
    }

    pub fn get_type_name(&self) -> &str {
        self.type_name
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
}

pub type RtpsTopicInnerRef<'a> = MaybeValidRef<'a, Arc<RtpsTopicInner>>;

impl<'a> RtpsTopicInnerRef<'a> {
    pub fn get(&self) -> DDSResult<&Arc<RtpsTopicInner>> {
        MaybeValid::get(self).ok_or(DDSError::AlreadyDeleted)
    }

    pub fn delete(&self) -> DDSResult<()> {
        if Arc::strong_count(self.get()?) == 1 {
            MaybeValid::delete(self);
            Ok(())
        } else {
            Err(DDSError::PreconditionNotMet(
                "Topic still attached to some data reader or data writer",
            ))
        }
    }

    pub fn topic_kind(&self) -> DDSResult<TopicKind> {
        Ok(self.get()?.topic_kind)
    }

    pub fn set_qos(&self, qos: Option<TopicQos>) -> DDSResult<()> {
        let qos = qos.unwrap_or_default();
        qos.is_consistent()?;
        *self.get()?.qos.lock().unwrap() = qos;
        Ok(())
    }

    pub fn get_qos(&self) -> DDSResult<TopicQos> {
        Ok(self.get()?.qos.lock().unwrap().clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_type_name(){
        let entity_id = EntityId::new([1;3], ENTITY_KIND_USER_DEFINED_UNKNOWN);
        let guid = GUID::new([1;12], entity_id);
        let entity = Entity::new(guid);
        let type_name = "TestType";
        let topic_name = "TestTopic";
        let qos = TopicQos::default();
        let listener = None;
        let status_mask = 0;
        let topic = RtpsTopicImpl::new(entity, type_name, topic_name, qos, listener, status_mask);

        assert_eq!(topic.get_type_name(), type_name);
    }
}