use crate::dds_infrastructure::read_condition::ReadCondition;
use crate::dds_infrastructure::sample_info::SampleInfo;
use crate::dds_infrastructure::status::{
    ViewStateKind,
    SampleStateKind,
    InstanceStateKind,
    SampleRejectedStatus,
    SubscriptionMatchedStatus,
    SampleLostStatus,
    RequestedIncompatibleQosStatus,
    LivelinessChangedStatus,
    RequestedDeadlineMissedStatus};
use crate::dds_rtps_implementation::rtps_object::RtpsObjectReference;

use crate::types::{DDSType, InstanceHandle, ReturnCode};
use crate::builtin_topics::PublicationBuiltinTopicData;
pub struct RtpsDataReaderInner<T: DDSType> {
    marker: std::marker::PhantomData<T>,
}

impl<T: DDSType> Default for RtpsDataReaderInner<T> {
    fn default() -> Self {
        Self {
            marker: std::marker::PhantomData,
        }
    }
}

pub type RtpsDataReader<'a, T> = RtpsObjectReference<'a, RtpsDataReaderInner<T>>;

impl<'a, T: DDSType> RtpsDataReader<'a, T> {
    pub fn read(
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

    pub fn take(
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

    pub fn read_w_condition(
        &self,
        _data_values: &mut [T],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        // a_condition: ReadCondition,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn take_w_condition(
        &self,
        _data_values: &mut [T],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _a_condition: ReadCondition,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn read_next_sample(
        &self,
        _data_value: &mut [T],
        _sample_info: &mut [SampleInfo],
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn take_next_sample(
        &self,
        _data_value: &mut [T],
        _sample_info: &mut [SampleInfo],
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn read_instance(
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

    pub fn take_instance(
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

    pub fn read_next_instance(
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

    pub fn take_next_instance(
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

    pub fn read_next_instance_w_condition(
        &self,
        _data_values: &mut [T],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _previous_handle: InstanceHandle,
        _a_condition: ReadCondition,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn take_next_instance_w_condition(
        &self,
        _data_values: &mut [T],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _previous_handle: InstanceHandle,
        _a_condition: ReadCondition,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn return_loan(
        &self,
        _data_values: &mut [T],
        _sample_infos: &mut [SampleInfo],
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_key_value(&self, _key_holder: &mut T, _handle: InstanceHandle) -> ReturnCode<()> {
        todo!()
    }

    pub fn lookup_instance(
        &self,
        _instance: &T,
    ) -> InstanceHandle {
        todo!()
    }

    pub fn get_liveliness_changed_status(
        &self,
        _status: &mut LivelinessChangedStatus
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_requested_deadline_missed_status(
        &self,
        _status: &mut RequestedDeadlineMissedStatus
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_requested_incompatible_qos_status(
        &self,
        _status: &mut RequestedIncompatibleQosStatus
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_sample_lost_status(
        &self,
        _status: &mut SampleLostStatus
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_sample_rejected_status(
        &self,
        _status: &mut SampleRejectedStatus
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_subscription_matched_status(
        &self,
        _status: &mut SubscriptionMatchedStatus
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn delete_contained_entities(&self) -> ReturnCode<()> {
        todo!()
    }

    pub fn wait_for_historical_data(&self) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_matched_publication_data(
        &self,
        _publication_data: &mut PublicationBuiltinTopicData,
        _publication_handle: InstanceHandle,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_match_publication(
        &self,
        _publication_handles: &mut [InstanceHandle],
    ) -> ReturnCode<()> {
        todo!()
    }
}
