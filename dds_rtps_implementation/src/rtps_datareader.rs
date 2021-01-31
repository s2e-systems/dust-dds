use std::{
    any::Any,
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use rust_dds_api::{
    builtin_topics::PublicationBuiltinTopicData,
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::{DataReaderQos, SubscriberQos},
        qos_policy::ReliabilityQosPolicyKind,
        read_condition::ReadCondition,
        sample_info::SampleInfo,
        status::{
            InstanceStateKind, LivelinessChangedStatus, RequestedDeadlineMissedStatus,
            RequestedIncompatibleQosStatus, SampleLostStatus, SampleRejectedStatus,
            SampleStateKind, StatusMask, SubscriptionMatchedStatus, ViewStateKind,
        },
    },
    subscription::{
        data_reader::{AnyDataReader, DataReader},
        data_reader_listener::DataReaderListener,
        query_condition::QueryCondition,
        subscriber::SubscriberChild,
    },
    topic::topic_description::TopicDescription,
};

use rust_rtps::{
    behavior::{self, StatefulReader},
    types::{ReliabilityKind, GUID},
};

use rust_dds_types::{DDSType, InstanceHandle, ReturnCode, ReturnCodes};

use crate::{
    inner::rtps_datareader_inner::RtpsAnyDataReaderRef,
    rtps_subscriber::RtpsSubscriber,
    utils::{
        as_any::AsAny,
        maybe_valid::{MaybeValid, MaybeValidRef},
    },
};

use super::rtps_topic::AnyRtpsTopic;

pub struct RtpsDataReader<'a, T: DDSType> {
    parent_subscriber: &'a RtpsSubscriber<'a>,
    data_reader_ref: RtpsAnyDataReaderRef<'a>,
    phantom_data: PhantomData<T>,
}

impl<'a, T: DDSType> DataReader<'a, T> for RtpsDataReader<'a, T> {
    fn read(
        &self,
        _data_values: &mut [T],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
    ) -> ReturnCode<()> {
        todo!()
    }

    fn take(
        &self,
        _data_values: &mut [T],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
    ) -> ReturnCode<()> {
        todo!()
    }

    fn read_w_condition(
        &self,
        _data_values: &mut [T],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _a_condition: ReadCondition,
    ) -> ReturnCode<()> {
        todo!()
    }

    fn take_w_condition(
        &self,
        _data_values: &mut [T],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _a_condition: ReadCondition,
    ) -> ReturnCode<()> {
        todo!()
    }

    fn read_next_sample(
        &self,
        _data_value: &mut [T],
        _sample_info: &mut [SampleInfo],
    ) -> ReturnCode<()> {
        todo!()
    }

    fn take_next_sample(
        &self,
        _data_value: &mut [T],
        _sample_info: &mut [SampleInfo],
    ) -> ReturnCode<()> {
        todo!()
    }

    fn read_instance(
        &self,
        _data_values: &mut [T],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _a_handle: InstanceHandle,
        _sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
    ) -> ReturnCode<()> {
        todo!()
    }

    fn take_instance(
        &self,
        _data_values: &mut [T],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _a_handle: InstanceHandle,
        _sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
    ) -> ReturnCode<()> {
        todo!()
    }

    fn read_next_instance(
        &self,
        _data_values: &mut [T],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _previous_handle: InstanceHandle,
        _sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
    ) -> ReturnCode<()> {
        todo!()
    }

    fn take_next_instance(
        &self,
        _data_values: &mut [T],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _previous_handle: InstanceHandle,
        _sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
    ) -> ReturnCode<()> {
        todo!()
    }

    fn read_next_instance_w_condition(
        &self,
        _data_values: &mut [T],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _previous_handle: InstanceHandle,
        _a_condition: ReadCondition,
    ) -> ReturnCode<()> {
        todo!()
    }

    fn take_next_instance_w_condition(
        &self,
        _data_values: &mut [T],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _previous_handle: InstanceHandle,
        _a_condition: ReadCondition,
    ) -> ReturnCode<()> {
        todo!()
    }

    fn return_loan(
        &self,
        _data_values: &mut [T],
        _sample_infos: &mut [SampleInfo],
    ) -> ReturnCode<()> {
        todo!()
    }

    fn get_key_value(&self, _key_holder: &mut T, _handle: InstanceHandle) -> ReturnCode<()> {
        todo!()
    }

    fn lookup_instance(&self, _instance: &T) -> InstanceHandle {
        todo!()
    }

    fn create_readcondition(
        &self,
        _sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
    ) -> ReadCondition {
        todo!()
    }

    fn create_querycondition(
        &self,
        _sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
        _query_expression: String,
        _query_parameters: &[String],
    ) -> QueryCondition {
        todo!()
    }

    fn delete_readcondition(&self, _a_condition: ReadCondition) -> ReturnCode<()> {
        todo!()
    }

    fn get_liveliness_changed_status(
        &self,
        _status: &mut LivelinessChangedStatus,
    ) -> ReturnCode<()> {
        todo!()
    }

    fn get_requested_deadline_missed_status(
        &self,
        _status: &mut RequestedDeadlineMissedStatus,
    ) -> ReturnCode<()> {
        todo!()
    }

    fn get_requested_incompatible_qos_status(
        &self,
        _status: &mut RequestedIncompatibleQosStatus,
    ) -> ReturnCode<()> {
        todo!()
    }

    fn get_sample_lost_status(&self, _status: &mut SampleLostStatus) -> ReturnCode<()> {
        todo!()
    }

    fn get_sample_rejected_status(&self, _status: &mut SampleRejectedStatus) -> ReturnCode<()> {
        todo!()
    }

    fn get_subscription_matched_status(
        &self,
        _status: &mut SubscriptionMatchedStatus,
    ) -> ReturnCode<()> {
        todo!()
    }

    fn get_topicdescription(&self) -> &dyn TopicDescription<T> {
        todo!()
    }

    fn get_subscriber(&self) -> <Self as SubscriberChild<'a>>::SubscriberType
    where
        Self: SubscriberChild<'a> + Sized,
    {
        todo!()
    }

    fn delete_contained_entities(&self) -> ReturnCode<()> {
        todo!()
    }

    fn wait_for_historical_data(&self) -> ReturnCode<()> {
        todo!()
    }

    fn get_matched_publication_data(
        &self,
        _publication_data: &mut PublicationBuiltinTopicData,
        _publication_handle: InstanceHandle,
    ) -> ReturnCode<()> {
        todo!()
    }

    fn get_match_publication(&self, _publication_handles: &mut [InstanceHandle]) -> ReturnCode<()> {
        todo!()
    }
}

impl<'a, T: DDSType> Entity for RtpsDataReader<'a, T> {
    type Qos = DataReaderQos;

    type Listener = Box<dyn DataReaderListener<T>>;

    fn set_qos(&self, _qos: Option<Self::Qos>) -> ReturnCode<()> {
        todo!()
    }

    fn get_qos(&self) -> ReturnCode<Self::Qos> {
        todo!()
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

impl<'a, T: DDSType> AnyDataReader for RtpsDataReader<'a, T> {}
