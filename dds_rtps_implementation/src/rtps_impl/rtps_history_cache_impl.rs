use rust_rtps_pim::{messages::types::Time, structure::{
    types::{ChangeKind, InstanceHandle, SequenceNumber, GUID},
    RtpsHistoryCache, RtpsCacheChange,
}};

struct CacheChangeRepresentation {
    kind: ChangeKind,
    writer_guid: GUID,
    sequence_number: SequenceNumber,
    instance_handle: InstanceHandle,
    data: Vec<u8>,
    info_timestamp: Option<Time>
}

pub struct RtpsHistoryCacheImpl {
    changes: Vec<CacheChangeRepresentation>,
    info: Option<Time>,
}

impl RtpsHistoryCacheImpl {
    /// Set the r t p s history cache impl's info.
    pub fn set_info(&mut self, info: Option<Time>) {
        self.info = info;
    }
}

impl RtpsHistoryCache for RtpsHistoryCacheImpl {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            changes: Vec::new(),
            info: None,
        }
    }

    fn add_change(&mut self, change: &RtpsCacheChange) {
        let local_change = CacheChangeRepresentation {
            kind: *change.kind(),
            writer_guid: *change.writer_guid(),
            sequence_number: *change.sequence_number(),
            instance_handle: *change.instance_handle(),
            data: change.data_value().iter().cloned().collect(),
            info_timestamp: self.info,
        };
        self.changes.push(local_change)
    }

    fn remove_change(&mut self, seq_num: &SequenceNumber) {
        self.changes.retain(|cc| &cc.sequence_number != seq_num)
    }

    fn get_change(&self, seq_num: &SequenceNumber) -> Option<RtpsCacheChange> {
        let local_change = self
            .changes
            .iter()
            .find(|&cc| &cc.sequence_number == seq_num)?;

        Some(RtpsCacheChange::new(
            local_change.kind,
            local_change.writer_guid,
            local_change.instance_handle,
            local_change.sequence_number,
            local_change.data.as_ref(),
            &[],
        ))
    }

    fn get_seq_num_min(&self) -> Option<SequenceNumber> {
        self.changes
            .iter()
            .map(|cc| cc.sequence_number)
            .min()
            .clone()
    }

    fn get_seq_num_max(&self) -> Option<SequenceNumber> {
        self.changes
            .iter()
            .map(|cc| cc.sequence_number)
            .max()
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use rust_rtps_pim::structure::types::GUID_UNKNOWN;

    use super::*;

    #[test]
    fn add_change() {
        let mut hc: RtpsHistoryCacheImpl = RtpsHistoryCacheImpl::new();
        let change = RtpsCacheChange::new(
            rust_rtps_pim::structure::types::ChangeKind::Alive,
            GUID_UNKNOWN,
            0,
            1,
            &[],
            &[],
        );
        hc.add_change(&change);
        assert!(hc.get_change(&1).is_some());
    }

    #[test]
    fn remove_change() {
        let mut hc: RtpsHistoryCacheImpl = RtpsHistoryCacheImpl::new();
        let change = RtpsCacheChange::new(
            rust_rtps_pim::structure::types::ChangeKind::Alive,
            GUID_UNKNOWN,
            0,
            1,
            &[],
            &[],
        );
        hc.add_change(&change);
        hc.remove_change(&1);
        assert!(hc.get_change(&1).is_none());
    }

    #[test]
    fn get_change() {
        let mut hc: RtpsHistoryCacheImpl = RtpsHistoryCacheImpl::new();
        let change = RtpsCacheChange::new(
            rust_rtps_pim::structure::types::ChangeKind::Alive,
            GUID_UNKNOWN,
            0,
            1,
            &[],
            &[],
        );
        hc.add_change(&change);
        assert!(hc.get_change(&1).is_some());
        assert!(hc.get_change(&2).is_none());
    }

    #[test]
    fn get_seq_num_min() {
        let mut hc: RtpsHistoryCacheImpl = RtpsHistoryCacheImpl::new();
        let change1 = RtpsCacheChange::new(
            rust_rtps_pim::structure::types::ChangeKind::Alive,
            GUID_UNKNOWN,
            0,
            1,
            &[],
            &[],
        );
        let change2 = RtpsCacheChange::new(
            rust_rtps_pim::structure::types::ChangeKind::Alive,
            GUID_UNKNOWN,
            0,
            2,
            &[],
            &[],
        );
        hc.add_change(&change1);
        hc.add_change(&change2);
        assert_eq!(hc.get_seq_num_min(), Some(1));
    }

    #[test]
    fn get_seq_num_max() {
        let mut hc: RtpsHistoryCacheImpl = RtpsHistoryCacheImpl::new();
        let change1 = RtpsCacheChange::new(
            rust_rtps_pim::structure::types::ChangeKind::Alive,
            GUID_UNKNOWN,
            0,
            1,
            &[],
            &[],
        );
        let change2 = RtpsCacheChange::new(
            rust_rtps_pim::structure::types::ChangeKind::Alive,
            GUID_UNKNOWN,
            0,
            2,
            &[],
            &[],
        );
        hc.add_change(&change1);
        hc.add_change(&change2);
        assert_eq!(hc.get_seq_num_max(), Some(2));
    }

    // #[test]
    // fn get_seq_num_max() {
    //     let mut hc: RTPSHistoryCacheImpl<RtpsUdpPsm> = RTPSHistoryCacheImpl::new();
    //     assert_eq!(hc.get_seq_num_max(), None);
    //     let guid = GUID::new(
    //         [1; 12],
    //         EntityId {
    //             entity_key: [1; 3],
    //             entity_kind: 1,
    //         },
    //     );

    //     hc.add_change(RTPSCacheChange {
    //         kind: ChangeKind::Alive,
    //         writer_guid: guid,
    //         instance_handle: 1,
    //         sequence_number: 5.into(),
    //         data_value: vec![],
    //         // inline_qos: vec![],
    //     });
    //     assert_eq!(hc.get_seq_num_max().unwrap(), 5.into());

    //     hc.add_change(RTPSCacheChange {
    //         kind: ChangeKind::Alive,
    //         writer_guid: guid,
    //         instance_handle: 1,
    //         sequence_number: 3.into(),
    //         data_value: vec![],
    //         // inline_qos: vec![],
    //     });
    //     assert_eq!(hc.get_seq_num_max().unwrap(), 5.into());
    // }

    // #[test]
    // fn get_seq_num_min() {
    //     let mut hc: RTPSHistoryCacheImpl<RtpsUdpPsm> = RTPSHistoryCacheImpl::new();
    //     assert_eq!(hc.get_seq_num_min(), None);
    //     let guid = GUID::new(
    //         [1; 12],
    //         EntityId {
    //             entity_key: [1; 3],
    //             entity_kind: 1,
    //         },
    //     );

    //     hc.add_change(RTPSCacheChange {
    //         kind: ChangeKind::Alive,
    //         writer_guid: guid,
    //         instance_handle: 1,
    //         sequence_number: 5.into(),
    //         data_value: vec![],
    //         // inline_qos: vec![],
    //     });
    //     assert_eq!(hc.get_seq_num_max().unwrap(), 5.into());

    //     hc.add_change(RTPSCacheChange {
    //         kind: ChangeKind::Alive,
    //         writer_guid: guid,
    //         instance_handle: 1,
    //         sequence_number: 3.into(),
    //         data_value: vec![],
    //         // inline_qos: vec![],
    //     });
    //     assert_eq!(hc.get_seq_num_min().unwrap(), 3.into());
    // }
}
