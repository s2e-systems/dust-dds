use std::marker::PhantomData;

use rust_dds_api::{
    dcps_psm::{Duration, InstanceHandle, StatusMask},
    dds_type::DDSType,
    domain::domain_participant::{DomainParticipantChild, TopicGAT},
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::{DataWriterQos, PublisherQos, TopicQos},
    },
    publication::{
        data_writer_listener::DataWriterListener,
        publisher::{DataWriterGAT, Publisher},
        publisher_listener::PublisherListener,
    },
    return_type::DDSResult,
};

use crate::{inner::rtps_publisher_inner::RtpsPublisherInnerRef, rtps_topic::RtpsTopic};

use super::{rtps_datawriter::RtpsDataWriter, rtps_domain_participant::RtpsDomainParticipant};

pub struct RtpsPublisher<'a> {
    pub(crate) parent_participant: &'a RtpsDomainParticipant,
    pub(crate) publisher_ref: RtpsPublisherInnerRef<'a>,
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

        Some(RtpsDataWriter {
            parent_publisher: self,
            data_writer_ref,
            phantom_data: PhantomData,
        })
    }

    fn delete_datawriter<T: DDSType>(
        &'a self,
        a_datawriter: &'a <Self as DataWriterGAT<'a, T>>::DataWriterType,
    ) -> DDSResult<()> {
        a_datawriter.data_writer_ref.delete()
    }

    fn lookup_datawriter<T: DDSType>(
        &self,
        _topic_name: &str,
    ) -> Option<<Self as DataWriterGAT<'a, T>>::DataWriterType> {
        todo!()
    }

    fn suspend_publications(&self) -> DDSResult<()> {
        todo!()
    }

    fn resume_publications(&self) -> DDSResult<()> {
        todo!()
    }

    fn begin_coherent_changes(&self) -> DDSResult<()> {
        todo!()
    }

    fn end_coherent_changes(&self) -> DDSResult<()> {
        todo!()
    }

    fn wait_for_acknowledgments(&self, _max_wait: Duration) -> DDSResult<()> {
        todo!()
    }

    fn get_participant(&self) -> &<Self as DomainParticipantChild<'a>>::DomainParticipantType {
        &self.parent_participant
    }

    fn delete_contained_entities(&self) -> DDSResult<()> {
        todo!()
    }

    fn set_default_datawriter_qos(&self, qos: Option<DataWriterQos>) -> DDSResult<()> {
        self.publisher_ref.set_default_datawriter_qos(qos)
    }

    fn get_default_datawriter_qos(&self) -> DDSResult<DataWriterQos> {
        self.publisher_ref.get_default_datawriter_qos()
    }

    fn copy_from_topic_qos(
        &self,
        _a_datawriter_qos: &mut DataWriterQos,
        _a_topic_qos: &TopicQos,
    ) -> DDSResult<()> {
        todo!()
    }
}

impl<'a> Entity for RtpsPublisher<'a> {
    type Qos = PublisherQos;
    type Listener = Box<dyn PublisherListener + 'a>;

    fn set_qos(&self, qos: Option<Self::Qos>) -> DDSResult<()> {
        self.publisher_ref.set_qos(qos)
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        self.publisher_ref.get_qos()
    }

    fn set_listener(&self, _a_listener: Self::Listener, _mask: StatusMask) -> DDSResult<()> {
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

    fn enable(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        self.publisher_ref.get_instance_handle()
    }
}

// impl<'a> RtpsPublisherNode<'a> {
// pub fn get(&self) -> DDSResult<&RtpsPublisher> {
//     Ok(MaybeValid::get(&self.maybe_valid_ref)
//         .ok_or(DDSError::AlreadyDeleted)?
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

// pub fn get_default_datawriter_qos(&self) -> DDSResult<DataWriterQos> {
//     Ok(self.get()?.default_datawriter_qos.lock().unwrap().clone())
// }

// pub fn set_default_datawriter_qos(&self, qos: Option<DataWriterQos>) -> DDSResult<()> {
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
