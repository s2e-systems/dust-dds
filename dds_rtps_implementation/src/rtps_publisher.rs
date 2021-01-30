use std::sync::{atomic, Arc, Mutex};

use rust_dds_api::{
    domain::domain_participant::{DomainParticipant, DomainParticipantChild, TopicGAT},
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::{DataWriterQos, PublisherQos, TopicQos},
        status::StatusMask,
    },
    publication::{
        data_writer::DataWriter,
        data_writer_listener::DataWriterListener,
        publisher::{DataWriterGAT, Publisher},
        publisher_listener::PublisherListener,
    },
    subscription::subscriber::Subscriber,
    topic::topic::Topic,
};

use rust_dds_types::{DDSType, Duration, InstanceHandle, ReturnCode, ReturnCodes};
use rust_rtps::{
    structure::Group,
    types::{
        constants::{ENTITY_KIND_BUILT_IN_WRITER_GROUP, ENTITY_KIND_USER_DEFINED_WRITER_GROUP},
        EntityId, EntityKey, GuidPrefix, GUID,
    },
};

use crate::{
    rtps_topic::RtpsTopic,
    utils::maybe_valid::{MaybeValid, MaybeValidList, MaybeValidRef},
};

use super::{
    rtps_datawriter::{AnyRtpsWriter, RtpsDataWriterInner, RtpsDataWriter},
    rtps_participant::RtpsParticipant,
};

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

pub struct RtpsPublisher<'a> {
    parent_participant: &'a RtpsParticipant,
    publisher_ref: RtpsPublisherRef<'a>,
}

impl<'a> RtpsPublisher<'a> {
    pub(crate) fn new(
        parent_participant: &'a RtpsParticipant,
        publisher_ref: RtpsPublisherRef<'a>,
    ) -> Self {
        Self {
            parent_participant,
            publisher_ref,
        }
    }

    pub(crate) fn publisher_ref(&self) -> &RtpsPublisherRef<'a> {
        &self.publisher_ref
    }
}

impl<'a, T: DDSType> TopicGAT<'a, T> for RtpsPublisher<'a> {
    type TopicType = RtpsTopic<'a, T>;
}

impl<'a, T: DDSType> DataWriterGAT<'a, T> for RtpsPublisher<'a> {
    type DataWriterType = RtpsDataWriter<'a, T>;
}

impl<'a> DomainParticipantChild<'a> for RtpsPublisher<'a> {
    type DomainParticipantType = RtpsParticipant;
}

impl<'a> Publisher<'a> for RtpsPublisher<'a> {
    fn create_datawriter<T: DDSType>(
        &'a self,
        _a_topic: &'a <Self as TopicGAT<'a, T>>::TopicType,
        _qos: Option<DataWriterQos>,
        _a_listener: Option<Box<dyn DataWriterListener<T>>>,
        _mask: StatusMask,
    ) -> Option<<Self as DataWriterGAT<'a, T>>::DataWriterType> {
        todo!()
    }

    fn delete_datawriter<T: DDSType>(
        &'a self,
        _a_datawriter: &'a <Self as DataWriterGAT<'a, T>>::DataWriterType,
    ) -> ReturnCode<()> {
        todo!()
    }

    fn lookup_datawriter<T: DDSType>(&self, _topic_name: &str) -> Option<Box<dyn DataWriter<T>>> {
        todo!()
    }

    fn suspend_publications(&self) -> ReturnCode<()> {
        todo!()
    }

    fn resume_publications(&self) -> ReturnCode<()> {
        todo!()
    }

    fn begin_coherent_changes(&self) -> ReturnCode<()> {
        todo!()
    }

    fn end_coherent_changes(&self) -> ReturnCode<()> {
        todo!()
    }

    fn wait_for_acknowledgments(&self, _max_wait: Duration) -> ReturnCode<()> {
        todo!()
    }

    fn get_participant(&self) -> &<Self as DomainParticipantChild<'a>>::DomainParticipantType{
        &self.parent_participant
    }

    fn delete_contained_entities(&self) -> ReturnCode<()> {
        todo!()
    }

    fn set_default_datawriter_qos(&self, _qos: Option<DataWriterQos>) -> ReturnCode<()> {
        todo!()
    }

    fn get_default_datawriter_qos(&self) -> ReturnCode<DataWriterQos> {
        todo!()
    }

    fn copy_from_topic_qos(
        &self,
        _a_datawriter_qos: &mut DataWriterQos,
        _a_topic_qos: &TopicQos,
    ) -> ReturnCode<()> {
        todo!()
    }
}

