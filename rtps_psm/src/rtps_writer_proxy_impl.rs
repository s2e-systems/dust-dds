use std::ops::{Deref, DerefMut};

use rust_rtps_pim::{
    behavior::reader::writer_proxy::{RtpsWriterProxy, RtpsWriterProxyOperations},
    structure::types::{Locator, SequenceNumber},
};

pub struct RtpsWriterProxyImpl(RtpsWriterProxy<Vec<Locator>>);

impl RtpsWriterProxyImpl {
    pub fn new(writer_proxy: RtpsWriterProxy<Vec<Locator>>) -> Self {
        Self(writer_proxy)
    }
}

impl Deref for RtpsWriterProxyImpl {
    type Target = RtpsWriterProxy<Vec<Locator>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RtpsWriterProxyImpl {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl RtpsWriterProxyOperations for RtpsWriterProxyImpl {
    type SequenceNumberVector = Vec<SequenceNumber>;

    fn available_changes_max(&self) -> &rust_rtps_pim::structure::types::SequenceNumber {
        todo!()
    }

    fn irrelevant_change_set(
        &mut self,
        _a_seq_num: &rust_rtps_pim::structure::types::SequenceNumber,
    ) {
        todo!()
    }

    fn lost_changes_update(
        &mut self,
        _first_available_seq_num: &rust_rtps_pim::structure::types::SequenceNumber,
    ) {
        todo!()
    }

    fn missing_changes(&self) -> Self::SequenceNumberVector {
        todo!()
    }

    fn missing_changes_update(
        &mut self,
        _last_available_seq_num: rust_rtps_pim::structure::types::SequenceNumber,
    ) {
        todo!()
    }

    fn received_change_set(&mut self, _a_seq_num: rust_rtps_pim::structure::types::SequenceNumber) {
        todo!()
    }
}
