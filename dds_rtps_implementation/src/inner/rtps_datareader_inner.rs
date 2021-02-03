use std::{
    any::Any,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

use rust_dds_api::{
    infrastructure::{
        qos::DataReaderQos, qos_policy::ReliabilityQosPolicyKind, status::StatusMask,
    },
    subscription::data_reader_listener::DataReaderListener,
};
use rust_dds_types::{DDSType, ReturnCode, ReturnCodes, TopicKind};
use rust_rtps::{
    behavior::{self, Reader, StatefulReader, StatelessReader},
    types::{
        constants::{
            ENTITY_KIND_BUILT_IN_READER_NO_KEY, ENTITY_KIND_BUILT_IN_READER_WITH_KEY,
            ENTITY_KIND_USER_DEFINED_READER_NO_KEY, ENTITY_KIND_USER_DEFINED_READER_WITH_KEY,
        },
        EntityId, EntityKey, GuidPrefix, ReliabilityKind, GUID,
    },
};

use crate::utils::{
    as_any::AsAny,
    maybe_valid::{MaybeValid, MaybeValidRef},
};

use super::rtps_topic_inner::{RtpsAnyTopicInner, RtpsAnyTopicInnerRef};

pub enum ReaderFlavor {
    Stateful(StatefulReader),
    Stateless(StatelessReader),
}
impl ReaderFlavor {
    pub fn try_get_stateless(&mut self) -> Option<&mut StatelessReader> {
        match self {
            ReaderFlavor::Stateless(reader) => Some(reader),
            ReaderFlavor::Stateful(_) => None,
        }
    }

    pub fn try_get_stateful(&mut self) -> Option<&mut StatefulReader> {
        match self {
            ReaderFlavor::Stateless(_) => None,
            ReaderFlavor::Stateful(reader) => Some(reader),
        }
    }
}
impl Deref for ReaderFlavor {
    type Target = Reader;

    fn deref(&self) -> &Self::Target {
        match self {
            ReaderFlavor::Stateful(reader) => reader,
            ReaderFlavor::Stateless(reader) => reader,
        }
    }
}
impl DerefMut for ReaderFlavor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            ReaderFlavor::Stateful(reader) => reader,
            ReaderFlavor::Stateless(reader) => reader,
        }
    }
}

pub struct RtpsDataReaderInner<T: DDSType> {
    pub reader: Mutex<ReaderFlavor>,
    pub qos: Mutex<DataReaderQos>,
    pub topic: Mutex<Option<Arc<dyn RtpsAnyTopicInner>>>,
    pub listener: Option<Box<dyn DataReaderListener<T>>>,
    pub status_mask: StatusMask,
}

impl<T: DDSType> RtpsDataReaderInner<T> {
    pub fn new_builtin_stateless(
        guid_prefix: GuidPrefix,
        entity_key: EntityKey,
        topic: &RtpsAnyTopicInnerRef,
        qos: DataReaderQos,
        listener: Option<Box<dyn DataReaderListener<T>>>,
        status_mask: StatusMask,
    ) -> Self {
        let entity_kind = match T::topic_kind() {
            TopicKind::NoKey => ENTITY_KIND_BUILT_IN_READER_NO_KEY,
            TopicKind::WithKey => ENTITY_KIND_BUILT_IN_READER_WITH_KEY,
        };
        let entity_id = EntityId::new(entity_key, entity_kind);
        let guid = GUID::new(guid_prefix, entity_id);
        Self::new_stateless(guid, topic, qos, listener, status_mask)
    }

    pub fn new_user_defined_stateless(
        guid_prefix: GuidPrefix,
        entity_key: EntityKey,
        topic: &RtpsAnyTopicInnerRef,
        qos: DataReaderQos,
        listener: Option<Box<dyn DataReaderListener<T>>>,
        status_mask: StatusMask,
    ) -> Self {
        let entity_kind = match T::topic_kind() {
            TopicKind::NoKey => ENTITY_KIND_USER_DEFINED_READER_NO_KEY,
            TopicKind::WithKey => ENTITY_KIND_USER_DEFINED_READER_WITH_KEY,
        };
        let entity_id = EntityId::new(entity_key, entity_kind);
        let guid = GUID::new(guid_prefix, entity_id);
        Self::new_stateless(guid, topic, qos, listener, status_mask)
    }

    pub fn new_builtin_stateful(
        guid_prefix: GuidPrefix,
        entity_key: EntityKey,
        topic: &RtpsAnyTopicInnerRef,
        qos: DataReaderQos,
        listener: Option<Box<dyn DataReaderListener<T>>>,
        status_mask: StatusMask,
    ) -> Self {
        let entity_kind = match T::topic_kind() {
            TopicKind::NoKey => ENTITY_KIND_BUILT_IN_READER_NO_KEY,
            TopicKind::WithKey => ENTITY_KIND_BUILT_IN_READER_WITH_KEY,
        };
        let entity_id = EntityId::new(entity_key, entity_kind);
        let guid = GUID::new(guid_prefix, entity_id);
        Self::new_stateful(guid, topic, qos, listener, status_mask)
    }

