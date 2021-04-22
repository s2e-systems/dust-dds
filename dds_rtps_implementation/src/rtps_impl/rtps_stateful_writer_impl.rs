use rust_rtps_pim::{
    behavior::RTPSWriter,
    structure::{
        types::{ChangeKind, Locator, ReliabilityKind, TopicKind, GUID},
        RTPSEndpoint, RTPSEntity, RTPSHistoryCache,
    },
};
use rust_rtps_udp_psm::RtpsUdpPsm;

pub struct RTPSStatefulWriterImpl {}

impl RTPSEntity<RtpsUdpPsm> for RTPSStatefulWriterImpl {
    fn guid(&self) -> GUID<RtpsUdpPsm> {
        todo!()
    }
}

impl RTPSEndpoint<RtpsUdpPsm> for RTPSStatefulWriterImpl {
    fn topic_kind(&self) -> TopicKind {
        todo!()
    }

    fn reliability_level(&self) -> ReliabilityKind {
        todo!()
    }

    fn unicast_locator_list(&self) -> &[Locator<RtpsUdpPsm>] {
        todo!()
    }

    fn multicast_locator_list(&self) -> &[Locator<RtpsUdpPsm>] {
        todo!()
    }
}

impl RTPSWriter<RtpsUdpPsm> for RTPSStatefulWriterImpl {
    fn push_mode(&self) -> bool {
        todo!()
    }

    fn heartbeat_period(&self) -> <RtpsUdpPsm as rust_rtps_pim::behavior::Types>::Duration {
        todo!()
    }

    fn nack_response_delay(&self) -> <RtpsUdpPsm as rust_rtps_pim::behavior::Types>::Duration {
        todo!()
    }

    fn nack_suppression_duration(
        &self,
    ) -> <RtpsUdpPsm as rust_rtps_pim::behavior::Types>::Duration {
        todo!()
    }

    fn last_change_sequence_number(
        &self,
    ) -> <RtpsUdpPsm as rust_rtps_pim::structure::Types>::SequenceNumber {
        todo!()
    }

    fn data_max_size_serialized(&self) -> i32 {
        todo!()
    }

    fn writer_cache(&self) -> &dyn RTPSHistoryCache<RtpsUdpPsm> {
        todo!()
    }

    fn writer_cache_mut(&mut self) -> &mut dyn RTPSHistoryCache<RtpsUdpPsm> {
        todo!()
    }

    fn new_change(
        &mut self,
        _kind: ChangeKind,
        _data: <RtpsUdpPsm as rust_rtps_pim::structure::Types>::Data,
        _inline_qos: <RtpsUdpPsm as rust_rtps_pim::structure::Types>::ParameterVector,
        _handle: <RtpsUdpPsm as rust_rtps_pim::structure::Types>::InstanceHandle,
    ) -> rust_rtps_pim::structure::RTPSCacheChange<RtpsUdpPsm> {
        todo!()
    }
}
