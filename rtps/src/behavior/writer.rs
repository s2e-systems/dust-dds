use crate::{
    messages::submessages::submessage_elements::ParameterList,
    structure::{CacheChange, Endpoint, HistoryCache},
    types::{ChangeKind, InstanceHandle, ReliabilityKind, SequenceNumber, TopicKind, GUID},
};

use super::types::Duration;

pub struct Writer {
    pub endpoint: Endpoint,
    pub push_mode: bool,
    pub heartbeat_period: Duration,
    pub nack_response_delay: Duration,
    pub nack_suppression_duration: Duration,
    pub last_change_sequence_number: SequenceNumber,
    pub writer_cache: HistoryCache,
    pub data_max_sized_serialized: Option<i32>,
}

impl Writer {
    pub fn new(
        guid: GUID,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        push_mode: bool,
        heartbeat_period: Duration,
        nack_response_delay: Duration,
        nack_suppression_duration: Duration,
        data_max_sized_serialized: Option<i32>,
    ) -> Self {
        let endpoint = Endpoint::new(guid, topic_kind, reliability_level);
        Self {
            endpoint,
            push_mode,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            last_change_sequence_number: 0,
            writer_cache: HistoryCache::new(),
            data_max_sized_serialized,
        }
    }

    pub fn new_change(
        &mut self,
        kind: ChangeKind,
        data: Option<Vec<u8>>,
        inline_qos: Option<ParameterList>,
        handle: InstanceHandle,
    ) -> CacheChange {
        self.last_change_sequence_number += 1;
        CacheChange::new(
            kind,
            self.endpoint.entity.guid.into(),
            handle,
            self.last_change_sequence_number,
            data,
            inline_qos,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        behavior::types::constants::DURATION_ZERO,
        types::constants::ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
    };

    #[test]
    fn new_change() {
        let mut writer = Writer::new(
            GUID::new([0; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER),
            TopicKind::WithKey,
            ReliabilityKind::BestEffort,
            true,
            DURATION_ZERO,
            Duration::from_millis(200),
            DURATION_ZERO,
            None,
        );

        let cache_change_seq1 =
            writer.new_change(ChangeKind::Alive, Some(vec![1, 2, 3]), None, [1; 16]);

        let cache_change_seq2 =
            writer.new_change(ChangeKind::NotAliveUnregistered, None, None, [1; 16]);

        assert_eq!(cache_change_seq1.sequence_number(), 1);
        assert_eq!(cache_change_seq1.change_kind(), ChangeKind::Alive);
        assert_eq!(cache_change_seq1.inline_qos(), None);
        assert_eq!(cache_change_seq1.instance_handle(), [1; 16]);

        assert_eq!(cache_change_seq2.sequence_number(), 2);
        assert_eq!(
            cache_change_seq2.change_kind(),
            ChangeKind::NotAliveUnregistered
        );
        assert_eq!(cache_change_seq2.inline_qos(), None);
        assert_eq!(cache_change_seq2.instance_handle(), [1; 16]);
    }
}
