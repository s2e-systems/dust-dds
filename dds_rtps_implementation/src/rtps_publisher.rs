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
    inner::{
        rtps_datawriter_inner::RtpsAnyDataWriterInnerRef,
        rtps_publisher_inner::RtpsPublisherInnerRef,
    },
    rtps_topic::RtpsTopic,
    utils::maybe_valid::{MaybeValid, MaybeValidList, MaybeValidRef},
};

use super::{rtps_datawriter::RtpsDataWriter, rtps_domain_participant::RtpsDomainParticipant};

pub struct RtpsPublisher<'a> {
    pub(crate) parent_participant: &'a RtpsDomainParticipant,
    pub(crate) publisher_ref: RtpsPublisherInnerRef<'a>,
}

impl<'a> RtpsPublisher<'a> {
    pub(crate) fn new(
        parent_participant: &'a RtpsDomainParticipant,
        publisher_ref: RtpsPublisherInnerRef<'a>,
    ) -> Self {
        Self {
            parent_participant,
            publisher_ref,
        }
    }

    pub(crate) fn publisher_ref(&self) -> &RtpsPublisherInnerRef<'a> {
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
    type DomainParticipantType = RtpsDomainParticipant;
}

impl<'a> Publisher<'a> for RtpsPublisher<'a> {
    fn create_datawriter<T: DDSType>(
        &'a self,
        a_topic: &'a <Self as TopicGAT<'a, T>>::TopicType,
        qos: Option<DataWriterQos>,
        a_listener: Option<Box<dyn DataWriterListener<T>>>,
        mask: StatusMask,
    ) -> Option<<Self as DataWriterGAT<'a, T>>::DataWriterType> {
        let data_writer_ref =
            self.publisher_ref
                .create_datawriter(&a_topic.topic_ref, qos, a_listener, mask)?;

        Some(RtpsDataWriter::new(self, data_writer_ref))
    }

    fn delete_datawriter<T: DDSType>(
        &'a self,
        a_datawriter: &'a <Self as DataWriterGAT<'a, T>>::DataWriterType,
    ) -> ReturnCode<()> {
        a_datawriter.data_writer_ref.delete()
    }

    fn lookup_datawriter<T: DDSType>(
        &self,
        _topic_name: &str,
    ) -> Option<<Self as DataWriterGAT<'a, T>>::DataWriterType> {
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

    fn get_participant(&self) -> &<Self as DomainParticipantChild<'a>>::DomainParticipantType {
        &self.parent_participant
    }

    fn delete_contained_entities(&self) -> ReturnCode<()> {
        todo!()
    }

    fn set_default_datawriter_qos(&self, qos: Option<DataWriterQos>) -> ReturnCode<()> {
        self.publisher_ref.set_default_datawriter_qos(qos)
    }

    fn get_default_datawriter_qos(&self) -> ReturnCode<DataWriterQos> {
        self.publisher_ref.get_default_datawriter_qos()
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

    fn set_qos(&self, qos: Option<Self::Qos>) -> ReturnCode<()> {
        self.publisher_ref.set_qos(qos)
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
        self.publisher_ref.get_instance_handle()
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
