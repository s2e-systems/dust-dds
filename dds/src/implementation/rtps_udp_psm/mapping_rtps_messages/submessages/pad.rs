use super::submessage::MappingWriteSubmessage;
use crate::implementation::rtps::messages::{
    overall_structure::SubmessageHeaderWrite, submessages::pad::PadSubmessageWrite,
    types::SubmessageKind,
};
use byteorder::ByteOrder;
use std::io::{Error, Write};

impl MappingWriteSubmessage for PadSubmessageWrite {
    fn submessage_header(&self) -> SubmessageHeaderWrite {
        SubmessageHeaderWrite {
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
        rtps::messages::submessages::pad::PadSubmessageWrite,
        rtps_udp_psm::mapping_traits::to_bytes,
    };

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

}
