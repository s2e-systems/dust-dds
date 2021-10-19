use crate::{
    behavior::types::Duration,
    messages::submessage_elements::Parameter,
    structure::{
        types::{
            ChangeKind, Guid, InstanceHandle, Locator, ReliabilityKind, SequenceNumber, TopicKind,
        },
        RtpsCacheChange, RtpsEndpoint, RtpsHistoryCache,
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
