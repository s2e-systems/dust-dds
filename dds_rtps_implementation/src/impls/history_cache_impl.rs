use rust_rtps_pim::structure::{RTPSCacheChange, RTPSHistoryCache};
use rust_rtps_udp_psm::RtpsUdpPsm;

pub struct HistoryCacheImpl {
    changes: Vec<RTPSCacheChange<RtpsUdpPsm>>,
}

impl RTPSHistoryCache<RtpsUdpPsm> for HistoryCacheImpl {

    fn new() -> Self {
        Self {
            changes: Vec::new(),
        }
    }

    fn add_change(&mut self, change: rust_rtps_pim::structure::RTPSCacheChange<RtpsUdpPsm>) {
        self.changes.push(change)
    }

    fn remove_change(
        &mut self,
        seq_num: &<RtpsUdpPsm as rust_rtps_pim::structure::Types>::SequenceNumber,
    ) {
        self.changes.retain(|x| &x.sequence_number != seq_num)
    }

    fn get_change(
        &self,
        seq_num: &<RtpsUdpPsm as rust_rtps_pim::structure::Types>::SequenceNumber,
    ) -> Option<&rust_rtps_pim::structure::RTPSCacheChange<RtpsUdpPsm>> {
        self.changes.iter().find(|x| &x.sequence_number == seq_num)
    }

    fn get_seq_num_min(
        &self,
    ) -> Option<<RtpsUdpPsm as rust_rtps_pim::structure::Types>::SequenceNumber> {
        self.changes.iter().map(|x| x.sequence_number).min()
    }

    fn get_seq_num_max(
        &self,
    ) -> Option<<RtpsUdpPsm as rust_rtps_pim::structure::Types>::SequenceNumber> {
        self.changes.iter().map(|x| x.sequence_number).max()
    }
}
