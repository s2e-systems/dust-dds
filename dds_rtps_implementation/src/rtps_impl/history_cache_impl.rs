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

#[cfg(test)]
mod tests {
    use super::*;
    use rust_rtps_pim::structure::types::{ChangeKind, GUID};
    use rust_rtps_udp_psm::types::EntityId;

    #[test]
    fn get_seq_num_max() {
        let mut hc = HistoryCacheImpl::new();
        assert_eq!(hc.get_seq_num_max(), None);
        let guid = GUID::new(
            [1; 12],
            EntityId {
                entity_key: [1; 3],
                entity_kind: 1,
            },
        );

        hc.add_change(RTPSCacheChange {
            kind: ChangeKind::Alive,
            writer_guid: guid,
            instance_handle: 1,
            sequence_number: 5.into(),
            data_value: vec![],
            inline_qos: vec![],
        });
        assert_eq!(hc.get_seq_num_max().unwrap(), 5.into());

        hc.add_change(RTPSCacheChange {
            kind: ChangeKind::Alive,
            writer_guid: guid,
            instance_handle: 1,
            sequence_number: 3.into(),
            data_value: vec![],
            inline_qos: vec![],
        });
        assert_eq!(hc.get_seq_num_max().unwrap(), 5.into());
    }

    #[test]
    fn get_seq_num_min() {
        let mut hc = HistoryCacheImpl::new();
        assert_eq!(hc.get_seq_num_min(), None);
        let guid = GUID::new(
            [1; 12],
            EntityId {
                entity_key: [1; 3],
                entity_kind: 1,
            },
        );

        hc.add_change(RTPSCacheChange {
            kind: ChangeKind::Alive,
            writer_guid: guid,
            instance_handle: 1,
            sequence_number: 5.into(),
            data_value: vec![],
            inline_qos: vec![],
        });
        assert_eq!(hc.get_seq_num_max().unwrap(), 5.into());

        hc.add_change(RTPSCacheChange {
            kind: ChangeKind::Alive,
            writer_guid: guid,
            instance_handle: 1,
            sequence_number: 3.into(),
            data_value: vec![],
            inline_qos: vec![],
        });
        assert_eq!(hc.get_seq_num_min().unwrap(), 3.into());
    }
}
