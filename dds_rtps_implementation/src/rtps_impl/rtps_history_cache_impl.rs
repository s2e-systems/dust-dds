use rust_rtps_pim::{
    messages::submessage_elements::ParameterListSubmessageElementPIM,
    structure::{
        types::{DataPIM, InstanceHandlePIM, SequenceNumberPIM, GUIDPIM},
        RTPSCacheChange, RTPSHistoryCache,
    },
};

use super::rtps_cache_change_impl::RTPSCacheChangeImpl;

pub struct RTPSHistoryCacheImpl<PSM>
where
    PSM: GUIDPIM<PSM>
        + InstanceHandlePIM
        + SequenceNumberPIM
        + DataPIM
        + ParameterListSubmessageElementPIM<PSM>,
{
    changes: Vec<RTPSCacheChangeImpl<PSM>>,
}

impl<PSM> RTPSHistoryCache<PSM> for RTPSHistoryCacheImpl<PSM>
where
    PSM: GUIDPIM<PSM>
        + InstanceHandlePIM
        + SequenceNumberPIM
        + DataPIM
        + ParameterListSubmessageElementPIM<PSM>,
    PSM::SequenceNumberType: PartialEq + Ord,
{
    type CacheChange = RTPSCacheChangeImpl<PSM>;

    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            changes: Vec::new(),
        }
    }

    fn add_change(&mut self, change: Self::CacheChange) {
        self.changes.push(change)
    }

    fn remove_change(&mut self, seq_num: &PSM::SequenceNumberType) {
        self.changes.retain(|cc| cc.sequence_number() != seq_num)
    }

    fn get_change(&self, seq_num: &PSM::SequenceNumberType) -> Option<&Self::CacheChange> {
        self.changes
            .iter()
            .find(|&cc| cc.sequence_number() == seq_num)
    }

    fn get_seq_num_min(&self) -> Option<&PSM::SequenceNumberType> {
        self.changes.iter().map(|cc| cc.sequence_number()).min()
    }

    fn get_seq_num_max(&self) -> Option<&PSM::SequenceNumberType> {
        self.changes.iter().map(|cc| cc.sequence_number()).max()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct MockPSM;

    impl rust_rtps_pim::structure::types::InstanceHandlePIM for MockPSM {
        type InstanceHandleType = ();
    }

    impl rust_rtps_pim::structure::types::SequenceNumberPIM for MockPSM {
        type SequenceNumberType = i64;
        const SEQUENCE_NUMBER_UNKNOWN: Self::SequenceNumberType = -1;
    }

    impl rust_rtps_pim::structure::types::DataPIM for MockPSM {
        type DataType = [u8; 0];
    }

    impl rust_rtps_pim::structure::types::EntityIdPIM for MockPSM {
        type EntityIdType = [u8; 4];

        const ENTITYID_UNKNOWN: Self::EntityIdType = [0; 4];
        const ENTITYID_PARTICIPANT: Self::EntityIdType = [1; 4];
    }

    impl rust_rtps_pim::messages::types::ParameterIdPIM for MockPSM {
        type ParameterIdType = u16;
    }

    impl rust_rtps_pim::structure::types::GuidPrefixPIM for MockPSM {
        type GuidPrefixType = [u8; 12];
        const GUIDPREFIX_UNKNOWN: Self::GuidPrefixType = [0; 12];
    }

    #[derive(Clone, Copy, PartialEq)]
    struct MockGUID;

    impl rust_rtps_pim::structure::types::GUIDType<MockPSM> for MockGUID {
        fn new(_prefix: [u8; 12], _entity_id: [u8; 4]) -> Self {
            todo!()
        }

        fn prefix(&self) -> &[u8; 12] {
            todo!()
        }

        fn entity_id(&self) -> &[u8; 4] {
            todo!()
        }
    }

    impl From<[u8; 16]> for MockGUID {
        fn from(_: [u8; 16]) -> Self {
            todo!()
        }
    }

    impl Into<[u8; 16]> for MockGUID {
        fn into(self) -> [u8; 16] {
            todo!()
        }
    }

    impl rust_rtps_pim::structure::types::GUIDPIM<Self> for MockPSM {
        type GUIDType = MockGUID;
        const GUID_UNKNOWN: Self::GUIDType = MockGUID;
    }

    impl rust_rtps_pim::messages::submessage_elements::ParameterListSubmessageElementPIM<MockPSM>
        for MockPSM
    {
        type ParameterListSubmessageElementType = MockParameterList;
    }

    pub struct MockParameterList;

    impl rust_rtps_pim::messages::submessage_elements::ParameterListSubmessageElementType<MockPSM>
        for MockParameterList
    {
        type Parameter = MockParameter;

        fn new(_parameter: &[MockParameter]) -> Self {
            todo!()
        }

        fn parameter(&self) -> &[MockParameter] {
            todo!()
        }
    }

    pub struct MockParameter;
    impl rust_rtps_pim::messages::submessage_elements::ParameterType<MockPSM> for MockParameter {
        fn parameter_id(&self) -> u16 {
            todo!()
        }

        fn length(&self) -> i16 {
            todo!()
        }

        fn value(&self) -> &[u8] {
            todo!()
        }
    }

    #[test]
    fn add_change() {
        let mut hc: RTPSHistoryCacheImpl<MockPSM> = RTPSHistoryCacheImpl::new();
        let change = RTPSCacheChangeImpl::new(
            rust_rtps_pim::structure::types::ChangeKind::Alive,
            MockGUID,
            (),
            1,
            [],
            MockParameterList,
        );
        hc.add_change(change);
        assert!(hc.get_change(&1).is_some());
    }

    #[test]
    fn remove_change() {
        let mut hc: RTPSHistoryCacheImpl<MockPSM> = RTPSHistoryCacheImpl::new();
        let change = RTPSCacheChangeImpl::new(
            rust_rtps_pim::structure::types::ChangeKind::Alive,
            MockGUID,
            (),
            1,
            [],
            MockParameterList,
        );
        hc.add_change(change);
        hc.remove_change(&1);
        assert!(hc.get_change(&1).is_none());
    }

    #[test]
    fn get_change() {
        let mut hc: RTPSHistoryCacheImpl<MockPSM> = RTPSHistoryCacheImpl::new();
        let change = RTPSCacheChangeImpl::new(
            rust_rtps_pim::structure::types::ChangeKind::Alive,
            MockGUID,
            (),
            1,
            [],
            MockParameterList,
        );
        hc.add_change(change);
        assert!(hc.get_change(&1).is_some());
        assert!(hc.get_change(&2).is_none());
    }

    #[test]
    fn get_seq_num_min() {
        let mut hc: RTPSHistoryCacheImpl<MockPSM> = RTPSHistoryCacheImpl::new();
        let change1 = RTPSCacheChangeImpl::new(
            rust_rtps_pim::structure::types::ChangeKind::Alive,
            MockGUID,
            (),
            1,
            [],
            MockParameterList,
        );
        let change2 = RTPSCacheChangeImpl::new(
            rust_rtps_pim::structure::types::ChangeKind::Alive,
            MockGUID,
            (),
            2,
            [],
            MockParameterList,
        );
        hc.add_change(change1);
        hc.add_change(change2);
        assert_eq!(hc.get_seq_num_min(), Some(&1));
    }

    #[test]
    fn get_seq_num_max() {
        let mut hc: RTPSHistoryCacheImpl<MockPSM> = RTPSHistoryCacheImpl::new();
        let change1 = RTPSCacheChangeImpl::new(
            rust_rtps_pim::structure::types::ChangeKind::Alive,
            MockGUID,
            (),
            1,
            [],
            MockParameterList,
        );
        let change2 = RTPSCacheChangeImpl::new(
            rust_rtps_pim::structure::types::ChangeKind::Alive,
            MockGUID,
            (),
            2,
            [],
            MockParameterList,
        );
        hc.add_change(change1);
        hc.add_change(change2);
        assert_eq!(hc.get_seq_num_max(), Some(&2));
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
