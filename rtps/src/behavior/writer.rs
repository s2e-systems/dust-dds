use crate::{
    messages::submessages::submessage_elements::{ParameterList, SerializedData},
    structure::{CacheChange, Endpoint, HistoryCache},
    types::{ChangeKind, InstanceHandle, SequenceNumber},
};

use super::types::Duration;

pub trait Writer: Endpoint {
    fn push_mode(&self) -> bool;
    fn heartbeat_period(&self) -> Duration;
    fn nack_response_delay(&self) -> Duration;
    fn nack_suppression_duration(&self) -> Duration;
    fn last_change_sequence_number(&self) -> SequenceNumber;
    fn writer_cache(&mut self) -> &mut HistoryCache;
    fn data_max_sized_serialized(&self) -> i32;
    fn new_change(
        &mut self,
        kind: ChangeKind,
        data: SerializedData,
        inline_qos: ParameterList,
        handle: InstanceHandle,
    ) -> CacheChange;
}
