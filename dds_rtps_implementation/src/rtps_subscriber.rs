use std::marker::PhantomData;

use crate::{
    inner::rtps_subscriber_inner::RtpsSubscriberInnerRef, rtps_datareader::RtpsDataReader,
    rtps_topic::RtpsTopic,
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
    return_type::DDSResult,
    subscription::{
        data_reader::AnyDataReader,
        data_reader_listener::DataReaderListener,
        subscriber::{DataReaderGAT, Subscriber},
        subscriber_listener::SubscriberListener,
    },
};

use super::rtps_domain_participant::RtpsDomainParticipant;

pub struct RtpsSubscriber<'a> {
    pub(crate) parent_participant: &'a RtpsDomainParticipant,
    pub(crate) subscriber_ref: RtpsSubscriberInnerRef<'a>,
}

impl<'a, T: DDSType> TopicGAT<'a, T> for RtpsSubscriber<'a> {
    type TopicType = RtpsTopic<'a, T>;
}

impl<'a, T: DDSType> DataReaderGAT<'a, T> for RtpsSubscriber<'a> {
    type DataReaderType = RtpsDataReader<'a, T>;
}

impl<'a> DomainParticipantChild<'a> for RtpsSubscriber<'a> {
    type DomainParticipantType = RtpsDomainParticipant;
}

impl<'a> Subscriber<'a> for RtpsSubscriber<'a> {
    fn create_datareader<T: DDSType>(
        &'a self,
        a_topic: &'a <Self as TopicGAT<'a, T>>::TopicType,
        qos: Option<DataReaderQos>,
        a_listener: Option<Box<dyn DataReaderListener<DataType=T>>>,
        mask: StatusMask,
    ) -> Option<<Self as DataReaderGAT<'a, T>>::DataReaderType> {
        let data_reader_ref =
            self.subscriber_ref
                .create_datareader(&a_topic.topic_ref, qos, a_listener, mask)?;

        Some(RtpsDataReader {
            parent_subscriber: self,
            data_reader_ref,
            phantom_data: PhantomData,
        })
    }

    fn delete_datareader<T: DDSType>(
        &'a self,
        a_datareader: &'a <Self as DataReaderGAT<'a, T>>::DataReaderType,
    ) -> DDSResult<()> {
        a_datareader.data_reader_ref.delete()
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
        &self.parent_participant
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

impl<'a> Entity for RtpsSubscriber<'a> {
    type Qos = SubscriberQos;
    type Listener = Box<dyn SubscriberListener>;

    fn set_qos(&self, _qos: Option<Self::Qos>) -> DDSResult<()> {
        todo!()
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        self.subscriber_ref.get_qos()
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
        todo!()
    }
}
