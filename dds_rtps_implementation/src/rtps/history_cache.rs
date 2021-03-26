use rust_rtps_pim::{
    messages,
    structure::{RTPSCacheChange, RTPSHistoryCache},
    types,
};

pub struct HistoryCache<
    GuidPrefix: types::GuidPrefix,
    EntityId: types::EntityId,
    InstanceHandle: types::InstanceHandle,
    SequenceNumber: types::SequenceNumber,
    Data,
    ParameterId: messages::types::ParameterId,
    ParameterValue: AsRef<[u8]> + Clone,
    ParameterList: IntoIterator<
            Item = messages::submessages::submessage_elements::Parameter<
                ParameterId,
                ParameterValue,
            >,
        > + Clone,
> {
    changes: Vec<
        RTPSCacheChange<
            GuidPrefix,
            EntityId,
            InstanceHandle,
            SequenceNumber,
            Data,
            ParameterId,
            ParameterValue,
            ParameterList,
        >,
    >,
}

impl<
        GuidPrefix: types::GuidPrefix,
        EntityId: types::EntityId,
        InstanceHandle: types::InstanceHandle,
        SequenceNumber: types::SequenceNumber,
        Data,
        ParameterId: messages::types::ParameterId,
        ParameterValue: AsRef<[u8]> + Clone,
        ParameterList: IntoIterator<
                Item = messages::submessages::submessage_elements::Parameter<
                    ParameterId,
                    ParameterValue,
                >,
            > + Clone,
    > RTPSHistoryCache
    for HistoryCache<
        GuidPrefix,
        EntityId,
        InstanceHandle,
        SequenceNumber,
        Data,
        ParameterId,
        ParameterValue,
        ParameterList,
    >
{
    type GuidPrefix = GuidPrefix;
    type EntityId = EntityId;
    type InstanceHandle = InstanceHandle;
    type SequenceNumber = SequenceNumber;
    type Data = Data;
    type ParameterId = ParameterId;
    type ParameterValue = ParameterValue;
    type ParameterList = ParameterList;

    fn new() -> Self {
        Self {
            changes: Vec::new(),
        }
    }

    fn add_change(
        &mut self,
        change: RTPSCacheChange<
            Self::GuidPrefix,
            Self::EntityId,
            Self::InstanceHandle,
            Self::SequenceNumber,
            Self::Data,
            Self::ParameterId,
            Self::ParameterValue,
            Self::ParameterList,
        >,
    ) {
        self.changes.push(change)
    }

    fn remove_change(&mut self, seq_num: &Self::SequenceNumber) {
        self.changes.retain(|x| &x.sequence_number != seq_num)
    }

    fn get_change(
        &self,
        seq_num: &Self::SequenceNumber,
    ) -> Option<
        &RTPSCacheChange<
            Self::GuidPrefix,
            Self::EntityId,
            Self::InstanceHandle,
            Self::SequenceNumber,
            Self::Data,
            Self::ParameterId,
            Self::ParameterValue,
            Self::ParameterList,
        >,
    > {
        self.changes.iter().find(|x| &x.sequence_number == seq_num)
    }

    fn get_seq_num_min(&self) -> Option<&Self::SequenceNumber> {
        self.changes.iter().map(|x| &x.sequence_number).min()
    }

    fn get_seq_num_max(&self) -> Option<&Self::SequenceNumber> {
        self.changes.iter().map(|x| &x.sequence_number).max()
    }
}

// impl<T: RTPSCacheChange> RTPSHistoryCache for HistoryCache<T> {
//     type CacheChangeType = T;

//     fn new() -> Self {
//         Self {
//             changes: Vec::new(),
//         }
//     }

//     fn add_change(&mut self, change: Self::CacheChangeType) {
//         self.changes.push(change)
//     }

//     fn remove_change(&mut self, seq_num: SequenceNumber) {
//         self.changes.retain(|cc| cc.sequence_number() != seq_num)
//     }

