use crate::builtin_topics::SubscriptionBuiltinTopicData;
use crate::dds_infrastructure::qos::DataWriterQos;
use crate::dds_infrastructure::qos_policy::ReliabilityQosPolicyKind;
use crate::dds_infrastructure::status::{
    LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
    PublicationMatchedStatus,
};
use crate::dds_rtps_implementation::rtps_object::RtpsObject;
use crate::dds_rtps_implementation::rtps_topic::RtpsTopicInner;
use crate::rtps::behavior;
use crate::rtps::behavior::StatefulWriter;
use crate::rtps::types::{ReliabilityKind, GUID};
use crate::types::{Data, Duration, InstanceHandle, ReturnCode, Time};
use std::sync::{Arc, Mutex, RwLockReadGuard};

pub struct RtpsDataWriterInner {
    pub writer: StatefulWriter,
    pub qos: DataWriterQos,
    pub topic: Mutex<Option<Arc<RtpsTopicInner>>>,
}

impl RtpsDataWriterInner {
    pub fn new(guid: GUID, topic: Arc<RtpsTopicInner>, qos: DataWriterQos) -> Self {
        qos.is_consistent()
            .expect("RtpsDataWriterInner can only be created with consistent QoS");

        let topic_kind = topic.topic_kind;
        let reliability_level = match qos.reliability.kind {
            ReliabilityQosPolicyKind::BestEffortReliabilityQos => ReliabilityKind::BestEffort,
            ReliabilityQosPolicyKind::ReliableReliabilityQos => ReliabilityKind::Reliable,
        };
        let push_mode = true;
        let data_max_sized_serialized = None;
        let heartbeat_period = behavior::types::Duration::from_millis(500);
        let nack_response_delay = behavior::types::constants::DURATION_ZERO;
        let nack_supression_duration = behavior::types::constants::DURATION_ZERO;
        let writer = StatefulWriter::new(
            guid,
            topic_kind,
            reliability_level,
            push_mode,
            data_max_sized_serialized,
            heartbeat_period,
            nack_response_delay,
            nack_supression_duration,
        );

        Self {
            writer,
            qos,
            topic: Mutex::new(Some(topic)),
        }
    }
}

pub type RtpsDataWriter<'a> = RwLockReadGuard<'a, RtpsObject<RtpsDataWriterInner>>;

impl RtpsObject<RtpsDataWriterInner> {
    pub fn register_instance(
        &self,
        _instance: InstanceHandle,
    ) -> ReturnCode<Option<InstanceHandle>> {
        todo!()
    }

    pub fn register_instance_w_timestamp(
        &self,
        _instance: InstanceHandle,
        _timestamp: Time,
    ) -> ReturnCode<Option<InstanceHandle>> {
        todo!()
    }

    pub fn unregister_instance(
        &self,
        _instance: InstanceHandle,
        _handle: Option<InstanceHandle>,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn unregister_instance_w_timestamp(
        &self,
        _instance: InstanceHandle,
        _handle: Option<InstanceHandle>,
        _timestamp: Time,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_key_value(
        &self,
        _key_holder: &mut InstanceHandle,
        _handle: InstanceHandle,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn lookup_instance(
        &self,
        _instance: &InstanceHandle,
    ) -> ReturnCode<Option<InstanceHandle>> {
        todo!()
    }

    pub fn write(&self, _data: Data, _handle: Option<InstanceHandle>) -> ReturnCode<()> {
        todo!()
    }

    pub fn write_w_timestamp(
        &self,
        _data: Data,
        _handle: Option<InstanceHandle>,
        _timestamp: Time,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn dispose(&self, _data: Data, _handle: Option<InstanceHandle>) -> ReturnCode<()> {
        todo!()
    }

    pub fn dispose_w_timestamp(
        &self,
        _data: Data,
        _handle: Option<InstanceHandle>,
        _timestamp: Time,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn wait_for_acknowledgments(&self, _max_wait: Duration) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_liveliness_lost_status(&self, _status: &mut LivelinessLostStatus) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_offered_deadline_missed_status(
        &self,
        _status: &mut OfferedDeadlineMissedStatus,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_offered_incompatible_qos_status(
        &self,
        _status: &mut OfferedIncompatibleQosStatus,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_publication_matched_status(
        &self,
        _status: &mut PublicationMatchedStatus,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn assert_liveliness(&self) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_matched_subscription_data(
        &self,
        _subscription_data: SubscriptionBuiltinTopicData,
        _subscription_handle: InstanceHandle,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_matched_subscriptions(
        &self,
        _subscription_handles: &mut [InstanceHandle],
    ) -> ReturnCode<()> {
        todo!()
    }
}
