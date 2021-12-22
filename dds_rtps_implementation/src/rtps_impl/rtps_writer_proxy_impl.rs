use rust_rtps_pim::{
    behavior::reader::writer_proxy::{
        RtpsWriterProxy, RtpsWriterProxyAttributes, RtpsWriterProxyOperations,
    },
    structure::types::{Locator, SequenceNumber},
};

pub struct RtpsWriterProxyImpl(RtpsWriterProxy<Vec<Locator>>);

impl RtpsWriterProxyImpl {
    pub fn new(writer_proxy: RtpsWriterProxy<Vec<Locator>>) -> Self {
        Self(writer_proxy)
    }
}

impl RtpsWriterProxyAttributes for RtpsWriterProxyImpl {
    fn remote_writer_guid(&self) -> &rust_rtps_pim::structure::types::Guid {
        &self.0.remote_writer_guid
    }
}

impl RtpsWriterProxyOperations for RtpsWriterProxyImpl {
    type SequenceNumberVector = Vec<SequenceNumber>;

    fn available_changes_max(&self) -> &SequenceNumber {
        todo!()
    }

    fn irrelevant_change_set(
        &mut self,
        _a_seq_num: &SequenceNumber,
    ) {
        todo!()
    }

    fn lost_changes_update(
        &mut self,
        _first_available_seq_num: &SequenceNumber,
    ) {
        todo!()
    }

    fn missing_changes(&self) -> Self::SequenceNumberVector {
        todo!()
    }

    fn missing_changes_update(
        &mut self,
        _last_available_seq_num: &SequenceNumber,
    ) {
        todo!()
    }

    fn received_change_set(&mut self, _a_seq_num: &SequenceNumber) {
        todo!()
    }
}
