use std::{
    any::Any,
    sync::{Arc, Mutex},
};

use rust_dds_api::{
    infrastructure::{qos::TopicQos, status::StatusMask},
    topic::topic_listener::TopicListener,
};
use rust_dds_types::{DDSType, ReturnCode, ReturnCodes, TopicKind};
use rust_rtps::types::{EntityId, EntityKey, GUID, GuidPrefix, constants::ENTITY_KIND_USER_DEFINED_UNKNOWN};

use crate::utils::{
    as_any::AsAny,
    maybe_valid::{MaybeValid, MaybeValidRef},
};

pub struct RtpsTopicInner<T: DDSType> {
    pub rtps_entity: rust_rtps::structure::Entity,
    pub topic_name: String,
    pub type_name: &'static str,
    pub topic_kind: TopicKind,
    pub qos: Mutex<TopicQos>,
    pub listener: Option<Box<dyn TopicListener<T>>>,
    pub status_mask: StatusMask,
}

impl<T: DDSType> RtpsTopicInner<T> {
    pub fn new(
        guid_prefix: GuidPrefix,
        entity_key: EntityKey,
        topic_name: String,
        qos: TopicQos,
        listener: Option<Box<dyn TopicListener<T>>>,
        status_mask: StatusMask,
    ) -> Self {
        let guid = GUID::new(guid_prefix, EntityId::new(entity_key, ENTITY_KIND_USER_DEFINED_UNKNOWN));
        Self {
            rtps_entity: rust_rtps::structure::Entity { guid },
            topic_name,
            type_name: T::type_name(),
            topic_kind: T::topic_kind(),
            qos: Mutex::new(qos),
            listener,
            status_mask,
        }
    }
}

pub trait RtpsAnyTopicInner: AsAny + Send + Sync {
    fn rtps_entity(&self) -> &rust_rtps::structure::Entity;
    fn topic_kind(&self) -> TopicKind;
    fn type_name(&self) -> &'static str;
    fn topic_name(&self) -> &String;
    fn qos(&self) -> &Mutex<TopicQos>;
}

impl<T: DDSType> RtpsAnyTopicInner for RtpsTopicInner<T> {
    fn rtps_entity(&self) -> &rust_rtps::structure::Entity {
        &self.rtps_entity
    }

    fn topic_kind(&self) -> TopicKind {
        self.topic_kind
    }

    fn type_name(&self) -> &'static str {
        &self.type_name
    }

    fn topic_name(&self) -> &String {
        &self.topic_name
    }

    fn qos(&self) -> &Mutex<TopicQos> {
        &self.qos
    }
}

impl<T: DDSType> AsAny for RtpsTopicInner<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub type RtpsAnyTopicInnerRef<'a> = MaybeValidRef<'a, Arc<dyn RtpsAnyTopicInner>>;

impl<'a> RtpsAnyTopicInnerRef<'a> {
    pub fn get(&self) -> ReturnCode<&Arc<dyn RtpsAnyTopicInner>> {
        MaybeValid::get(self).ok_or(ReturnCodes::AlreadyDeleted)
    }

    pub fn delete(&self) -> ReturnCode<()> {
        let rtps_topic = self.get()?;
        if Arc::strong_count(rtps_topic) == 1 {
            MaybeValid::delete(self);
            Ok(())
        } else {
            Err(ReturnCodes::PreconditionNotMet(
                "Topic still attached to some data reader or data writer",
            ))
        }
    }

    pub fn set_qos(&self, qos: Option<TopicQos>) -> ReturnCode<()> {
        let qos = qos.unwrap_or_default();
        qos.is_consistent()?;
        *self.get()?.qos().lock().unwrap() = qos;
        Ok(())
    }

    pub fn get_qos(&self) -> ReturnCode<TopicQos> {
        Ok(self.get()?.qos().lock().unwrap().clone())
    }
}
