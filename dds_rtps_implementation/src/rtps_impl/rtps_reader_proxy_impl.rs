use std::ops::{Deref, DerefMut};

use rust_rtps_pim::{
    behavior::writer::reader_proxy::{RtpsReaderProxy, RtpsReaderProxyOperations},
    structure::types::{Locator, SequenceNumber},
};

pub struct RtpsReaderProxyImpl(RtpsReaderProxy<Vec<Locator>>);

impl RtpsReaderProxyImpl {
    pub fn new(reader_proxy: RtpsReaderProxy<Vec<Locator>>) -> Self {
        Self(reader_proxy)
    }
}

impl Deref for RtpsReaderProxyImpl {
    type Target = RtpsReaderProxy<Vec<Locator>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RtpsReaderProxyImpl {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl RtpsReaderProxyOperations for RtpsReaderProxyImpl {
    type SequenceNumberVector = Vec<SequenceNumber>;

    fn acked_changes_set(&mut self, _committed_seq_num: SequenceNumber) {
        todo!()
    }

    fn next_requested_change(&mut self) -> Option<SequenceNumber> {
        todo!()
    }

    fn next_unsent_change(&mut self) -> Option<SequenceNumber> {
        todo!()
    }

    fn unsent_changes(&self) -> Self::SequenceNumberVector {
        todo!()
    }

    fn requested_changes(&self) -> Self::SequenceNumberVector {
        todo!()
    }

    fn requested_changes_set(&mut self, _req_seq_num_set: Self::SequenceNumberVector) {
        todo!()
    }

    fn unacked_changes(&self) -> Self::SequenceNumberVector {
        todo!()
    }
}
