use rust_rtps::{
    behavior::{
        stateful_writer::reader_proxy::RTPSChangeForReader, types::ChangeForReaderStatusKind,
    },
    types::SequenceNumber,
};
pub struct ChangeForReader {
    change: SequenceNumber,
    status: ChangeForReaderStatusKind,
    is_relevant: bool,
}

impl RTPSChangeForReader for ChangeForReader {
    type CacheChangeRepresentation = SequenceNumber;

    fn new(
        change: Self::CacheChangeRepresentation,
        status: ChangeForReaderStatusKind,
        is_relevant: bool,
    ) -> Self {
        Self {
            change,
            status,
            is_relevant,
        }
    }

    fn change(&self) -> Self::CacheChangeRepresentation {
        self.change
    }

    fn status(&self) -> ChangeForReaderStatusKind {
        self.status
    }

    fn is_relevant(&self) -> bool {
        self.is_relevant
    }
}
