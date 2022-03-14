use rust_rtps_pim::{
    behavior::{
        types::ChangeForReaderStatusKind,
        writer::change_for_reader::{
            RtpsChangeForReaderAttributes,
        },
    },
    structure::types::SequenceNumber,
};

#[derive(Debug, PartialEq)]
pub struct RtpsChangeForReaderImpl {
    pub status: ChangeForReaderStatusKind,
    pub is_relevant: bool,
    pub sequence_number: SequenceNumber,
}

impl RtpsChangeForReaderAttributes for RtpsChangeForReaderImpl {
    fn status(&self) -> ChangeForReaderStatusKind {
        self.status
    }

    fn is_relevant(&self) -> bool {
        self.is_relevant
    }
}
