use crate::{
    dcps_psm::{Duration, InstanceHandle, Time},
    infrastructure::qos::DataWriterQos,
    return_type::DdsResult,
};

use super::{
    endpoint::RtpsEndpoint,
    history_cache::{RtpsParameter, RtpsWriterCacheChange, WriterHistoryCache},
    types::{ChangeKind, Guid, Locator, SequenceNumber, TopicKind},
};

pub struct RtpsWriter {
    endpoint: RtpsEndpoint,
    push_mode: bool,
    heartbeat_period: Duration,
    nack_response_delay: Duration,
    nack_suppression_duration: Duration,
    last_change_sequence_number: SequenceNumber,
    data_max_size_serialized: Option<i32>,
    writer_cache: WriterHistoryCache,
    qos: DataWriterQos,
}

impl RtpsWriter {
    pub fn new(
        endpoint: RtpsEndpoint,
        push_mode: bool,
        heartbeat_period: Duration,
        nack_response_delay: Duration,
        nack_suppression_duration: Duration,
        data_max_size_serialized: Option<i32>,
        qos: DataWriterQos,
    ) -> Self {
        Self {
            endpoint,
            push_mode,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            last_change_sequence_number: 0,
            data_max_size_serialized,
            writer_cache: WriterHistoryCache::new(),
            qos,
        }
    }
}

impl RtpsWriter {
    pub fn guid(&self) -> Guid {
        self.endpoint.guid()
    }
}

impl RtpsWriter {
    pub fn topic_kind(&self) -> TopicKind {
        self.endpoint.topic_kind()
    }

    pub fn unicast_locator_list(&self) -> &[Locator] {
        self.endpoint.unicast_locator_list()
    }

    pub fn multicast_locator_list(&self) -> &[Locator] {
        self.endpoint.multicast_locator_list()
    }
}

impl RtpsWriter {
    pub fn push_mode(&self) -> bool {
        self.push_mode
    }

    pub fn heartbeat_period(&self) -> Duration {
        self.heartbeat_period
    }

    pub fn nack_response_delay(&self) -> Duration {
        self.nack_response_delay
    }

    pub fn nack_suppression_duration(&self) -> Duration {
        self.nack_suppression_duration
    }

    pub fn last_change_sequence_number(&self) -> SequenceNumber {
        self.last_change_sequence_number
    }

    pub fn data_max_size_serialized(&self) -> Option<i32> {
        self.data_max_size_serialized
    }

    pub fn writer_cache(&self) -> &WriterHistoryCache {
        &self.writer_cache
    }

    pub fn writer_cache_mut(&mut self) -> &mut WriterHistoryCache {
        &mut self.writer_cache
    }
}

impl RtpsWriter {
    pub fn new_change(
        &mut self,
        kind: ChangeKind,
        data: Vec<u8>,
        inline_qos: Vec<RtpsParameter>,
        handle: InstanceHandle,
        timestamp: Time,
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

impl RtpsWriter {
    pub fn get_qos(&self) -> &DataWriterQos {
        &self.qos
    }

    pub fn set_qos(&mut self, qos: DataWriterQos) -> DdsResult<()> {
        qos.is_consistent()?;
        self.qos = qos;
        Ok(())
    }
}
