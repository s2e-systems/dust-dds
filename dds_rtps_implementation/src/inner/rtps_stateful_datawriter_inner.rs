use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use rust_dds_api::{
    dcps_psm::StatusMask, dds_type::DDSType, infrastructure::qos::DataWriterQos,
    publication::data_writer_listener::DataWriterListener,
};
use rust_rtps::{
    behavior::{ReaderProxy, StatefulWriter},
    types::{
        constants::{
            ENTITY_KIND_BUILT_IN_WRITER_NO_KEY, ENTITY_KIND_BUILT_IN_WRITER_WITH_KEY,
            ENTITY_KIND_USER_DEFINED_WRITER_NO_KEY, ENTITY_KIND_USER_DEFINED_WRITER_WITH_KEY,
        },
        EntityId, GuidPrefix, GUID,
    },
};

use super::{
    rtps_datawriter_inner::{RtpsAnyDataWriterInner, RtpsDataWriterInner},
    rtps_topic_inner::RtpsTopicInner,
};

pub struct RtpsStatefulDataWriterInner {
    matched_readers: Mutex<HashMap<GUID, ReaderProxy>>,
    inner: Box<dyn RtpsAnyDataWriterInner>,
}

impl RtpsStatefulDataWriterInner {
    pub fn new_builtin<T: DDSType>(
        guid_prefix: GuidPrefix,
        entity_key: [u8; 3],
        topic: &Arc<RtpsTopicInner>,
        qos: DataWriterQos,
        listener: Option<Box<dyn DataWriterListener<DataType = T>>>,
        status_mask: StatusMask,
    ) -> Self {
        let entity_kind = match T::has_key() {
            false => ENTITY_KIND_BUILT_IN_WRITER_NO_KEY,
            true => ENTITY_KIND_BUILT_IN_WRITER_WITH_KEY,
        };
        let entity_id = EntityId::new(entity_key, entity_kind);
        Self::new(guid_prefix, entity_id, topic, qos, listener, status_mask)
    }

    pub fn new_user_defined<T: DDSType>(
        guid_prefix: GuidPrefix,
        entity_key: [u8; 3],
        topic: &Arc<RtpsTopicInner>,
        qos: DataWriterQos,
        listener: Option<Box<dyn DataWriterListener<DataType = T>>>,
        status_mask: StatusMask,
    ) -> Self {
        let entity_kind = match T::has_key() {
            false => ENTITY_KIND_USER_DEFINED_WRITER_NO_KEY,
            true => ENTITY_KIND_USER_DEFINED_WRITER_WITH_KEY,
        };
        let entity_id = EntityId::new(entity_key, entity_kind);
        Self::new(guid_prefix, entity_id, topic, qos, listener, status_mask)
    }

    fn new<T: DDSType>(
        guid_prefix: GuidPrefix,
        entity_id: EntityId,
        topic: &Arc<RtpsTopicInner>,
        qos: DataWriterQos,
        listener: Option<Box<dyn DataWriterListener<DataType = T>>>,
        status_mask: StatusMask,
    ) -> Self {
        let inner = Box::new(
            RtpsDataWriterInner::new(guid_prefix, entity_id, topic, qos, listener, status_mask));

        Self {
            matched_readers: Mutex::new(HashMap::new()),
            inner,
        }
    }
}

impl StatefulWriter for RtpsStatefulDataWriterInner {
    fn matched_reader_add(&self, a_reader_proxy: ReaderProxy) {
        let remote_reader_guid = a_reader_proxy.remote_reader_guid;
        self.matched_readers
            .lock()
            .unwrap()
            .insert(remote_reader_guid, a_reader_proxy);
    }

    fn matched_reader_remove(&self, reader_proxy_guid: &GUID) {
        self.matched_readers.lock().unwrap().remove(reader_proxy_guid);
    }

    fn matched_reader_lookup(&self, _a_reader_guid: GUID) -> Option<&ReaderProxy> {
        todo!()
    }

    fn is_acked_by_all(&self) -> bool {
        todo!()
    }
}