use crate::endpoint::{Endpoint};
use crate::cache::{WriterHistoryCache};
use crate::types::{Duration, SequenceNumber};

pub struct Writer {
    endpoint : Endpoint,
    pub push_mode: bool,
    pub heartbeat_period: Duration,
    pub nack_response_delay: Duration,
    pub nack_suppression_duration : Duration,
    pub last_change_sequence_number: SequenceNumber,
    pub writer_cache: WriterHistoryCache,
    pub data_max_sized_serialized : Option<i32>,
}
