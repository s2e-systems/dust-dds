use crate::{
    behavior::types::Duration,
    messages::types::Count,
    structure::{
        cache_change::RtpsCacheChange,
        endpoint::RtpsEndpoint,
        history_cache::RtpsHistoryCacheConstructor,
        types::{ChangeKind, Guid, InstanceHandle, ReliabilityKind, SequenceNumber, TopicKind},
    },
};

pub struct RtpsWriter<L, C> {
    pub endpoint: RtpsEndpoint<L>,
    pub push_mode: bool,
    pub heartbeat_period: Duration,
    pub nack_response_delay: Duration,
    pub nack_suppression_duration: Duration,
    pub last_change_sequence_number: SequenceNumber,
    pub data_max_size_serialized: Option<i32>,
    pub writer_cache: C,
    pub heartbeat_count: Count, // This member is a deviation from the standard to implement the behavior
}

impl<L, C> RtpsWriter<L, C> {
    pub fn new(
        guid: Guid,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: L,
        multicast_locator_list: L,
        push_mode: bool,
        heartbeat_period: Duration,
        nack_response_delay: Duration,
        nack_suppression_duration: Duration,
        data_max_size_serialized: Option<i32>,
    ) -> Self
    where
        C: RtpsHistoryCacheConstructor,
    {
        Self {
            endpoint: RtpsEndpoint::new(
                guid,
                topic_kind,
                reliability_level,
                unicast_locator_list,
                multicast_locator_list,
            ),
            push_mode,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            last_change_sequence_number: 0,
            data_max_size_serialized,
            writer_cache: C::new(),
            heartbeat_count: Count(0),
        }
    }
}

impl<L, C> RtpsWriterOperations for RtpsWriter<L, C> {
    fn new_change<'a, P, D>(
        &mut self,
        kind: ChangeKind,
        data: D,
        inline_qos: P,
        handle: InstanceHandle,
    ) -> RtpsCacheChange<P, D> {
        self.last_change_sequence_number = self.last_change_sequence_number + 1;
        RtpsCacheChange {
            kind,
            writer_guid: self.endpoint.entity.guid,
            instance_handle: handle,
            sequence_number: self.last_change_sequence_number,
            data_value: data,
            inline_qos,
        }
    }
}

pub trait RtpsWriterOperations {
    fn new_change<'a, P, D>(
        &mut self,
        kind: ChangeKind,
        data: D,
        inline_qos: P,
        handle: InstanceHandle,
    ) -> RtpsCacheChange<P, D>;
}
