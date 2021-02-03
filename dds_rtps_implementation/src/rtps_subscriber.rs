use std::{marker::PhantomData, ops::Deref, sync::{atomic, Arc, Mutex}};

use crate::{
    inner::rtps_subscriber_inner::RtpsSubscriberInnerRef,
    rtps_datareader::RtpsDataReader,
    rtps_topic::RtpsTopic,
    utils::maybe_valid::{MaybeValid, MaybeValidList, MaybeValidRef},
};
use rust_dds_api::{
    domain::domain_participant::{DomainParticipant, DomainParticipantChild, TopicGAT},
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::{DataReaderQos, SubscriberQos, TopicQos},
        status::{InstanceStateKind, SampleLostStatus, SampleStateKind, StatusMask, ViewStateKind},
    },
    publication::publisher::Publisher,
    subscription::{
        data_reader::{AnyDataReader, DataReader},
        data_reader_listener::DataReaderListener,
        subscriber::{DataReaderGAT, Subscriber},
        subscriber_listener::SubscriberListener,
    },
    topic::topic::Topic,
};
use rust_dds_types::{DDSType, InstanceHandle, ReturnCode, ReturnCodes, TopicKind};
use rust_rtps::{
    structure::Group,
    types::{
        constants::{
            ENTITY_KIND_BUILT_IN_READER_WITH_KEY, ENTITY_KIND_USER_DEFINED_READER_NO_KEY,
            ENTITY_KIND_USER_DEFINED_READER_WITH_KEY,
        },
        EntityId, GUID,
    },
};

use super::rtps_domain_participant::RtpsDomainParticipant;

pub struct RtpsSubscriber<'a> {
    pub(crate) parent_participant: &'a RtpsDomainParticipant,
    pub(crate) subscriber_ref: RtpsSubscriberInnerRef<'a>,
}

impl<'a> RtpsSubscriber<'a> {
    pub(crate) fn new(
        parent_participant: &'a RtpsDomainParticipant,
        subscriber_ref: RtpsSubscriberInnerRef<'a>,
    ) -> Self {
        Self {
            parent_participant,
            subscriber_ref,
        }
    }

    pub(crate) fn subscriber_ref(&self) -> &RtpsSubscriberInnerRef<'a> {
        &self.subscriber_ref
    }
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
        a_listener: Option<Box<dyn DataReaderListener<T>>>,
        mask: StatusMask,
    ) -> Option<<Self as DataReaderGAT<'a, T>>::DataReaderType> {
        let data_reader_ref =
            self.subscriber_ref
                .create_datareader(&a_topic.topic_ref, qos, a_listener, mask)?;

        Some(RtpsDataReader{parent_subscriber:self, data_reader_ref, phantom_data: PhantomData})
    }

    fn delete_datareader<T: DDSType>(
        &'a self,
        _a_datareader: &'a <Self as DataReaderGAT<'a, T>>::DataReaderType,
    ) -> ReturnCode<()> {
        todo!()
    }

    fn lookup_datareader<T: DDSType>(
        &self,
        _topic: &<Self as TopicGAT<'a, T>>::TopicType,
    ) -> Option<<Self as DataReaderGAT<'a, T>>::DataReaderType> {
        todo!()
    }

    fn begin_access(&self) -> ReturnCode<()> {
        todo!()
    }

    fn end_access(&self) -> ReturnCode<()> {
        todo!()
    }

    fn notify_datareaders(&self) -> ReturnCode<()> {
        todo!()
    }

    fn get_participant(&self) -> &<Self as DomainParticipantChild<'a>>::DomainParticipantType {
        &self.parent_participant
    }

    fn get_sample_lost_status(&self, _status: &mut SampleLostStatus) -> ReturnCode<()> {
        todo!()
    }

    fn delete_contained_entities(&self) -> ReturnCode<()> {
        todo!()
    }

    fn set_default_datareader_qos(&self, _qos: Option<DataReaderQos>) -> ReturnCode<()> {
        todo!()
    }

    fn get_default_datareader_qos(&self) -> ReturnCode<DataReaderQos> {
        todo!()
    }

    fn copy_from_topic_qos(
        &self,
        _a_datareader_qos: &mut DataReaderQos,
        _a_topic_qos: &TopicQos,
    ) -> ReturnCode<()> {
        todo!()
    }

    fn get_datareaders(
        &self,
        _readers: &mut [&mut dyn AnyDataReader],
        _sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
    ) -> ReturnCode<()> {
        todo!()
    }
}

impl<'a> Entity for RtpsSubscriber<'a> {
    type Qos = SubscriberQos;
    type Listener = Box<dyn SubscriberListener>;

    fn set_qos(&self, _qos: Option<Self::Qos>) -> ReturnCode<()> {
        todo!()
    }

    fn get_qos(&self) -> ReturnCode<Self::Qos> {
        self.subscriber_ref.get_qos()
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
