use crate::{behavior, structure::{self, RTPSCacheChange, RTPSEndpoint, RTPSHistoryCache, types::ChangeKind}};

pub trait RTPSWriter<PSM: structure::Types + behavior::Types>: RTPSEndpoint<PSM> {
    fn push_mode(&self) -> bool;
    fn heartbeat_period(&self) -> PSM::Duration;
    fn nack_response_delay(&self) -> PSM::Duration;
    fn nack_suppression_duration(&self) -> PSM::Duration;
    fn last_change_sequence_number(&self) -> PSM::SequenceNumber;
    fn data_max_size_serialized(&self) -> i32;
    fn writer_cache(&self) -> &dyn RTPSHistoryCache<PSM>;
    fn writer_cache_mut(&mut self) -> &mut dyn RTPSHistoryCache<PSM>;

    fn new_change(
        &mut self,
        kind: ChangeKind,
        data: PSM::Data,
        inline_qos: &[PSM::Parameter],
        handle: PSM::InstanceHandle,
    ) -> RTPSCacheChange<PSM>;
}
