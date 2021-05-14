use rust_rtps_pim::structure::{RTPSCacheChange, RTPSHistoryCache};

pub struct RTPSHistoryCacheImpl<PSM: rust_rtps_pim::structure::Types> {
    changes: Vec<RTPSCacheChange<PSM>>,
}

impl<PSM: rust_rtps_pim::structure::Types> RTPSHistoryCache<PSM> for RTPSHistoryCacheImpl<PSM> {
    fn new() -> Self {
        Self {
            changes: Vec::new(),
        }
    }

    fn add_change(&mut self, change: rust_rtps_pim::structure::RTPSCacheChange<PSM>) {
        self.changes.push(change)
    }

    fn remove_change(
        &mut self,
        seq_num: &<PSM as rust_rtps_pim::structure::Types>::SequenceNumber,
    ) {
        self.changes.retain(|x| &x.sequence_number != seq_num)
    }

    fn get_change(
        &self,
        seq_num: &<PSM as rust_rtps_pim::structure::Types>::SequenceNumber,
    ) -> Option<&rust_rtps_pim::structure::RTPSCacheChange<PSM>> {
        self.changes.iter().find(|x| &x.sequence_number == seq_num)
    }

    fn get_seq_num_min(&self) -> Option<<PSM as rust_rtps_pim::structure::Types>::SequenceNumber> {
        self.changes.iter().map(|x| x.sequence_number).min()
    }

    fn get_seq_num_max(&self) -> Option<<PSM as rust_rtps_pim::structure::Types>::SequenceNumber> {
        self.changes.iter().map(|x| x.sequence_number).max()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_rtps_pim::structure::types::{ChangeKind, GUID};
    use rust_rtps_udp_psm::types::EntityId;
    use rust_rtps_udp_psm::RtpsUdpPsm;

    #[test]
    fn get_seq_num_max() {
        let mut hc: RTPSHistoryCacheImpl<RtpsUdpPsm> = RTPSHistoryCacheImpl::new();
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
            // inline_qos: vec![],
        });
        assert_eq!(hc.get_seq_num_max().unwrap(), 5.into());

        hc.add_change(RTPSCacheChange {
            kind: ChangeKind::Alive,
            writer_guid: guid,
            instance_handle: 1,
            sequence_number: 3.into(),
            data_value: vec![],
            // inline_qos: vec![],
        });
        assert_eq!(hc.get_seq_num_max().unwrap(), 5.into());
    }

    #[test]
    fn get_seq_num_min() {
        let mut hc: RTPSHistoryCacheImpl<RtpsUdpPsm> = RTPSHistoryCacheImpl::new();
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
            // inline_qos: vec![],
        });
        assert_eq!(hc.get_seq_num_max().unwrap(), 5.into());

        hc.add_change(RTPSCacheChange {
            kind: ChangeKind::Alive,
            writer_guid: guid,
            instance_handle: 1,
            sequence_number: 3.into(),
            data_value: vec![],
            // inline_qos: vec![],
        });
        assert_eq!(hc.get_seq_num_min().unwrap(), 3.into());
    }
}
