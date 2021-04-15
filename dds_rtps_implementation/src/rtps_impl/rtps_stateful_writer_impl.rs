use rust_rtps_pim::{behavior::RTPSWriter, structure::{RTPSEndpoint, RTPSEntity, types::{ChangeKind, GUID, Locator, ReliabilityKind, TopicKind}}};
use rust_rtps_udp_psm::{RtpsUdpPsm};

use super::rtps_history_cache_impl::RTPSHistoryCacheImpl;

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

impl RTPSWriter<RtpsUdpPsm, RTPSHistoryCacheImpl> for RTPSStatefulWriterImpl {
    fn push_mode(&self) -> bool {
        todo!()
    }

    fn heartbeat_period(&self) -> <RtpsUdpPsm as rust_rtps_pim::behavior::Types>::Duration {
        todo!()
    }

    fn nack_response_delay(&self) -> <RtpsUdpPsm as rust_rtps_pim::behavior::Types>::Duration {
        todo!()
    }

    fn nack_suppression_duration(&self) -> <RtpsUdpPsm as rust_rtps_pim::behavior::Types>::Duration {
        todo!()
    }

    fn last_change_sequence_number(&self) -> <RtpsUdpPsm as rust_rtps_pim::structure::Types>::SequenceNumber {
        todo!()
    }

    fn data_max_size_serialized(&self) -> i32 {
        todo!()
    }

    fn writer_cache(&self) -> &RTPSHistoryCacheImpl {
        todo!()
    }

    fn writer_cache_mut(&mut self) -> &mut RTPSHistoryCacheImpl {
        todo!()
    }

    fn new_change(
        &mut self,
        kind: ChangeKind,
        data: <RtpsUdpPsm as rust_rtps_pim::structure::Types>::Data,
        inline_qos: <RtpsUdpPsm as rust_rtps_pim::structure::Types>::ParameterVector,
        handle:<RtpsUdpPsm as rust_rtps_pim::structure::Types>::InstanceHandle,
    ) -> rust_rtps_pim::structure::RTPSCacheChange<RtpsUdpPsm> {
        todo!()
    }
}