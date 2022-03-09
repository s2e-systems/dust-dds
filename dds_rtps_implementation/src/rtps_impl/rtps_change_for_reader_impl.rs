use rust_rtps_pim::{
    behavior::{
        types::ChangeForReaderStatusKind,
        writer::change_for_reader::{
            RtpsChangeForReaderAttributes, RtpsChangeForReaderConstructor,
        },
    },
    structure::types::SequenceNumber,
};

struct RtpsCacheChangeImpl {
    status: ChangeForReaderStatusKind,
    is_relevant: bool,
    seq_num: SequenceNumber,
}

impl RtpsChangeForReaderAttributes for RtpsCacheChangeImpl {
    fn status(&self) -> ChangeForReaderStatusKind {
        self.status
    }

    fn is_relevant(&self) -> bool {
        self.is_relevant
    }
}

impl RtpsChangeForReaderConstructor for RtpsCacheChangeImpl {
    fn new(status: ChangeForReaderStatusKind, is_relevant: bool) -> Self {
        Self {
            status,
            is_relevant,
            seq_num: 0,
        }
    }
}
