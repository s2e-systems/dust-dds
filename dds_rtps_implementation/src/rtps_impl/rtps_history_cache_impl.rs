use rust_rtps_pim::{messages::types::ParameterIdType, structure::{RTPSCacheChange, RTPSHistoryCache, types::{
            DataType, EntityIdType, GUIDType, GuidPrefixType, InstanceHandleType,
            ParameterListType, SequenceNumberType,
        }}};

use super::rtps_cache_change_impl::RTPSCacheChangeImpl;

pub trait RTPSHistoryCacheImplTrait:
    InstanceHandleType
    + SequenceNumberType
    + DataType
    + ParameterIdType
    + EntityIdType
    + GuidPrefixType
    + GUIDType<Self>
    + ParameterListType<Self>
    + Sized
{
}
impl<
        T: InstanceHandleType
            + SequenceNumberType
            + DataType
            + ParameterIdType
            + EntityIdType
            + GuidPrefixType
            + GUIDType<Self>
            + ParameterListType<Self>
            + Sized,
    > RTPSHistoryCacheImplTrait for T
{
}

pub struct RTPSHistoryCacheImpl<PSM: RTPSHistoryCacheImplTrait> {
    changes: Vec<RTPSCacheChangeImpl<PSM>>,
}

impl<PSM: RTPSHistoryCacheImplTrait> RTPSHistoryCache<PSM> for RTPSHistoryCacheImpl<PSM> {
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

    fn remove_change(&mut self, seq_num: &PSM::SequenceNumber) {
        self.changes.retain(|cc| cc.sequence_number() != seq_num)
    }

    fn get_change(&self, seq_num: &PSM::SequenceNumber) -> Option<&Self::CacheChange> {
        self.changes.iter().find(|&cc| cc.sequence_number() == seq_num)
    }

    fn get_seq_num_min(&self) -> Option<&PSM::SequenceNumber> {
        self.changes.iter().map(|cc| cc.sequence_number()).min()
    }

    fn get_seq_num_max(&self) -> Option<&PSM::SequenceNumber> {
        self.changes.iter().map(|cc| cc.sequence_number()).max()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct MockPSM;

    impl rust_rtps_pim::structure::types::InstanceHandleType for MockPSM {
        type InstanceHandle = ();
    }

    impl rust_rtps_pim::structure::types::SequenceNumberType for MockPSM {
        type SequenceNumber = i64;
        const SEQUENCE_NUMBER_UNKNOWN: Self::SequenceNumber = -1;
    }

    impl rust_rtps_pim::structure::types::DataType for MockPSM {
        type Data = ();
    }

    impl rust_rtps_pim::structure::types::EntityIdType for MockPSM {
        type EntityId = [u8; 4];

        const ENTITYID_UNKNOWN: Self::EntityId = [0; 4];
        const ENTITYID_PARTICIPANT: Self::EntityId = [1; 4];
    }

    impl rust_rtps_pim::messages::types::ParameterIdType for MockPSM {
        type ParameterId = u16;
    }

    impl rust_rtps_pim::structure::types::GuidPrefixType for MockPSM {
        type GuidPrefix = [u8; 12];
        const GUIDPREFIX_UNKNOWN: Self::GuidPrefix = [0; 12];
    }

    #[derive(Clone, Copy)]
    struct MockGUID;

    impl rust_rtps_pim::structure::types::GUID<MockPSM> for MockGUID {
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

    impl rust_rtps_pim::structure::types::GUIDType<MockPSM> for MockPSM {
        type GUID = MockGUID;
        const GUID_UNKNOWN: Self::GUID = MockGUID;
    }

    impl rust_rtps_pim::structure::types::ParameterListType<MockPSM> for MockPSM {
        type ParameterList = MockParameterList;
    }

    pub struct MockParameterList;

    impl rust_rtps_pim::messages::submessage_elements::ParameterList<MockPSM> for MockParameterList {
        type Parameter = MockParameter;

        fn parameter(&self) -> &[Self::Parameter] {
            todo!()
        }
    }

    pub struct MockParameter;
    impl rust_rtps_pim::messages::submessage_elements::Parameter<MockPSM> for MockParameter {
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
            (),
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
            (),
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
            (),
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
            (),
            MockParameterList,
        );
        let change2 = RTPSCacheChangeImpl::new(
            rust_rtps_pim::structure::types::ChangeKind::Alive,
            MockGUID,
            (),
            2,
            (),
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
            (),
            MockParameterList,
        );
        let change2 = RTPSCacheChangeImpl::new(
            rust_rtps_pim::structure::types::ChangeKind::Alive,
            MockGUID,
            (),
            2,
            (),
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
