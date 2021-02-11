use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use rust_dds_api::{
    dcps_psm::StatusMask, dds_type::DDSType, infrastructure::qos::DataWriterQos,
    publication::data_writer_listener::DataWriterListener,
};
use rust_rtps::{
    behavior::{stateless_writer::ReaderLocator, StatelessWriter},
    types::{
        constants::{
            ENTITY_KIND_BUILT_IN_WRITER_NO_KEY, ENTITY_KIND_BUILT_IN_WRITER_WITH_KEY,
            ENTITY_KIND_USER_DEFINED_WRITER_NO_KEY, ENTITY_KIND_USER_DEFINED_WRITER_WITH_KEY,
        },
        EntityId, GuidPrefix, Locator,
    },
};

use super::{
    rtps_datawriter_inner::{RtpsAnyDataWriterInner, RtpsDataWriterInner},
    rtps_topic_inner::RtpsTopicInner,
};

pub struct RtpsStatelessDataWriterInner {
    reader_locators: Mutex<HashMap<Locator, ReaderLocator>>,
    inner: Box<dyn RtpsAnyDataWriterInner>,
}

impl RtpsStatelessDataWriterInner {
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
        let inner = Box::new(RtpsDataWriterInner::new(
            guid_prefix,
            entity_id,
            topic,
            qos,
            listener,
            status_mask,
        ));

        Self {
            reader_locators: Mutex::new(HashMap::new()),
            inner,
        }
    }
}

impl StatelessWriter for RtpsStatelessDataWriterInner {
    fn reader_locator_add(&self, a_locator: Locator) {
        self.reader_locators
            .lock()
            .unwrap()
            .insert(a_locator, ReaderLocator::new(a_locator));
    }

    fn reader_locator_remove(&self, a_locator: &Locator) {
        self.reader_locators.lock().unwrap().remove(a_locator);
    }

    fn unsent_changes_reset(&self) {
        let mut reader_locators = self.reader_locators.lock().unwrap();
        for (_, rl) in reader_locators.iter_mut() {
            rl.unsent_changes_reset();
        }
    }
}
