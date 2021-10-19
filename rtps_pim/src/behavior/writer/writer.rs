use core::ops::{Deref, DerefMut};

use crate::{
    behavior::types::Duration,
    messages::submessage_elements::Parameter,
    structure::{
        types::{ChangeKind, Guid, InstanceHandle, ReliabilityKind, SequenceNumber, TopicKind},
        RtpsCacheChange, RtpsEndpoint, RtpsHistoryCache,
    },
};

pub struct RtpsWriter<L, C> {
    endpoint: RtpsEndpoint<L>,
    pub push_mode: bool,
    pub heartbeat_period: Duration,
    pub nack_response_delay: Duration,
    pub nack_suppression_duration: Duration,
    pub last_change_sequence_number: SequenceNumber,
    pub data_max_size_serialized: Option<i32>,
    pub writer_cache: C,
}

impl<L, C> Deref for RtpsWriter<L, C> {
    type Target = RtpsEndpoint<L>;

    fn deref(&self) -> &Self::Target {
        &self.endpoint
    }
}

impl<L, C> DerefMut for RtpsWriter<L, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.endpoint
    }
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
        C: for<'a> RtpsHistoryCache<'a>,
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
        }
    }
}

impl<L, C> RtpsWriterOperations<C> for RtpsWriter<L, C> {
    fn new_change<'a>(
        &mut self,
        kind: ChangeKind,
        data: C::CacheChangeDataType,
        inline_qos: &'a [Parameter<'a>],
        handle: InstanceHandle,
    ) -> RtpsCacheChange<'a, C::CacheChangeDataType>
    where
        C: RtpsHistoryCache<'a>,
    {
        self.last_change_sequence_number = self.last_change_sequence_number + 1;
        RtpsCacheChange {
            kind,
            writer_guid: self.endpoint.guid,
            instance_handle: handle,
            sequence_number: self.last_change_sequence_number,
            data_value: data,
            inline_qos,
        }
    }
}

pub trait RtpsWriterOperations<C> {
    fn new_change<'a>(
        &mut self,
        kind: ChangeKind,
        data: C::CacheChangeDataType,
        inline_qos: &'a [Parameter<'a>],
        handle: InstanceHandle,
    ) -> RtpsCacheChange<'a, C::CacheChangeDataType>
    where
        C: RtpsHistoryCache<'a>;
}
