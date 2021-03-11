use rust_rtps::{
    behavior::RTPSWriter,
    structure::{RTPSCacheChange, RTPSEndpoint, RTPSEntity, RTPSHistoryCache},
};

use super::history_cache_impl::HistoryCache;

pub struct WriterImpl;

impl RTPSEntity for WriterImpl {
    fn guid(&self) -> rust_rtps::types::GUID {
        todo!()
    }
}

impl RTPSEndpoint for WriterImpl {
    fn unicast_locator_list(&self) -> &[rust_rtps::types::Locator] {
        todo!()
    }

    fn multicast_locator_list(&self) -> &[rust_rtps::types::Locator] {
        todo!()
    }

    fn topic_kind(&self) -> rust_rtps::types::TopicKind {
        todo!()
    }

    fn reliability_level(&self) -> rust_rtps::types::ReliabilityKind {
        todo!()
    }
}

impl RTPSWriter for WriterImpl {
    type HistoryCacheType = HistoryCache;

    fn push_mode(&self) -> bool {
        todo!()
    }

    fn heartbeat_period(&self) -> rust_rtps::behavior::types::Duration {
        todo!()
    }

    fn nack_response_delay(&self) -> rust_rtps::behavior::types::Duration {
        todo!()
    }

    fn nack_suppression_duration(&self) -> rust_rtps::behavior::types::Duration {
        todo!()
    }

    fn last_change_sequence_number(&self) -> rust_rtps::types::SequenceNumber {
        todo!()
    }

    fn writer_cache(&mut self) -> &mut Self::HistoryCacheType {
        todo!()
    }

    fn data_max_sized_serialized(&self) -> i32 {
        todo!()
    }

    fn new_change(
        &mut self,
        _kind: rust_rtps::types::ChangeKind,
        _data: <<Self::HistoryCacheType as RTPSHistoryCache>::CacheChangeType as RTPSCacheChange>::Data,
        _inline_qos: rust_rtps::messages::submessages::submessage_elements::ParameterList,
        _handle: rust_rtps::types::InstanceHandle,
    ) -> <Self::HistoryCacheType as RTPSHistoryCache>::CacheChangeType {
        todo!()
    }
}
