use rust_rtps_pim::structure::{RTPSCacheChange, RTPSHistoryCache};
use rust_rtps_udp_psm::RtpsUdpPsm;

pub struct HistoryCacheImpl {
    changes: Vec<RTPSCacheChange<RtpsUdpPsm>>,
}

impl RTPSHistoryCache for HistoryCacheImpl {
    type PSM = RtpsUdpPsm;

    fn new() -> Self {
        Self {
            changes: Vec::new(),
        }
    }

    fn add_change(&mut self, change: rust_rtps_pim::structure::RTPSCacheChange<Self::PSM>) {
        self.changes.push(change)
    }

    fn remove_change(
        &mut self,
        seq_num: &<Self::PSM as rust_rtps_pim::structure::Types>::SequenceNumber,
    ) {
        self.changes.retain(|x| &x.sequence_number != seq_num)
    }

    fn get_change(
        &self,
        seq_num: &<Self::PSM as rust_rtps_pim::structure::Types>::SequenceNumber,
    ) -> Option<&rust_rtps_pim::structure::RTPSCacheChange<Self::PSM>> {
        self.changes.iter().find(|x| &x.sequence_number == seq_num)
    }

    fn get_seq_num_min(
        &self,
    ) -> Option<<Self::PSM as rust_rtps_pim::structure::Types>::SequenceNumber> {
        self.changes.iter().map(|x| x.sequence_number).min()
    }

    fn get_seq_num_max(
        &self,
    ) -> Option<<Self::PSM as rust_rtps_pim::structure::Types>::SequenceNumber> {
        self.changes.iter().map(|x| x.sequence_number).max()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn get_seq_num_max() {
        let mut hc = HistoryCacheImpl::new();
        assert_eq!(hc.get_seq_num_max(), None);

        hc.add_change(RTPSCacheChange {
            kind: <RtpsUdpPsm as rust_rtps_pim::structure::Types>::ALIVE,
            writer_guid: [1; 16].into(),
            instance_handle: 1,
            sequence_number: 5.into(),
            data_value: vec![],
            inline_qos: vec![],
        });
        assert_eq!(hc.get_seq_num_max().unwrap(), 5.into());

        hc.add_change(RTPSCacheChange {
            kind: <RtpsUdpPsm as rust_rtps_pim::structure::Types>::ALIVE,
            writer_guid: [1; 16].into(),
            instance_handle: 1,
            sequence_number: 3.into(),
            data_value: vec![],
            inline_qos: vec![],
        });
        assert_eq!(hc.get_seq_num_max().unwrap(), 5.into());
    }
}