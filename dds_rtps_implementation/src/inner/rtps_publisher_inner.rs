use std::sync::{atomic, Mutex};

use rust_dds_api::{
    infrastructure::{
        qos::{DataWriterQos, PublisherQos},
        status::StatusMask,
    },
    publication::publisher_listener::PublisherListener,
};
use rust_dds_types::{ReturnCode, ReturnCodes};
use rust_rtps::{
    structure::Group,
    types::{
        constants::{ENTITY_KIND_BUILT_IN_WRITER_GROUP, ENTITY_KIND_USER_DEFINED_WRITER_GROUP},
        EntityId, EntityKey, GuidPrefix, GUID,
    },
};

use crate::utils::maybe_valid::{MaybeValid, MaybeValidList, MaybeValidRef};

use super::rtps_datawriter_inner::AnyRtpsWriter;

enum Statefulness {
    Stateless,
    Stateful,
}
enum EntityType {
    BuiltIn,
    UserDefined,
}
pub struct RtpsPublisherInner {
    pub group: Group,
    entity_type: EntityType,
    pub writer_list: MaybeValidList<Box<dyn AnyRtpsWriter>>,
    pub writer_count: atomic::AtomicU8,
    pub default_datawriter_qos: Mutex<DataWriterQos>,
    pub qos: PublisherQos,
    pub listener: Option<Box<dyn PublisherListener>>,
    pub status_mask: StatusMask,
}

impl RtpsPublisherInner {
    pub fn new_builtin(
        guid_prefix: GuidPrefix,
        entity_key: EntityKey,
        qos: PublisherQos,
        listener: Option<Box<dyn PublisherListener>>,
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
        entity_key: EntityKey,
        qos: PublisherQos,
        listener: Option<Box<dyn PublisherListener>>,
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
        entity_key: EntityKey,
        qos: PublisherQos,
        listener: Option<Box<dyn PublisherListener>>,
        status_mask: StatusMask,
        entity_type: EntityType,
    ) -> Self {
        let entity_id = match entity_type {
            EntityType::BuiltIn => EntityId::new(entity_key, ENTITY_KIND_BUILT_IN_WRITER_GROUP),
            EntityType::UserDefined => {
                EntityId::new(entity_key, ENTITY_KIND_USER_DEFINED_WRITER_GROUP)
            }
        };
        let guid = GUID::new(guid_prefix, entity_id);

        Self {
            group: Group::new(guid),
            entity_type,
            writer_list: Default::default(),
            writer_count: atomic::AtomicU8::new(0),
            default_datawriter_qos: Mutex::new(DataWriterQos::default()),
            qos,
            listener,
            status_mask,
        }
    }

    // pub fn create_stateful_datawriter<T: DDSType>(
    //     &self,
    //     guid_prefix: GuidPrefix,
    //     entity_key: EntityKey,
    //     a_topic: &RtpsAnyTopicRef,
    //     qos: DataWriterQos,
    // ) -> Option<RtpsAnyDataWriterRef> {
    //     let writer: RtpsDataWriter<T> = match self.entity_type {
    //         EntityType::UserDefined => RtpsDataWriter::new_user_defined_stateful(
    //             guid_prefix,
    //             entity_key,
    //             a_topic,
    //             qos,
    //             None,
    //             0,
    //         ),
    //         EntityType::BuiltIn => {
    //             RtpsDataWriter::new_builtin_stateful(guid_prefix, entity_key, a_topic, qos, None, 0)
    //         }
    //     };
    //     self.writer_list.add(Box::new(writer))
    // }

    // pub fn create_stateless_datawriter<T: DDSType>(
    //     &self,
    //     guid_prefix: GuidPrefix,
    //     entity_key: EntityKey,
    //     a_topic: &RtpsAnyTopicRef,
    //     qos: DataWriterQos,
    // ) -> Option<RtpsAnyDataWriterRef> {
    //     let writer: RtpsDataWriter<T> = match self.entity_type {
    //         EntityType::UserDefined => RtpsDataWriter::new_user_defined_stateless(
    //             guid_prefix,
    //             entity_key,
    //             a_topic,
    //             qos,
    //             None,
    //             0,
    //         ),
    //         EntityType::BuiltIn => {
    //             RtpsDataWriter::new_builtin_stateless(guid_prefix, entity_key, a_topic, qos, None, 0)
    //         }
    //     };
    //     self.writer_list.add(Box::new(writer))
    // }
}

pub type RtpsPublisherRef<'a> = MaybeValidRef<'a, Box<RtpsPublisherInner>>;

impl<'a> RtpsPublisherRef<'a> {
    pub fn get(&self) -> ReturnCode<&Box<RtpsPublisherInner>> {
        MaybeValid::get(self).ok_or(ReturnCodes::AlreadyDeleted)
    }

    pub fn delete(&self) {
        MaybeValid::delete(self)
    }

    pub fn get_qos(&self) -> ReturnCode<PublisherQos> {
        Ok(self.get()?.qos.clone())
    }
}
