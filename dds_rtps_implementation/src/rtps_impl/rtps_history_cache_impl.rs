use rust_dds_api::dcps_psm::{InstanceStateKind, SampleStateKind, ViewStateKind};
use rust_rtps_pim::{
    messages::types::Time,
    structure::{
        types::{ChangeKind, Guid, InstanceHandle, SequenceNumber},
        RtpsCacheChange, RtpsHistoryCache,
    },
};

pub struct CacheChange {
    kind: ChangeKind,
    writer_guid: Guid,
    sequence_number: SequenceNumber,
    instance_handle: InstanceHandle,
    data: Vec<u8>,
    source_timestamp: Option<Time>,
    creation_timestamp: Time,
    sample_state_kind: SampleStateKind,
    view_state_kind: ViewStateKind,
    instance_state_kind: InstanceStateKind,
}

impl CacheChange {
    /// Get a reference to the cache change's kind.
    pub fn kind(&self) -> &ChangeKind {
        &self.kind
    }

    /// Get a reference to the cache change's writer guid.
    pub fn writer_guid(&self) -> &Guid {
        &self.writer_guid
    }

    /// Get a reference to the cache change's sequence number.
    pub fn sequence_number(&self) -> &SequenceNumber {
        &self.sequence_number
    }

    /// Get a reference to the cache change's instance handle.
    pub fn instance_handle(&self) -> &InstanceHandle {
        &self.instance_handle
    }

    /// Get a reference to the cache change's data.
    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    /// Get a reference to the cache change's source timestamp.
    pub fn source_timestamp(&self) -> Option<&Time> {
        self.source_timestamp.as_ref()
    }

    /// Get a reference to the cache change's creation timestamp.
    pub fn creation_timestamp(&self) -> &Time {
        &self.creation_timestamp
    }

    /// Get a reference to the cache change's sample state kind.
    pub fn sample_state_kind(&self) -> &SampleStateKind {
        &self.sample_state_kind
    }

    /// Get a reference to the cache change's view state kind.
    pub fn view_state_kind(&self) -> &ViewStateKind {
        &self.view_state_kind
    }

    /// Get a reference to the cache change's instance state kind.
    pub fn instance_state_kind(&self) -> &InstanceStateKind {
        &self.instance_state_kind
    }

    /// Mark the cache change as read
    pub fn mark_read(&mut self) {
        self.sample_state_kind = SampleStateKind::Read;
    }
}

pub struct HistoryCache {
    changes: Vec<CacheChange>,
    source_timestamp: Option<Time>,
}

impl HistoryCache {
    /// Set the Rtps history cache impl's info.
    pub fn set_source_timestamp(&mut self, info: Option<Time>) {
        self.source_timestamp = info;
    }

    /// Get a reference to the rtps history cache impl's changes.
    pub fn changes(&self) -> &[CacheChange] {
        self.changes.as_slice()
    }

    /// Get a mutable reference to the rtps history cache impl's changes.
    pub fn changes_mut(&mut self) -> &mut Vec<CacheChange> {
        &mut self.changes
    }
}

impl RtpsHistoryCache for HistoryCache {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            changes: Vec::new(),
            source_timestamp: None,
        }
    }

    fn add_change(&mut self, change: &RtpsCacheChange) {
        let instance_state_kind = match change.kind() {
            ChangeKind::Alive => InstanceStateKind::Alive,
            ChangeKind::AliveFiltered => InstanceStateKind::Alive,
            ChangeKind::NotAliveDisposed => InstanceStateKind::NotAliveDisposed,
            ChangeKind::NotAliveUnregistered => todo!(),
        };

        let current_time = std::time::SystemTime::now();
        let creation_timestamp = Time(
            current_time
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );

        let local_change = CacheChange {
            kind: *change.kind(),
            writer_guid: *change.writer_guid(),
            sequence_number: *change.sequence_number(),
            instance_handle: *change.instance_handle(),
            data: change.data_value().iter().cloned().collect(),
            source_timestamp: self.source_timestamp,
            creation_timestamp,
            sample_state_kind: SampleStateKind::NotRead,
            view_state_kind: ViewStateKind::New,
            instance_state_kind,
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
        let mut hc: HistoryCache = HistoryCache::new();
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
        let mut hc: HistoryCache = HistoryCache::new();
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
        let mut hc: HistoryCache = HistoryCache::new();
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
        let mut hc: HistoryCache = HistoryCache::new();
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
        let mut hc: HistoryCache = HistoryCache::new();
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
