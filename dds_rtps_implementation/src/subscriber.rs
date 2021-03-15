use std::ops::Deref;

use crate::{
    impls::data_reader_impl::DataReaderImpl,
    domain_participant::{DomainParticipant, Subscriber, Topic},
    utils::node::Node,
};
use rust_dds_api::{
    dcps_psm::{
        InstanceHandle, InstanceStateKind, SampleLostStatus, SampleStateKind, StatusMask,
        ViewStateKind,
    },
    dds_type::DDSType,
    domain::domain_participant::{DomainParticipantChild, TopicGAT},
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::{DataReaderQos, SubscriberQos, TopicQos},
    },
    return_type::{DDSError, DDSResult},
    subscription::{
        data_reader::AnyDataReader, data_reader_listener::DataReaderListener,
        subscriber::DataReaderGAT, subscriber_listener::SubscriberListener,
    },
};

pub struct DataReader<'a, T: DDSType>(<Self as Deref>::Target);

impl<'a, T: DDSType> Deref for DataReader<'a, T> {
    type Target = Node<(&'a Subscriber<'a>, &'a Topic<'a, T>), DataReaderImpl>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T: DDSType> TopicGAT<'a, T> for Subscriber<'a> {
    type TopicType = Topic<'a, T>;
}

impl<'a, T: DDSType> DataReaderGAT<'a, T> for Subscriber<'a> {
    type DataReaderType = DataReader<'a, T>;
}

impl<'a> DomainParticipantChild<'a> for Subscriber<'a> {
    type DomainParticipantType = DomainParticipant;
}

impl<'a> rust_dds_api::subscription::subscriber::Subscriber<'a> for Subscriber<'a> {
    fn create_datareader<T: DDSType>(
        &'a self,
        a_topic: &'a <Self as TopicGAT<'a, T>>::TopicType,
        qos: Option<DataReaderQos>,
        a_listener: Option<Box<dyn DataReaderListener<DataType = T>>>,
        mask: StatusMask,
    ) -> Option<<Self as DataReaderGAT<'a, T>>::DataReaderType> {
        let topic = a_topic.impl_ref.upgrade()?;
        let data_reader_ref = self
            .impl_ref
            .upgrade()?
            .lock()
            .unwrap()
            .create_datareader(topic, qos, a_listener, mask)?;

        Some(DataReader(Node {
            parent: (self, a_topic),
            impl_ref: data_reader_ref,
        }))
    }

    fn delete_datareader<T: DDSType>(
        &'a self,
        a_datareader: &<Self as DataReaderGAT<'a, T>>::DataReaderType,
    ) -> DDSResult<()> {
        // todo!()
        if std::ptr::eq(a_datareader.parent.0, self) {
            self.impl_ref
                .upgrade()
                .ok_or(DDSError::AlreadyDeleted)?
                .lock()
                .unwrap()
                .delete_datareader(&a_datareader.impl_ref)
        } else {
            Err(DDSError::PreconditionNotMet(
                "Publisher can only be deleted from its parent participant",
            ))
        }
    }

    fn lookup_datareader<T: DDSType>(
        &self,
        _topic: &<Self as TopicGAT<'a, T>>::TopicType,
    ) -> Option<<Self as DataReaderGAT<'a, T>>::DataReaderType> {
        todo!()
    }

    fn begin_access(&self) -> DDSResult<()> {
        todo!()
    }

    fn end_access(&self) -> DDSResult<()> {
        todo!()
    }

    fn notify_datareaders(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_participant(&self) -> &<Self as DomainParticipantChild<'a>>::DomainParticipantType {
        self.parent
    }

    fn get_sample_lost_status(&self, _status: &mut SampleLostStatus) -> DDSResult<()> {
        todo!()
    }

    fn delete_contained_entities(&self) -> DDSResult<()> {
        todo!()
    }

    fn set_default_datareader_qos(&self, _qos: Option<DataReaderQos>) -> DDSResult<()> {
        todo!()
    }

    fn get_default_datareader_qos(&self) -> DDSResult<DataReaderQos> {
        todo!()
    }

    fn copy_from_topic_qos(
        &self,
        _a_datareader_qos: &mut DataReaderQos,
        _a_topic_qos: &TopicQos,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_datareaders(
        &self,
        _readers: &mut [&mut dyn AnyDataReader],
        _sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
    ) -> DDSResult<()> {
        todo!()
    }
}

impl<'a> Entity for Subscriber<'a> {
    type Qos = SubscriberQos;
    type Listener = Box<dyn SubscriberListener + 'a>;

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
        todo!()
    }
}