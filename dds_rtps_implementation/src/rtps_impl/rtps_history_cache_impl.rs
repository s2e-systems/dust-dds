use rust_rtps_pim::structure::{RTPSCacheChange, RTPSHistoryCache};

pub struct RTPSHistoryCacheImpl<PSM> {}

impl<PSM> RTPSHistoryCache<PSM> for RTPSHistoryCacheImpl<PSM> {
    type CacheChange;

    fn new() -> Self
    where
        Self: Sized,
    {
        todo!()
    }

    fn add_change(&mut self, change: Self::CacheChange) {
        todo!()
    }

    fn remove_change(&mut self, seq_num: &PSM::SequenceNumber) {
        todo!()
    }

    fn get_change(&self, seq_num: &PSM::SequenceNumber) -> Option<&Self::CacheChange> {
        todo!()
    }

    fn get_seq_num_min(&self) -> Option<PSM::SequenceNumber> {
        todo!()
    }

    fn get_seq_num_max(&self) -> Option<PSM::SequenceNumber> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use rust_rtps_pim::structure::types::{ChangeKind, GUID};
    // use rust_rtps_udp_psm::types::EntityId;
    // use rust_rtps_udp_psm::RtpsUdpPsm;

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
