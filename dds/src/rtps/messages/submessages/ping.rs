use std::io::Write;

use crate::{
    rtps::{
        error::RtpsResult,
        messages::{
            overall_structure::{
                Submessage, SubmessageHeaderRead, SubmessageHeaderWrite, TryReadFromBytes,
                WriteIntoBytes,
            },
            types::SubmessageKind,
        },
    },
    transport::types::SequenceNumber,
};

#[derive(Debug, PartialEq, Eq)]
pub struct PingSubmessage {
    sequence_number: SequenceNumber,
}

impl PingSubmessage {
    pub fn new(sequence_number: SequenceNumber) -> Self {
        Self { sequence_number }
    }

    pub fn try_from_bytes(
        submessage_header: &SubmessageHeaderRead,
        mut data: &[u8],
    ) -> RtpsResult<Self> {
        let endianness = submessage_header.endianness();
        Ok(Self {
            sequence_number: SequenceNumber::try_read_from_bytes(&mut data, endianness)?,
        })
    }

    pub fn sequence_number(&self) -> i64 {
        self.sequence_number
    }
}

impl Submessage for PingSubmessage {
    fn write_submessage_header_into_bytes(&self, octets_to_next_header: u16, buf: &mut dyn Write) {
        SubmessageHeaderWrite::new(SubmessageKind::PING, &[], octets_to_next_header)
            .write_into_bytes(buf);
    }

    fn write_submessage_elements_into_bytes(&self, buf: &mut dyn Write) {
        self.sequence_number.write_into_bytes(buf);
    }
}
