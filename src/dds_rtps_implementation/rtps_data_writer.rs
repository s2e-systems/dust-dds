use crate::builtin_topics::SubscriptionBuiltinTopicData;
use crate::dds_infrastructure::qos::DataWriterQos;
use crate::dds_infrastructure::status::{
    LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
    PublicationMatchedStatus,
};
use crate::dds_rtps_implementation::rtps_object::RtpsObject;
use crate::rtps::behavior::StatefulWriter;
use crate::rtps::types::GUID;
use crate::types::{Data, Duration, InstanceHandle, ReturnCode, Time, TopicKind};
use std::cell::Ref;

pub struct RtpsDataWriterInner {
    // pub writer: StatefulWriter,
    // pub qos: DataWriterQos,
}

impl RtpsDataWriterInner {
    pub fn new(guid: GUID, topic_kind: TopicKind, qos: DataWriterQos) -> Self {
        qos.is_consistent()
            .expect("RtpsDataWriterInner can only be created with consistent QoS");

        // let reliablity_level =  match qos.reliability.kind {

        // };
        // let push_mode = true;
        // let writer = StatefulWriter::new(
        //     guid,
        //     topic_kind,
        //     reliability_level,
        //     push_mode,
        //     writer_cache,
        //     data_max_sized_serialized,
        //     heartbeat_period,
        //     nack_response_delay,
        //     nack_supression_duration,
        // );

        Self { }
    }
}

pub type RtpsDataWriter<'a> = Ref<'a, RtpsObject<RtpsDataWriterInner>>;

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
