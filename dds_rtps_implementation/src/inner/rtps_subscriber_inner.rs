use std::sync::{atomic, Arc, Mutex};

use rust_dds_api::{
    infrastructure::{
        qos::{DataReaderQos, SubscriberQos},
        status::StatusMask,
    },
    subscription::subscriber_listener::SubscriberListener,
};
use rust_dds_types::{DDSType, ReturnCode, ReturnCodes, TopicKind};
use rust_rtps::{
    structure::Group,
    types::{
        constants::{
            ENTITY_KIND_BUILT_IN_READER_NO_KEY, ENTITY_KIND_BUILT_IN_READER_WITH_KEY,
            ENTITY_KIND_USER_DEFINED_READER_NO_KEY, ENTITY_KIND_USER_DEFINED_READER_WITH_KEY,
        },
        EntityId, GUID,
    },
};

use crate::{
    utils::maybe_valid::{MaybeValid, MaybeValidList, MaybeValidRef},
};

use super::{rtps_datareader_inner::{AnyRtpsReader, RtpsAnyDataReaderRef, RtpsDataReaderInner}, rtps_topic_inner::RtpsAnyTopicInner};

enum EntityType {
    BuiltIn,
    UserDefined,
}
pub struct RtpsSubscriberInner {
    pub group: Group,
    pub reader_list: MaybeValidList<Box<dyn AnyRtpsReader>>,
    pub reader_count: atomic::AtomicU8,
    pub default_datareader_qos: Mutex<DataReaderQos>,
    pub qos: SubscriberQos,
    pub listener: Option<Box<dyn SubscriberListener>>,
    pub status_mask: StatusMask,
}

impl RtpsSubscriberInner {
    pub fn new(
        guid: GUID,
        qos: SubscriberQos,
        listener: Option<Box<dyn SubscriberListener>>,
        status_mask: StatusMask,
    ) -> Self {
        Self {
            group: Group::new(guid),
            reader_list: Default::default(),
            reader_count: atomic::AtomicU8::new(0),
            default_datareader_qos: Mutex::new(DataReaderQos::default()),
            qos,
            listener,
            status_mask,
        }
    }

    pub fn create_builtin_datareader<T: DDSType>(
        &self,
        a_topic: Arc<dyn RtpsAnyTopicInner>,
        qos: Option<DataReaderQos>,
        // _a_listener: impl DataReaderListener<T>,
        // _mask: StatusMask
    ) -> Option<RtpsAnyDataReaderRef> {
        self.create_datareader::<T>(a_topic, qos, EntityType::BuiltIn)
    }

    pub fn create_user_defined_datareader<T: DDSType>(
        &self,
        a_topic: Arc<dyn RtpsAnyTopicInner>,
        qos: Option<DataReaderQos>,
        // _a_listener: impl DataReaderListener<T>,
        // _mask: StatusMask
    ) -> Option<RtpsAnyDataReaderRef> {
        self.create_datareader::<T>(a_topic, qos, EntityType::BuiltIn)
    }

    fn create_datareader<T: DDSType>(
        &self,
        a_topic: Arc<dyn RtpsAnyTopicInner>,
        qos: Option<DataReaderQos>,
        entity_type: EntityType,
        // _a_listener: impl DataReaderListener<T>,
        // _mask: StatusMask
    ) -> Option<RtpsAnyDataReaderRef> {
        let guid_prefix = self.group.entity.guid.prefix();
        let entity_key = [
            0,
            self.reader_count.fetch_add(1, atomic::Ordering::Relaxed),
            0,
        ];
        let entity_kind = match (a_topic.topic_kind(), entity_type) {
            (TopicKind::WithKey, EntityType::UserDefined) => {
                ENTITY_KIND_USER_DEFINED_READER_WITH_KEY
            }
            (TopicKind::NoKey, EntityType::UserDefined) => ENTITY_KIND_USER_DEFINED_READER_NO_KEY,
            (TopicKind::WithKey, EntityType::BuiltIn) => ENTITY_KIND_BUILT_IN_READER_WITH_KEY,
            (TopicKind::NoKey, EntityType::BuiltIn) => ENTITY_KIND_BUILT_IN_READER_NO_KEY,
        };
        let entity_id = EntityId::new(entity_key, entity_kind);
        let new_reader_guid = GUID::new(guid_prefix, entity_id);
        let new_reader_qos = qos.unwrap_or(self.get_default_datareader_qos());
        let new_reader: Box<RtpsDataReaderInner<T>> = Box::new(RtpsDataReaderInner::new(
            new_reader_guid,
            a_topic,
            new_reader_qos,
            None,
            0,
        ));
        self.reader_list.add(new_reader)
    }

    pub fn get_default_datareader_qos(&self) -> DataReaderQos {
        self.default_datareader_qos.lock().unwrap().clone()
    }

    pub fn set_default_datawriter_qos(&self, qos: Option<DataReaderQos>) -> ReturnCode<()> {
        let datareader_qos = qos.unwrap_or_default();
        datareader_qos.is_consistent()?;
        *self.default_datareader_qos.lock().unwrap() = datareader_qos;
        Ok(())
    }
}

pub type RtpsSubscriberRef<'a> = MaybeValidRef<'a, Box<RtpsSubscriberInner>>;

impl<'a> RtpsSubscriberRef<'a> {
    pub(crate) fn get(&self) -> ReturnCode<&Box<RtpsSubscriberInner>> {
        MaybeValid::get(self).ok_or(ReturnCodes::AlreadyDeleted)
    }

    pub(crate) fn delete(&self) {
        MaybeValid::delete(self)
    }

    pub(crate) fn get_qos(&self) -> ReturnCode<SubscriberQos> {
        Ok(self.get()?.qos.clone())
    }
}
