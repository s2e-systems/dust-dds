use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex, MutexGuard},
};

use behavior::types::constants::DURATION_ZERO;
use rust_dds_api::{
    dcps_psm::StatusMask,
    dds_type::DDSType,
    infrastructure::{qos::DataReaderQos, qos_policy::ReliabilityQosPolicyKind},
    return_type::{DDSError, DDSResult},
    subscription::data_reader_listener::DataReaderListener,
};
use rust_rtps::{
    behavior::{self, Reader, StatefulReader, StatelessReader},
    types::{
        constants::{
            ENTITY_KIND_BUILT_IN_READER_NO_KEY, ENTITY_KIND_BUILT_IN_READER_WITH_KEY,
            ENTITY_KIND_USER_DEFINED_READER_NO_KEY, ENTITY_KIND_USER_DEFINED_READER_WITH_KEY,
        },
        EntityId, GuidPrefix, ReliabilityKind, GUID,
    },
};

use crate::{
    rtps_datareader::RtpsDataReader,
    utils::maybe_valid::{MaybeValid, MaybeValidRef},
};

use super::rtps_topic_inner::{RtpsTopicInner, RtpsTopicInnerRef};

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

pub struct RtpsDataReaderInner<'a, T: DDSType> {
    reader: Mutex<ReaderFlavor>,
    qos: Mutex<DataReaderQos>,
    topic: Mutex<Option<Arc<RtpsTopicInner>>>,
    listener: Option<Box<dyn DataReaderListener<DataType = T> + 'a>>,
    status_mask: StatusMask,
}

impl<'a, T: DDSType> RtpsDataReaderInner<'a, T> {
    pub fn new_builtin_stateless(
        guid_prefix: GuidPrefix,
        entity_key: [u8; 3],
        topic: &RtpsTopicInnerRef,
        qos: DataReaderQos,
        listener: Option<Box<dyn DataReaderListener<DataType = T>>>,
        status_mask: StatusMask,
    ) -> Self {
        let entity_kind = match T::has_key() {
            false => ENTITY_KIND_BUILT_IN_READER_NO_KEY,
            true => ENTITY_KIND_BUILT_IN_READER_WITH_KEY,
        };
        let entity_id = EntityId::new(entity_key, entity_kind);
        let guid = GUID::new(guid_prefix, entity_id);
        Self::new_stateless(guid, topic, qos, listener, status_mask)
    }

    pub fn new_user_defined_stateless(
        guid_prefix: GuidPrefix,
        entity_key: [u8; 3],
        topic: &RtpsTopicInnerRef,
        qos: DataReaderQos,
        listener: Option<Box<dyn DataReaderListener<DataType = T>>>,
        status_mask: StatusMask,
    ) -> Self {
        let entity_kind = match T::has_key() {
            false => ENTITY_KIND_USER_DEFINED_READER_NO_KEY,
            true => ENTITY_KIND_USER_DEFINED_READER_WITH_KEY,
        };
        let entity_id = EntityId::new(entity_key, entity_kind);
        let guid = GUID::new(guid_prefix, entity_id);
        Self::new_stateless(guid, topic, qos, listener, status_mask)
    }

    pub fn new_builtin_stateful(
        guid_prefix: GuidPrefix,
        entity_key: [u8; 3],
        topic: &RtpsTopicInnerRef,
        qos: DataReaderQos,
        listener: Option<Box<dyn DataReaderListener<DataType = T>>>,
        status_mask: StatusMask,
    ) -> Self {
        let entity_kind = match T::has_key() {
            false => ENTITY_KIND_BUILT_IN_READER_NO_KEY,
            true => ENTITY_KIND_BUILT_IN_READER_WITH_KEY,
        };
        let entity_id = EntityId::new(entity_key, entity_kind);
        let guid = GUID::new(guid_prefix, entity_id);
        Self::new_stateful(guid, topic, qos, listener, status_mask)
    }

    pub fn new_user_defined_stateful(
        guid_prefix: GuidPrefix,
        entity_key: [u8; 3],
        topic: &RtpsTopicInnerRef,
        qos: DataReaderQos,
        listener: Option<Box<dyn DataReaderListener<DataType = T>>>,
        status_mask: StatusMask,
    ) -> Self {
        let entity_kind = match T::has_key() {
            false => ENTITY_KIND_BUILT_IN_READER_NO_KEY,
            true => ENTITY_KIND_BUILT_IN_READER_WITH_KEY,
        };
        let entity_id = EntityId::new(entity_key, entity_kind);
        let guid = GUID::new(guid_prefix, entity_id);
        Self::new_stateful(guid, topic, qos, listener, status_mask)
    }

