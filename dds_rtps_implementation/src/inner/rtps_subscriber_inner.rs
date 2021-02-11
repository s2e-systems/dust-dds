use std::sync::{atomic, Mutex};

use rust_dds_api::{
    dcps_psm::StatusMask,
    dds_type::DDSType,
    infrastructure::qos::{DataReaderQos, SubscriberQos},
    return_type::{DDSError, DDSResult},
    subscription::{
        data_reader_listener::DataReaderListener, subscriber_listener::SubscriberListener,
    },
};
use rust_rtps::{
    structure::Group,
    types::{
        constants::{ENTITY_KIND_BUILT_IN_READER_GROUP, ENTITY_KIND_USER_DEFINED_READER_GROUP},
        EntityId, GuidPrefix, GUID,
    },
};

use crate::utils::maybe_valid::{MaybeValid, MaybeValidList, MaybeValidRef};

use super::{
    rtps_datareader_inner::{
        RtpsAnyDataReaderInner, RtpsAnyDataReaderInnerRef, RtpsDataReaderInner,
    },
    rtps_topic_inner::RtpsTopicInnerRef,
};

enum EntityType {
    BuiltIn,
    UserDefined,
}
pub struct RtpsSubscriberInner {
    group: Group,
    entity_type: EntityType,
    reader_list: MaybeValidList<Box<dyn RtpsAnyDataReaderInner>>,
    reader_count: atomic::AtomicU8,
    default_datareader_qos: Mutex<DataReaderQos>,
    qos: Mutex<SubscriberQos>,
    listener: Option<Box<dyn SubscriberListener>>,
    status_mask: StatusMask,
}

impl RtpsSubscriberInner {
    pub fn new_builtin(
        guid_prefix: GuidPrefix,
        entity_key: [u8; 3],
        qos: SubscriberQos,
        listener: Option<Box<dyn SubscriberListener>>,
        status_mask: StatusMask,
    ) -> Self {
        Self::new(
            guid_prefix,
            entity_key,
            qos,
            listener,
            status_mask,
            EntityType::BuiltIn,
        )
    }

    pub fn new_user_defined(
        guid_prefix: GuidPrefix,
        entity_key: [u8; 3],
        qos: SubscriberQos,
        listener: Option<Box<dyn SubscriberListener>>,
        status_mask: StatusMask,
    ) -> Self {
        Self::new(
            guid_prefix,
            entity_key,
            qos,
            listener,
            status_mask,
            EntityType::UserDefined,
        )
    }

    fn new(
        guid_prefix: GuidPrefix,
        entity_key: [u8; 3],
        qos: SubscriberQos,
        listener: Option<Box<dyn SubscriberListener>>,
        status_mask: StatusMask,
        entity_type: EntityType,
    ) -> Self {
        let entity_id = match entity_type {
            EntityType::BuiltIn => EntityId::new(entity_key, ENTITY_KIND_BUILT_IN_READER_GROUP),
            EntityType::UserDefined => {
                EntityId::new(entity_key, ENTITY_KIND_USER_DEFINED_READER_GROUP)
            }
        };
        let guid = GUID::new(guid_prefix, entity_id);

        Self {
            group: Group::new(guid),
            entity_type,
            reader_list: Default::default(),
            reader_count: atomic::AtomicU8::new(0),
            default_datareader_qos: Mutex::new(DataReaderQos::default()),
            qos: Mutex::new(qos),
            listener,
            status_mask,
        }
    }
}

pub type RtpsSubscriberInnerRef<'a> = MaybeValidRef<'a, Box<RtpsSubscriberInner>>;

impl<'a> RtpsSubscriberInnerRef<'a> {
    pub fn get(&self) -> DDSResult<&Box<RtpsSubscriberInner>> {
        MaybeValid::get(self).ok_or(DDSError::AlreadyDeleted)
    }

    pub fn delete(&self) -> DDSResult<()> {
        if self.get()?.reader_list.is_empty() {
            MaybeValid::delete(self);
            Ok(())
        } else {
            Err(DDSError::PreconditionNotMet(
                "Subscriber still contains data readers",
            ))
        }
    }

    pub fn create_datareader<T: DDSType>(
        &self,
        a_topic: &RtpsTopicInnerRef,
        qos: Option<DataReaderQos>,
        a_listener: Option<Box<dyn DataReaderListener<DataType=T>>>,
        status_mask: StatusMask,
    ) -> Option<RtpsAnyDataReaderInnerRef> {
        let entity_key = [
            0,
            self.get()
                .ok()?
                .reader_count
                .fetch_add(1, atomic::Ordering::Relaxed),
            0,
        ];
        self.create_stateful_datareader(entity_key, a_topic, qos, a_listener, status_mask)
    }