    pub fn new_user_defined_stateful(
        guid_prefix: GuidPrefix,
        entity_key: EntityKey,
        topic: &RtpsAnyTopicInnerRef,
        qos: DataReaderQos,
        listener: Option<Box<dyn DataReaderListener<T>>>,
        status_mask: StatusMask,
    ) -> Self {
        let entity_kind = match T::topic_kind() {
            TopicKind::NoKey => ENTITY_KIND_BUILT_IN_READER_NO_KEY,
            TopicKind::WithKey => ENTITY_KIND_BUILT_IN_READER_WITH_KEY,
        };
        let entity_id = EntityId::new(entity_key, entity_kind);
        let guid = GUID::new(guid_prefix, entity_id);
        Self::new_stateful(guid, topic, qos, listener, status_mask)
    }

    fn new_stateful(
        guid: GUID,
        topic: &RtpsAnyTopicInnerRef,
        qos: DataReaderQos,
        listener: Option<Box<dyn DataReaderListener<T>>>,
        status_mask: StatusMask,
    ) -> Self {
        assert!(
            qos.is_consistent().is_ok(),
            "RtpsDataReader can only be created with consistent QoS"
        );
        let topic = topic.get().unwrap().clone();
        let topic_kind = topic.topic_kind();
        let reliability_level = match qos.reliability.kind {
            ReliabilityQosPolicyKind::BestEffortReliabilityQos => ReliabilityKind::BestEffort,
            ReliabilityQosPolicyKind::ReliableReliabilityQos => ReliabilityKind::Reliable,
        };
        let expects_inline_qos = false;
        let heartbeat_response_delay = behavior::types::Duration::from_millis(200);
        let reader = StatefulReader::new(
            guid,
            topic_kind,
            reliability_level,
            expects_inline_qos,
            heartbeat_response_delay,
        );

        Self {
            reader: Mutex::new(ReaderFlavor::Stateful(reader)),
            qos: Mutex::new(qos),
            topic: Mutex::new(Some(topic)),
            listener,
            status_mask,
        }
    }

    fn new_stateless(
        guid: GUID,
        topic: &RtpsAnyTopicInnerRef,
        qos: DataReaderQos,
        listener: Option<Box<dyn DataReaderListener<T>>>,
        status_mask: StatusMask,
    ) -> Self {
        assert!(
            qos.is_consistent().is_ok(),
            "RtpsDataReader can only be created with consistent QoS"
        );
        let topic = topic.get().unwrap().clone();
        let topic_kind = topic.topic_kind();
        let reliability_level = match qos.reliability.kind {
            ReliabilityQosPolicyKind::BestEffortReliabilityQos => ReliabilityKind::BestEffort,
            ReliabilityQosPolicyKind::ReliableReliabilityQos => ReliabilityKind::Reliable,
        };
        let expects_inline_qos = false;
        let reader = StatelessReader::new(guid, topic_kind, reliability_level, expects_inline_qos);

        Self {
            reader: Mutex::new(ReaderFlavor::Stateless(reader)),
            qos: Mutex::new(qos),
            topic: Mutex::new(Some(topic)),
            listener,
            status_mask,
        }
    }
}

pub trait RtpsAnyDataReaderInner: AsAny + Send + Sync {
    fn reader(&self) -> &Mutex<ReaderFlavor>;
    fn qos(&self) -> &Mutex<DataReaderQos>;
    fn topic(&self) -> &Mutex<Option<Arc<dyn RtpsAnyTopicInner>>>;
    fn status_mask(&self) -> &StatusMask;
}

impl<T: DDSType + Sized> RtpsAnyDataReaderInner for RtpsDataReaderInner<T> {
    fn reader(&self) -> &Mutex<ReaderFlavor> {
        &self.reader
    }

    fn qos(&self) -> &Mutex<DataReaderQos> {
        &self.qos
    }

    fn topic(&self) -> &Mutex<Option<Arc<dyn RtpsAnyTopicInner>>> {
        &self.topic
    }

    fn status_mask(&self) -> &StatusMask {
        &self.status_mask
    }
}

impl<T: DDSType + Sized> AsAny for RtpsDataReaderInner<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub type RtpsAnyDataReaderInnerRef<'a> = MaybeValidRef<'a, Box<dyn RtpsAnyDataReaderInner>>;

impl<'a> RtpsAnyDataReaderInnerRef<'a> {
    pub fn get(&self) -> ReturnCode<&Box<dyn RtpsAnyDataReaderInner>> {
        MaybeValid::get(self).ok_or(ReturnCodes::AlreadyDeleted)
    }

    pub fn get_as<U: DDSType>(&self) -> ReturnCode<&RtpsDataReaderInner<U>> {
        self.get()?
            .as_ref()
            .as_any()
            .downcast_ref()
            .ok_or(ReturnCodes::Error)
    }

    pub fn delete(&self) {
        MaybeValid::delete(self)
    }
}
