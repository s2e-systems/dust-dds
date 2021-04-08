use rust_rtps_pim::structure::RTPSHistoryCache;
use rust_rtps_udp_psm::RtpsUdpPsm;

pub struct HistoryCacheImpl;

impl RTPSHistoryCache for HistoryCacheImpl {
    type PSM = RtpsUdpPsm;

    fn new() -> Self {
        todo!()
    }

    fn add_change(&mut self, _change: rust_rtps_pim::structure::RTPSCacheChange<Self::PSM>) {
        todo!()
    }

    fn remove_change(
        &mut self,
        _seq_num: &<Self::PSM as rust_rtps_pim::structure::Types>::SequenceNumber,
    ) {
        todo!()
    }

    fn get_change(
        &self,
        _seq_num: &<Self::PSM as rust_rtps_pim::structure::Types>::SequenceNumber,
    ) -> Option<&rust_rtps_pim::structure::RTPSCacheChange<Self::PSM>> {
        todo!()
    }

    fn get_seq_num_min(
        &self,
    ) -> Option<<Self::PSM as rust_rtps_pim::structure::Types>::SequenceNumber> {
        todo!()
    }

    fn get_seq_num_max(
        &self,
    ) -> Option<<Self::PSM as rust_rtps_pim::structure::Types>::SequenceNumber> {
        todo!()
    }
}
