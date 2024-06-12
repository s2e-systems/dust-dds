use super::{
    behavior_types::{Duration, InstanceHandle},
    endpoint::RtpsEndpoint,
    messages::{
        self,
        submessage_elements::{Data, ParameterList},
    },
    types::{ChangeKind, Guid, Locator, SequenceNumber},
    writer_history_cache::RtpsWriterCacheChange,
};

pub struct RtpsWriter {
    endpoint: RtpsEndpoint,
    _push_mode: bool,
    heartbeat_period: Duration,
    _nack_response_delay: Duration,
    _nack_suppression_duration: Duration,
    last_change_sequence_number: SequenceNumber,
    data_max_size_serialized: usize,
}

impl RtpsWriter {
    pub fn new(
        endpoint: RtpsEndpoint,
        push_mode: bool,
        heartbeat_period: Duration,
        nack_response_delay: Duration,
        nack_suppression_duration: Duration,
        data_max_size_serialized: usize,
    ) -> Self {
        Self {
            endpoint,
            _push_mode: push_mode,
            heartbeat_period,
            _nack_response_delay: nack_response_delay,
            _nack_suppression_duration: nack_suppression_duration,
            last_change_sequence_number: 0,
            data_max_size_serialized,
        }
    }

    pub fn guid(&self) -> Guid {
        self.endpoint.guid()
    }

    pub fn unicast_locator_list(&self) -> &[Locator] {
        self.endpoint.unicast_locator_list()
    }

    pub fn multicast_locator_list(&self) -> &[Locator] {
        self.endpoint.multicast_locator_list()
    }

    pub fn _push_mode(&self) -> bool {
        self._push_mode
    }

    pub fn heartbeat_period(&self) -> Duration {
        self.heartbeat_period
    }

    pub fn data_max_size_serialized(&self) -> usize {
        self.data_max_size_serialized
    }

    pub fn new_change(
        &mut self,
        kind: ChangeKind,
        data: Data,
        inline_qos: ParameterList,
        handle: InstanceHandle,
        timestamp: messages::types::Time,
    ) -> RtpsWriterCacheChange {
        self.last_change_sequence_number += 1;
        RtpsWriterCacheChange::new(
            kind,
            self.guid(),
            handle,
            self.last_change_sequence_number,
            timestamp,
            data,
            inline_qos,
        )
    }
}