impl<'a> Entity for RtpsPublisher<'a> {
    type Qos = PublisherQos;
    type Listener = Box<dyn PublisherListener>;

    fn set_qos(&self, _qos: Option<Self::Qos>) -> ReturnCode<()> {
        todo!()
    }

    fn get_qos(&self) -> ReturnCode<Self::Qos> {
        self.publisher_ref.get_qos()
    }

    fn set_listener(&self, _a_listener: Self::Listener, _mask: StatusMask) -> ReturnCode<()> {
        todo!()
    }

    fn get_listener(&self) -> &Self::Listener {
        todo!()
    }

    fn get_statuscondition(&self) -> StatusCondition {
        todo!()
    }

    fn get_status_changes(&self) -> StatusMask {
        todo!()
    }

    fn enable(&self) -> ReturnCode<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> ReturnCode<InstanceHandle> {
        todo!()
    }
}

// impl<'a> RtpsPublisherNode<'a> {
// pub fn get(&self) -> ReturnCode<&RtpsPublisher> {
//     Ok(MaybeValid::get(&self.maybe_valid_ref)
//         .ok_or(ReturnCodes::AlreadyDeleted)?
//         .as_ref())
// }

// pub fn create_datawriter<T: DDSType>(
//     &self,
//     a_topic: &RtpsAnyTopicRef,
//     qos: Option<DataWriterQos>,
//     // _a_listener: impl DataWriterListener<T>,
//     // _mask: StatusMask
// ) -> Option<RtpsAnyDataWriterRef> {
//     let this = self.get().ok()?;
//     let qos = qos.unwrap_or(self.get_default_datawriter_qos().ok()?);
//     let guid_prefix = this.group.entity.guid.prefix();
//     let entity_key = [
//         0,
//         this.writer_count.fetch_add(1, atomic::Ordering::Relaxed),
//         0,
//     ];
//     this.create_stateful_datawriter::<T>(guid_prefix, entity_key, a_topic, qos)
// }

// pub fn lookup_datawriter<T: DDSType>(&self, topic_name: &str) -> Option<RtpsAnyDataWriterRef> {
//     self.get().ok()?.writer_list.into_iter().find(|writer| {
//         if let Some(any_writer) = writer.get_as::<T>().ok() {
//             let topic_mutex_guard = any_writer.topic.lock().unwrap();
//             match &*topic_mutex_guard {
//                 Some(any_topic) => any_topic.topic_name() == topic_name,
//                 _ => false,
//             }
//         } else {
//             false
//         }
//     })
// }

// pub fn get_default_datawriter_qos(&self) -> ReturnCode<DataWriterQos> {
//     Ok(self.get()?.default_datawriter_qos.lock().unwrap().clone())
// }

// pub fn set_default_datawriter_qos(&self, qos: Option<DataWriterQos>) -> ReturnCode<()> {
//     let datawriter_qos = qos.unwrap_or_default();
//     datawriter_qos.is_consistent()?;
//     *self.get()?.default_datawriter_qos.lock().unwrap() = datawriter_qos;
//     Ok(())
// }

// pub fn delete(&self) {
//     MaybeValid::delete(&self.maybe_valid_ref)
// }

// pub fn create_stateful_datawriter<T: DDSType>(
//     &self,
//     a_topic: &RtpsAnyTopicRef,
//     qos: Option<DataWriterQos>,
//     // _a_listener: impl DataWriterListener<T>,
//     // _mask: StatusMask
// ) -> Option<RtpsAnyDataWriterRef> {
//     self.create_datawriter::<T>(a_topic, qos, Statefulness::Stateful)
// }

// pub fn create_stateless_datawriter<T: DDSType>(
//     &self,
//     a_topic: &RtpsAnyTopicRef,
//     qos: Option<DataWriterQos>,
//     // _a_listener: impl DataWriterListener<T>,
//     // _mask: StatusMask
// ) -> Option<RtpsAnyDataWriterRef> {
//     self.create_datawriter::<T>(a_topic, qos, Statefulness::Stateless)
// }
// }