    fn new_stateful(
        guid: GUID,
        topic: &RtpsTopicInnerRef,
        qos: DataReaderQos,
        listener: Option<Box<dyn DataReaderListener<DataType = T>>>,
        status_mask: StatusMask,
    ) -> Self {
        assert!(
            qos.is_consistent().is_ok(),
            "RtpsDataReader can only be created with consistent QoS"
        );
        let topic_kind = topic.topic_kind().unwrap();
        let reliability_level = match qos.reliability.kind {
            ReliabilityQosPolicyKind::BestEffortReliabilityQos => ReliabilityKind::BestEffort,
            ReliabilityQosPolicyKind::ReliableReliabilityQos => ReliabilityKind::Reliable,
        };
        let expects_inline_qos = false;
        let heartbeat_response_delay = behavior::types::Duration::from_millis(200);
        let heartbeat_supression_duration = DURATION_ZERO;
        let reader = StatefulReader::new(
            guid,
            topic_kind,
            reliability_level,
            expects_inline_qos,
            heartbeat_response_delay,
            heartbeat_supression_duration,
        );
        let topic = topic.get().unwrap().clone();

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
        topic: &RtpsTopicInnerRef,
        qos: DataReaderQos,
        listener: Option<Box<dyn DataReaderListener<DataType = T>>>,
        status_mask: StatusMask,
    ) -> Self {
        assert!(
            qos.is_consistent().is_ok(),
            "RtpsDataReader can only be created with consistent QoS"
        );
        let topic_kind = topic.topic_kind().unwrap();
        let reliability_level = match qos.reliability.kind {
            ReliabilityQosPolicyKind::BestEffortReliabilityQos => ReliabilityKind::BestEffort,
            ReliabilityQosPolicyKind::ReliableReliabilityQos => ReliabilityKind::Reliable,
        };
        let expects_inline_qos = false;
        let heartbeat_response_delay = behavior::types::Duration::from_millis(200);
        let heartbeat_supression_duration = DURATION_ZERO;
        let reader = StatelessReader::new(
            guid,
            topic_kind,
            reliability_level,
            expects_inline_qos,
            heartbeat_response_delay,
            heartbeat_supression_duration,
        );
        let topic = topic.get().unwrap().clone();

        Self {
            reader: Mutex::new(ReaderFlavor::Stateless(reader)),
            qos: Mutex::new(qos),
            topic: Mutex::new(Some(topic)),
            listener,
            status_mask,
        }
    }
}

pub trait RtpsAnyDataReaderInner: Send + Sync {
    fn reader(&self) -> &Mutex<ReaderFlavor>;
    fn qos(&self) -> &Mutex<DataReaderQos>;
    fn topic(&self) -> MutexGuard<Option<Arc<RtpsTopicInner>>>;
    fn status_mask(&self) -> &StatusMask;
    fn on_data_available(&self, data_reader_ref: RtpsAnyDataReaderInnerRef);
}

impl<'a, T: DDSType + Sized> RtpsAnyDataReaderInner for RtpsDataReaderInner<'a, T> {
    fn reader(&self) -> &Mutex<ReaderFlavor> {
        &self.reader
    }

    fn qos(&self) -> &Mutex<DataReaderQos> {
        &self.qos
    }

    fn topic(&self) -> MutexGuard<Option<Arc<RtpsTopicInner>>> {
        self.topic.lock().unwrap()
    }

    fn status_mask(&self) -> &StatusMask {
        &self.status_mask
    }

    fn on_data_available(&self, data_reader_ref: RtpsAnyDataReaderInnerRef) {
        let the_reader = RtpsDataReader {
            parent_subscriber: None,
            data_reader_ref,
            phantom_data: PhantomData,
        };
        self.listener
            .as_ref()
            .unwrap()
            .on_data_available(&the_reader)
    }
}

pub type RtpsAnyDataReaderInnerRef<'a> = MaybeValidRef<'a, Box<dyn RtpsAnyDataReaderInner>>;

impl<'a> RtpsAnyDataReaderInnerRef<'a> {
    pub fn get(&self) -> DDSResult<&Box<dyn RtpsAnyDataReaderInner>> {
        MaybeValid::get(self).ok_or(DDSError::AlreadyDeleted)
    }

    pub fn delete(&self) -> DDSResult<()> {
        self.get()?.topic().take(); // Drop the topic
        MaybeValid::delete(self);
        Ok(())
    }
}