    pub fn create_stateful_datareader<T: DDSType>(
        &self,
        entity_key: [u8; 3],
        a_topic: &RtpsTopicInnerRef,
        qos: Option<DataReaderQos>,
        a_listener: Option<Box<dyn DataReaderListener<DataType=T>>>,
        status_mask: StatusMask,
    ) -> Option<RtpsAnyDataReaderInnerRef> {
        let this = self.get().ok()?;
        let qos = qos.unwrap_or(self.get_default_datareader_qos().ok()?);
        let guid_prefix = this.group.entity.guid.prefix();
        let reader: RtpsDataReaderInner<T> = match this.entity_type {
            EntityType::UserDefined => RtpsDataReaderInner::new_user_defined_stateful(
                guid_prefix,
                entity_key,
                a_topic,
                qos,
                a_listener,
                status_mask,
            ),
            EntityType::BuiltIn => RtpsDataReaderInner::new_builtin_stateful(
                guid_prefix,
                entity_key,
                a_topic,
                qos,
                a_listener,
                status_mask,
            ),
        };
        this.reader_list.add(Box::new(reader))
    }

    pub fn create_stateless_datareader<T: DDSType>(
        &self,
        entity_key: [u8; 3],
        a_topic: &RtpsTopicInnerRef,
        qos: Option<DataReaderQos>,
        a_listener: Option<Box<dyn DataReaderListener<DataType=T>>>,
        status_mask: StatusMask,
    ) -> Option<RtpsAnyDataReaderInnerRef> {
        let this = self.get().ok()?;
        let qos = qos.unwrap_or(self.get_default_datareader_qos().ok()?);
        let guid_prefix = this.group.entity.guid.prefix();
        let reader: RtpsDataReaderInner<T> = match this.entity_type {
            EntityType::UserDefined => RtpsDataReaderInner::new_user_defined_stateless(
                guid_prefix,
                entity_key,
                a_topic,
                qos,
                a_listener,
                status_mask,
            ),
            EntityType::BuiltIn => RtpsDataReaderInner::new_builtin_stateless(
                guid_prefix,
                entity_key,
                a_topic,
                qos,
                a_listener,
                status_mask,
            ),
        };
        this.reader_list.add(Box::new(reader))
    }

    pub fn get_qos(&self) -> DDSResult<SubscriberQos> {
        Ok(self.get()?.qos.lock().unwrap().clone())
    }

    pub fn get_default_datareader_qos(&self) -> DDSResult<DataReaderQos> {
        Ok(self.get()?.default_datareader_qos.lock().unwrap().clone())
    }

    pub fn set_default_datareader_qos(&self, qos: Option<DataReaderQos>) -> DDSResult<()> {
        let datareader_qos = qos.unwrap_or_default();
        datareader_qos.is_consistent()?;
        *self.get()?.default_datareader_qos.lock().unwrap() = datareader_qos;
        Ok(())
    }
}

// pub fn create_builtin_datareader<T: DDSType>(
//     &self,
//     a_topic: Arc<dyn RtpsAnyTopicInner>,
//     qos: Option<DataReaderQos>,
//     // _a_listener: impl DataReaderListener<T>,
//     // _mask: StatusMask
// ) -> Option<RtpsAnyDataReaderRef> {
//     self.create_datareader::<T>(a_topic, qos, EntityType::BuiltIn)
// }

// pub fn create_user_defined_datareader<T: DDSType>(
//     &self,
//     a_topic: Arc<dyn RtpsAnyTopicInner>,
//     qos: Option<DataReaderQos>,
//     // _a_listener: impl DataReaderListener<T>,
//     // _mask: StatusMask
// ) -> Option<RtpsAnyDataReaderRef> {
//     self.create_datareader::<T>(a_topic, qos, EntityType::BuiltIn)
// }

// fn create_datareader<T: DDSType>(
//     &self,
//     a_topic: Arc<dyn RtpsAnyTopicInner>,
//     qos: Option<DataReaderQos>,
//     entity_type: EntityType,
//     // _a_listener: impl DataReaderListener<T>,
//     // _mask: StatusMask
// ) -> Option<RtpsAnyDataReaderRef> {
//     let guid_prefix = self.group.entity.guid.prefix();
//     let entity_key = [
//         0,
//         self.reader_count.fetch_add(1, atomic::Ordering::Relaxed),
//         0,
//     ];
//     let entity_kind = match (a_topic.topic_kind(), entity_type) {
//         (TopicKind::WithKey, EntityType::UserDefined) => {
//             ENTITY_KIND_USER_DEFINED_READER_WITH_KEY
//         }
//         (TopicKind::NoKey, EntityType::UserDefined) => ENTITY_KIND_USER_DEFINED_READER_NO_KEY,
//         (TopicKind::WithKey, EntityType::BuiltIn) => ENTITY_KIND_BUILT_IN_READER_WITH_KEY,
//         (TopicKind::NoKey, EntityType::BuiltIn) => ENTITY_KIND_BUILT_IN_READER_NO_KEY,
//     };
//     let entity_id = EntityId::new(entity_key, entity_kind);
//     let new_reader_guid = GUID::new(guid_prefix, entity_id);
//     let new_reader_qos = qos.unwrap_or(self.get_default_datareader_qos());
//     let new_reader: Box<RtpsDataReaderInner<T>> = Box::new(RtpsDataReaderInner::new(
//         new_reader_guid,
//         a_topic,
//         new_reader_qos,
//         None,
//         0,
//     ));
//     self.reader_list.add(new_reader)
// }
