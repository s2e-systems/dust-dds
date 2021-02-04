use std::{
    any::Any,
    sync::{Arc, Mutex},
};

use rust_dds_api::{dcps_psm::StatusMask, dds_type::DDSType, infrastructure::{qos::TopicQos}, return_type::{DDSError, DDSResult}, topic::topic_listener::TopicListener};
use rust_rtps::types::{EntityId, GUID, GuidPrefix, TopicKind, constants::ENTITY_KIND_USER_DEFINED_UNKNOWN};

use crate::utils::{
    as_any::AsAny,
    maybe_valid::{MaybeValid, MaybeValidRef},
};

fn topic_kind_from_dds_type<T: DDSType>() -> TopicKind {
    match T::has_key() {
        false => TopicKind::NoKey,
        true => TopicKind::WithKey
    }
}

pub struct RtpsTopicInner<T: DDSType> {
    rtps_entity: rust_rtps::structure::Entity,
    topic_name: String,
    type_name: &'static str,
    topic_kind: TopicKind,
    qos: Mutex<TopicQos>,
    listener: Option<Box<dyn TopicListener<T>>>,
    status_mask: StatusMask,
}

impl<T: DDSType> RtpsTopicInner<T> {
    pub fn new(
        guid_prefix: GuidPrefix,
        entity_key: [u8;3],
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
            topic_kind: topic_kind_from_dds_type::<T>(),
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
    pub fn get(&self) -> DDSResult<&Arc<dyn RtpsAnyTopicInner>> {
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

    pub fn set_qos(&self, qos: Option<TopicQos>) -> DDSResult<()> {
        let qos = qos.unwrap_or_default();
        qos.is_consistent()?;
        *self.get()?.qos().lock().unwrap() = qos;
        Ok(())
    }

    pub fn get_qos(&self) -> DDSResult<TopicQos> {
        Ok(self.get()?.qos().lock().unwrap().clone())
    }
}
