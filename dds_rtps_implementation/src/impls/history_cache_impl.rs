use rust_rtps::structure::{CacheChange, HistoryCache};

pub struct HistoryCacheImpl {}

impl HistoryCache for HistoryCacheImpl {
    type CacheChangeType = CacheChangeImpl;

    fn new() -> Self {
        todo!()
    }

    fn add_change(&mut self, change: Self::CacheChangeType) {
        todo!()
    }

    fn remove_change(&mut self, seq_num: rust_rtps::types::SequenceNumber) {
        todo!()
    }

    fn get_change(
        &self,
        seq_num: rust_rtps::types::SequenceNumber,
    ) -> Option<&Self::CacheChangeType> {
        todo!()
    }

    fn get_seq_num_min(&self) -> Option<rust_rtps::types::SequenceNumber> {
        todo!()
    }

    fn get_seq_num_max(&self) -> Option<rust_rtps::types::SequenceNumber> {
        todo!()
    }
}

pub struct CacheChangeImpl {}

impl CacheChange for CacheChangeImpl {
    type Data = Vec<u8>;

    fn new(
        kind: rust_rtps::types::ChangeKind,
        writer_guid: rust_rtps::types::GUID,
        instance_handle: rust_rtps::types::InstanceHandle,
        sequence_number: rust_rtps::types::SequenceNumber,
        data_value: Self::Data,
        inline_qos: rust_rtps::messages::submessages::submessage_elements::ParameterList,
    ) -> Self {
        todo!()
    }

    fn kind(&self) -> rust_rtps::types::ChangeKind {
        todo!()
    }

    fn writer_guid(&self) -> rust_rtps::types::GUID {
        todo!()
    }

    fn instance_handle(&self) -> &rust_rtps::types::InstanceHandle {
        todo!()
    }

    fn sequence_number(&self) -> rust_rtps::types::SequenceNumber {
        todo!()
    }

    fn data_value(&self) -> &Self::Data {
        todo!()
    }

    fn inline_qos(&self) -> &rust_rtps::messages::submessages::submessage_elements::ParameterList {
        todo!()
    }
}
