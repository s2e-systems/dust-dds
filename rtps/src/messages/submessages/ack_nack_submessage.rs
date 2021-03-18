use crate::types::ProtocolVersion;

use super::{
    submessage_elements::{self},
    Serialize,
};
use super::{Submessage, SubmessageHeader};
use super::{SubmessageFlag, SubmessageKind};

#[derive(PartialEq, Debug)]
pub struct AckNack {
    pub endianness_flag: SubmessageFlag,
    pub final_flag: SubmessageFlag,
    pub reader_id: submessage_elements::EntityId,
    pub writer_id: submessage_elements::EntityId,
    pub reader_sn_state: submessage_elements::SequenceNumberSet,
    pub count: submessage_elements::Count,
}

impl Submessage for AckNack {
    fn submessage_header(&self, octets_to_next_header: u16) -> SubmessageHeader {
        let submessage_id = SubmessageKind::AckNack;

        const X: SubmessageFlag = false;
        let e = self.endianness_flag;
        let f = self.final_flag;
        let flags = [e, f, X, X, X, X, X, X];

        SubmessageHeader::new(submessage_id, flags, octets_to_next_header)
    }

    fn is_valid(&self) -> bool {
        todo!()
        // self.reader_sn_state.is_valid()
    }
}

impl Serialize for AckNack {
    fn serialize(&self, buf: &mut [u8], protocol_version: ProtocolVersion) -> Result<usize, ()> {
        SubmessageKind::AckNack.serialize(&mut [buf[0]], protocol_version)?;


        self.count.serialize(&mut buf[10..14], protocol_version)

    }
}