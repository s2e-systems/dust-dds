use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::implementation::rtps::messages::{
    overall_structure::RtpsSubmessageHeader, submessages::PadSubmessage, types::SubmessageKind,
};

use super::submessage::{MappingReadSubmessage, MappingWriteSubmessage};

impl MappingWriteSubmessage for PadSubmessage {
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

impl<'de> MappingReadSubmessage<'de> for PadSubmessage {
    fn mapping_read_submessage<B: ByteOrder>(
        _buf: &mut &'de [u8],
        header: RtpsSubmessageHeader,
    ) -> Result<Self, Error> {
        Ok(PadSubmessage {
            endianness_flag: header.flags[0],
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::implementation::rtps_udp_psm::mapping_traits::{from_bytes, to_bytes};

    use super::*;

    #[test]
    fn serialize_pad() {
        let submessage = PadSubmessage {
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
        let buf = [
            0x01, 0b_0000_0001, 0, 0, // Submessage header
        ];
        assert_eq!(
            PadSubmessage {
                endianness_flag: true
            },
            from_bytes(&buf).unwrap()
        );
    }
}
