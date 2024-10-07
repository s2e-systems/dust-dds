use crate::implementation::data_representation_builtin_endpoints::discovered_reader_data::ReaderProxy;

use super::{
    behavior_types::{Duration, InstanceHandle},
    cache_change::RtpsCacheChange,
    endpoint::RtpsEndpoint,
    messages::{
        self,
        submessage_elements::{Data, ParameterList},
    },
    types::{ChangeKind, Guid, Locator, SequenceNumber},
};

pub trait WriterHistoryCache {
    fn add_change(&mut self, cache_change: RtpsCacheChange);

    fn remove_change(&mut self, sequence_number: SequenceNumber);

    fn get_changes(&self) -> &[RtpsCacheChange];
}

pub trait TransportWriter {
    fn add_matched_reader(&mut self, reader_proxy: ReaderProxy);

    fn delete_matched_reader(&mut self, reader_guid: Guid);
}

pub struct RtpsWriter {
    endpoint: RtpsEndpoint,
    changes: Vec<RtpsCacheChange>,
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
            changes: Vec::new(),
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
    ) -> RtpsCacheChange {
        self.last_change_sequence_number += 1;

        RtpsCacheChange {
            kind,
            writer_guid: self.guid(),
            instance_handle: handle,
            sequence_number: self.last_change_sequence_number,
            source_timestamp: Some(timestamp),
            data_value: data,
            inline_qos,
        }
    }
}

impl RtpsWriter {
    pub fn add_change(&mut self, cache_change: RtpsCacheChange) {
        self.changes.push(cache_change);
    }

    pub fn remove_change(&mut self, sequence_number: SequenceNumber) {
        self.changes
            .retain(|cc| cc.sequence_number() != sequence_number);
    }

    pub fn get_changes(&self) -> &[RtpsCacheChange] {
        &self.changes
    }
}
