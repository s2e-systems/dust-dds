use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::implementation::rtps::messages::{
    overall_structure::RtpsSubmessageHeader, submessages::PadSubmessageWrite, types::SubmessageKind,
};

use super::submessage::{MappingWriteSubmessage};

impl MappingWriteSubmessage for PadSubmessageWrite {
    fn submessage_header(&self) -> RtpsSubmessageHeader {
        RtpsSubmessageHeader {
            submessage_id: SubmessageKind::PAD,
            flags: [
                self.endianness_flag,
                false,
                false,
                false,
                false,
                false,
                false,
                false,
            ],
            submessage_length: 0,
        }
    }

    fn mapping_write_submessage_elements<W: Write, B: ByteOrder>(
        &self,
        _writer: W,
    ) -> Result<(), Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::implementation::{
        rtps::messages::submessages::PadSubmessageRead,
        rtps_udp_psm::mapping_traits::{from_bytes, to_bytes},
    };

    use super::*;

    #[test]
    fn serialize_pad() {
        let submessage = PadSubmessageWrite {
            endianness_flag: true,
        };
        #[rustfmt::skip]
        assert_eq!(to_bytes(&submessage).unwrap(), vec![
                0x01, 0b_0000_0001, 0, 0, // Submessage header
            ]
        );
    }

    #[test]
    fn deserialize_pad() {
        #[rustfmt::skip]
        let submessage = PadSubmessageRead::new(&[
            0x01, 0b_0000_0001, 0, 0, // Submessage header
        ]);
        let expected_endianness_flag = true;
        assert_eq!(expected_endianness_flag, submessage.endianness());
    }
}
