use crate::builtin_topics::PublicationBuiltinTopicData;
use crate::dds_infrastructure::qos::DataReaderQos;
use crate::dds_infrastructure::qos_policy::ReliabilityQosPolicyKind;
use crate::dds_infrastructure::read_condition::ReadCondition;
use crate::dds_infrastructure::sample_info::SampleInfo;
use crate::dds_infrastructure::status::{
    InstanceStateKind, LivelinessChangedStatus, RequestedDeadlineMissedStatus,
    RequestedIncompatibleQosStatus, SampleLostStatus, SampleRejectedStatus, SampleStateKind,
    SubscriptionMatchedStatus, ViewStateKind,
};
use crate::dds_rtps_implementation::rtps_object::RtpsObject;
use crate::rtps::behavior;
use crate::rtps::behavior::StatefulReader;
use crate::rtps::types::{ReliabilityKind, GUID};
use crate::types::{Data, InstanceHandle, ReturnCode, TopicKind};
use std::cell::Ref;

pub struct RtpsDataReaderInner {
    pub reader: StatefulReader,
    pub qos: DataReaderQos,
}

impl RtpsDataReaderInner {
    pub fn new(guid: GUID, topic_kind: TopicKind, qos: DataReaderQos) -> Self {
        qos.is_consistent()
            .expect("RtpsDataReaderInner can only be created with consistent QoS");

        let reliability_level = match qos.reliability.kind {
            ReliabilityQosPolicyKind::BestEffortReliabilityQos => ReliabilityKind::BestEffort,
            ReliabilityQosPolicyKind::ReliableReliabilityQos => ReliabilityKind::Reliable,
        };
        let expects_inline_qos = false;
        let heartbeat_response_delay = behavior::types::constants::DURATION_ZERO;
        let reader = StatefulReader::new(guid, topic_kind, reliability_level, expects_inline_qos, heartbeat_response_delay);
        Self { reader, qos }
    }
}

pub type RtpsDataReader<'a> = Ref<'a, RtpsObject<RtpsDataReaderInner>>;

impl RtpsObject<RtpsDataReaderInner> {
    pub fn read(
        &self,
        _data_values: &mut [Data],
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
        _data_values: &mut [Data],
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
        _data_values: &mut [Data],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        // a_condition: ReadCondition,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn take_w_condition(
        &self,
        _data_values: &mut [Data],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _a_condition: ReadCondition,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn read_next_sample(
        &self,
        _data_value: &mut [Data],
        _sample_info: &mut [SampleInfo],
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn take_next_sample(
        &self,
        _data_value: &mut [Data],
        _sample_info: &mut [SampleInfo],
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn read_instance(
        &self,
        _data_values: &mut [Data],
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
        _data_values: &mut [Data],
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
        _data_values: &mut [Data],
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
        _data_values: &mut [Data],
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
        _data_values: &mut [Data],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _previous_handle: InstanceHandle,
        _a_condition: ReadCondition,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn take_next_instance_w_condition(
        &self,
        _data_values: &mut [Data],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _previous_handle: InstanceHandle,
        _a_condition: ReadCondition,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn return_loan(
        &self,
        _data_values: &mut [Data],
        _sample_infos: &mut [SampleInfo],
    ) -> ReturnCode<()> {
        todo!()
    }

    // pub fn get_key_value(&self, _key_holder: &mut T, _handle: InstanceHandle) -> ReturnCode<()> {
    //     todo!()
    // }

    pub fn lookup_instance(&self, _instance: &InstanceHandle) -> InstanceHandle {
        todo!()
    }

    pub fn get_liveliness_changed_status(
        &self,
        _status: &mut LivelinessChangedStatus,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_requested_deadline_missed_status(
        &self,
        _status: &mut RequestedDeadlineMissedStatus,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_requested_incompatible_qos_status(
        &self,
        _status: &mut RequestedIncompatibleQosStatus,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_sample_lost_status(&self, _status: &mut SampleLostStatus) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_sample_rejected_status(&self, _status: &mut SampleRejectedStatus) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_subscription_matched_status(
        &self,
        _status: &mut SubscriptionMatchedStatus,
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
