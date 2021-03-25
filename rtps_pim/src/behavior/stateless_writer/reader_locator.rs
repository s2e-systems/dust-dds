use crate::{structure::RTPSCacheChange, types};

pub trait RTPSReaderLocator {
    type CacheChange;
    type CacheChangeList: IntoIterator<Item = Self::CacheChange>;
    type SequenceNumber: types::SequenceNumber;

    fn requested_changes(&self) -> Self::CacheChangeList;
    fn unsent_changes(&self) -> Self::CacheChangeList;
    fn next_requested_change(&mut self) -> Option<Self::CacheChange>;
    fn next_unsent_change(&mut self) -> Option<Self::CacheChange>;
    fn requested_changes_set(&mut self, req_seq_num_set: &[Self::SequenceNumber]);
}

// impl<CC, CCL, SN>
//     dyn RTPSReaderLocator<CacheChange = CC, CacheChangeList = CCL, SequenceNumber = SN>
// where
//     CC: RTPSCacheChange<SequenceNumber = SN>,
//     CCL: IntoIterator<Item = CC>,
//     SN: types::SequenceNumber,
// {
//     fn transition_t4(&mut self) -> Option<BestEffortReaderLocatorSendSubmessages<DataSubmessage, GapSubmessage>>{
//         if let Some(next_unsent_cache_change) = self.next_unsent_change() {

//         } else {

//         }
//     }
// }
