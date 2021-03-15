use std::ops::Deref;

use rust_dds_api::{
    dcps_psm::{Duration, InstanceHandle, StatusMask},
    dds_type::DDSType,
    domain::domain_participant::{DomainParticipantChild, TopicGAT},
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::{DataWriterQos, PublisherQos, TopicQos},
    },
    publication::{
        data_writer_listener::DataWriterListener, publisher::DataWriterGAT,
        publisher_listener::PublisherListener,
    },
    return_type::{DDSError, DDSResult},
};

use crate::{
    domain_participant::{DomainParticipant, Publisher, Topic},
    utils::node::Node,
};

pub struct DataWriter<'a, T: DDSType>(<Self as Deref>::Target);

impl<'a, T: DDSType> Deref for DataWriter<'a, T> {
    type Target = Node<(&'a Publisher<'a>, &'a Topic<'a, T>), ()>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T: DDSType> TopicGAT<'a, T> for Publisher<'a> {
    type TopicType = Topic<'a, T>;
}

impl<'a, T: DDSType> DataWriterGAT<'a, T> for Publisher<'a> {
    type DataWriterType = DataWriter<'a, T>;
}

impl<'a> DomainParticipantChild<'a> for Publisher<'a> {
    type DomainParticipantType = DomainParticipant;
}

impl<'a> rust_dds_api::publication::publisher::Publisher<'a> for Publisher<'a> {
    fn create_datawriter<T: DDSType>(
        &'a self,
        _a_topic: &'a <Self as TopicGAT<'a, T>>::TopicType,
        _qos: Option<DataWriterQos>,
        _a_listener: Option<Box<dyn DataWriterListener<DataType = T>>>,
        _mask: StatusMask,
    ) -> Option<<Self as DataWriterGAT<'a, T>>::DataWriterType> {
        todo!()
        // let topic = a_topic.impl_ref.upgrade()?;
        // let data_writer_ref = self
        //     .impl_ref
        //     .upgrade()?
        //     .lock()
        //     .unwrap()
        //     .create_datawriter(topic, qos, a_listener, mask)?;

        // Some(RtpsDataWriter(Node {
        //     parent: (self, a_topic),
        //     impl_ref: data_writer_ref,
        // }))
    }

    fn delete_datawriter<T: DDSType>(
        &'a self,
        _a_datawriter: &<Self as DataWriterGAT<'a, T>>::DataWriterType,
    ) -> DDSResult<()> {
        todo!()
        // if std::ptr::eq(a_datawriter.parent.0, self) {
        //     self.impl_ref
        //         .upgrade()
        //         .ok_or(DDSError::AlreadyDeleted)?
        //         .lock()
        //         .unwrap()
        //         .delete_datawriter(&a_datawriter.impl_ref)
        // } else {
        //     Err(DDSError::PreconditionNotMet(
        //         "Publisher can only be deleted from its parent participant",
        //     ))
        // }
    }

    fn lookup_datawriter<T: DDSType>(
        &self,
        _topic: &<Self as TopicGAT<'a, T>>::TopicType,
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
        self.parent
    }

    fn delete_contained_entities(&self) -> DDSResult<()> {
        todo!()
    }

    fn set_default_datawriter_qos(&self, _qos: Option<DataWriterQos>) -> DDSResult<()> {
        // self.publisher_ref.set_default_datawriter_qos(qos)
        todo!()
    }

    fn get_default_datawriter_qos(&self) -> DDSResult<DataWriterQos> {
        // self.publisher_ref.get_default_datawriter_qos()
        todo!()
    }

    fn copy_from_topic_qos(
        &self,
        _a_datawriter_qos: &mut DataWriterQos,
        _a_topic_qos: &TopicQos,
    ) -> DDSResult<()> {
        todo!()
    }
}

impl<'a> Entity for Publisher<'a> {
    type Qos = PublisherQos;
    type Listener = Box<dyn PublisherListener + 'a>;

    fn set_qos(&self, qos: Option<Self::Qos>) -> DDSResult<()> {
        Ok(self
            .impl_ref
            .upgrade()
            .ok_or(DDSError::AlreadyDeleted)?
            .lock()
            .unwrap()
            .set_qos(qos))
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        Ok(self
            .impl_ref
            .upgrade()
            .ok_or(DDSError::AlreadyDeleted)?
            .lock()
            .unwrap()
            .get_qos()
            .clone())
    }

    fn set_listener(
        &self,
        _a_listener: Option<Self::Listener>,
        _mask: StatusMask,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_listener(&self) -> DDSResult<Option<Self::Listener>> {
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
        // self.publisher_ref.get_instance_handle()
        todo!()
    }
}