//     fn get_change(&self, seq_num: SequenceNumber) -> Option<&Self::CacheChangeType> {
//         self.changes
//             .iter()
//             .find(|cc| cc.sequence_number() == seq_num)
//     }

//     fn get_seq_num_min(&self) -> Option<SequenceNumber> {
//         self.changes.iter().map(|cc| cc.sequence_number()).min()
//     }

//     fn get_seq_num_max(&self) -> Option<SequenceNumber> {
//         self.changes.iter().map(|cc| cc.sequence_number()).max()
//     }
// }

// #[cfg(test)]
// mod tests {
//     use rust_rtps::{
//         messages::submessages::submessage_elements::ParameterList,
//         types::{ChangeKind, GUID},
//     };

//     use super::*;

//     struct MockParameterList;

//     impl ParameterList for MockParameterList {
//         fn parameter(
//             &self,
//         ) -> &[Box<dyn rust_rtps::messages::submessages::submessage_elements::Parameter>] {
//             todo!()
//         }
//     }

//     #[derive(Debug, Clone, PartialEq)]
//     struct MockCacheChange {
//         sequence_number: SequenceNumber,
//     }

//     impl RTPSCacheChange for MockCacheChange {
//         type Data = ();
//         type InstanceHandle = ();
//         type ParameterList = MockParameterList;

//         fn new(
//             _kind: ChangeKind,
//             _writer_guid: GUID,
//             _instance_handle: Self::InstanceHandle,
//             _sequence_number: SequenceNumber,
//             _data_value: Self::Data,
//             _inline_qos: Self::ParameterList,
//         ) -> Self {
//             todo!()
//         }

//         fn kind(&self) -> ChangeKind {
//             todo!()
//         }

//         fn writer_guid(&self) -> GUID {
//             todo!()
//         }

//         fn instance_handle(&self) -> &Self::InstanceHandle {
//             todo!()
//         }

//         fn sequence_number(&self) -> SequenceNumber {
//             self.sequence_number
//         }

//         fn data_value(&self) -> &Self::Data {
//             todo!()
//         }

//         fn inline_qos(&self) -> &Self::ParameterList {
//             todo!()
//         }
//     }
//     #[test]
//     fn add_and_get_change() {
//         let mut history_cache = HistoryCache::new();
//         let cc1 = MockCacheChange {
//             sequence_number: 1.into(),
//         };
//         let cc2 = MockCacheChange {
//             sequence_number: 2.into(),
//         };
//         history_cache.add_change(cc1.clone());
//         history_cache.add_change(cc2.clone());

//         assert_eq!(*history_cache.get_change(1.into()).unwrap(), cc1);
//         assert_eq!(*history_cache.get_change(2.into()).unwrap(), cc2);
//         assert!(history_cache.get_change(3.into()).is_none());
//     }

//     #[test]
//     fn remove_change() {
//         let mut history_cache = HistoryCache::new();
//         let cc1 = MockCacheChange {
//             sequence_number: 1.into(),
//         };
//         let cc2 = MockCacheChange {
//             sequence_number: 2.into(),
//         };
//         history_cache.add_change(cc1.clone());
//         history_cache.add_change(cc2.clone());
//         history_cache.remove_change(1.into());

//         assert!(history_cache.get_change(1.into()).is_none());
//         assert_eq!(*history_cache.get_change(2.into()).unwrap(), cc2);
//     }

//     #[test]
//     fn get_seq_num_min_and_max() {
//         let mut history_cache = HistoryCache::new();
//         let min_seq_num = 4.into();
//         let max_seq_num = 6.into();
//         let cc_min = MockCacheChange {
//             sequence_number: min_seq_num,
//         };
//         let cc_max = MockCacheChange {
//             sequence_number: max_seq_num,
//         };
//         history_cache.add_change(cc_min);
//         history_cache.add_change(cc_max);

//         assert_eq!(history_cache.get_seq_num_max(), Some(max_seq_num));
//         assert_eq!(history_cache.get_seq_num_min(), Some(min_seq_num));
//     }
// }
