use crate::{
    structure::{RTPSCacheChange, RTPSEndpoint, RTPSEntity, RTPSHistoryCache},
    RtpsPsm,
};

pub struct RTPSWriter<PSM: RtpsPsm, HistoryCache: RTPSHistoryCache> {
    pub endpoint: RTPSEndpoint<PSM>,
    pub push_mode: bool,
    pub heartbeat_period: PSM::Duration,
    pub nack_response_delay: PSM::Duration,
    pub nack_suppression_duration: PSM::Duration,
    pub last_change_sequence_number: PSM::SequenceNumber,
    pub data_max_size_serialized: i32,
    pub writer_cache: HistoryCache,
}

impl<PSM: RtpsPsm, HistoryCache: RTPSHistoryCache> RTPSWriter<PSM, HistoryCache> {
    pub fn new(
        guid: PSM::Guid,
        topic_kind: PSM::TopicKind,
        reliability_level: PSM::ReliabilityKind,
        unicast_locator_list: PSM::LocatorList,
        multicast_locator_list: PSM::LocatorList,
        push_mode: bool,
        heartbeat_period: PSM::Duration,
        nack_response_delay: PSM::Duration,
        nack_suppression_duration: PSM::Duration,
        data_max_size_serialized: i32,
    ) -> Self {
        let entity = RTPSEntity { guid };
        let endpoint = RTPSEndpoint {
            entity,
            topic_kind,
            reliability_level,
            unicast_locator_list,
            multicast_locator_list,
        };

        Self {
            endpoint,
            push_mode,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            last_change_sequence_number: 0i64.into(),
            data_max_size_serialized,
            writer_cache: HistoryCache::new(),
        }
    }

    pub fn new_change(
        &mut self,
        kind: PSM::ChangeKind,
        data: PSM::Data,
        inline_qos: PSM::ParameterList,
        handle: PSM::InstanceHandle,
    ) -> RTPSCacheChange<PSM> {
        self.last_change_sequence_number = (self.last_change_sequence_number.into() + 1).into();

        RTPSCacheChange {
            kind,
            writer_guid: self.endpoint.guid,
            instance_handle: handle,
            sequence_number: self.last_change_sequence_number,
            data_value: data,
            inline_qos,
        }
    }
}
