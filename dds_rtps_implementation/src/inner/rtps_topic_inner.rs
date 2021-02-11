use std::sync::{Arc, Mutex};

use rust_dds_api::{
    dcps_psm::StatusMask,
    dds_type::DDSType,
    infrastructure::qos::TopicQos,
    return_type::{DDSError, DDSResult},
    topic::topic_listener::TopicListener,
};
use rust_rtps::types::{
    constants::ENTITY_KIND_USER_DEFINED_UNKNOWN, EntityId, GuidPrefix, TopicKind, GUID,
};

use crate::utils::maybe_valid::{MaybeValid, MaybeValidRef};

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
