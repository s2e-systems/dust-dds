use dds_transport::messages::{
    overall_structure::RtpsSubmessageHeader, submessage_elements::TimestampSubmessageElement,
    submessages::InfoTimestampSubmessage, types::SubmessageKind,
};

use std::io::{Error, Write};

use crate::mapping_traits::{MappingReadByteOrdered, MappingWriteByteOrdered};

use super::submessage::{MappingReadSubmessage, MappingWriteSubmessage};

impl MappingWriteSubmessage for InfoTimestampSubmessage {
    fn submessage_header(&self) -> RtpsSubmessageHeader {
        let submessage_length = match self.invalidate_flag {
            true => 0,
            false => 8,
        };
        RtpsSubmessageHeader {
            submessage_id: SubmessageKind::INFO_TS,
            flags: [
                self.endianness_flag,
                self.invalidate_flag,
                false,
                false,
                false,
                false,
                false,
                false,
            ],
            submessage_length,
        }
    }

    fn mapping_write_submessage_elements<W: Write, B: byteorder::ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        if !self.invalidate_flag {
            self.timestamp
                .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        }
        Ok(())
    }
}
impl<'de> MappingReadSubmessage<'de> for InfoTimestampSubmessage {
    fn mapping_read_submessage<B: byteorder::ByteOrder>(
        buf: &mut &'de [u8],
        header: RtpsSubmessageHeader,
    ) -> Result<Self, Error> {
        let endianness_flag = header.flags[0];
        let invalidate_flag = header.flags[1];
        let timestamp = if invalidate_flag {
            TimestampSubmessageElement { value: u64::MAX }
        } else {
            MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?
        };
        Ok(InfoTimestampSubmessage {
            endianness_flag,
            invalidate_flag,
            timestamp,
        })
    }
}

#[cfg(test)]
mod tests {
    use dds_transport::messages::submessage_elements::TimestampSubmessageElement;

    use crate::mapping_traits::{from_bytes, to_bytes};

    use super::*;

    #[test]
    fn serialize_info_timestamp_valid_time() {
        let submessage = InfoTimestampSubmessage {
            endianness_flag: true,
            invalidate_flag: false,
            timestamp: TimestampSubmessageElement { value: 4000000000 },
        };
        #[rustfmt::skip]
        assert_eq!(to_bytes(&submessage).unwrap(), vec![
                0x09_u8, 0b_0000_0001, 8, 0, // Submessage header
                4, 0, 0, 0, // Time
                0, 0, 0, 0, // Time
            ]
        );
    }

    #[test]
    fn serialize_info_timestamp_invalid_time() {
        let submessage = InfoTimestampSubmessage {
            endianness_flag: true,
            invalidate_flag: true,
            timestamp: TimestampSubmessageElement { value: u64::MAX },
        };
        #[rustfmt::skip]
        assert_eq!(to_bytes(&submessage).unwrap(), vec![
                0x09_u8, 0b_0000_0011, 0, 0, // Submessage header
            ]
        );
    }

    #[test]
    fn deserialize_info_timestamp_valid_time() {
        #[rustfmt::skip]
        let buf = [
            0x09_u8, 0b_0000_0001, 8, 0, // Submessage header
            4, 0, 0, 0, // Time
            0, 0, 0, 0, // Time
        ];

        assert_eq!(
            InfoTimestampSubmessage {
                endianness_flag: true,
                invalidate_flag: false,
                timestamp: TimestampSubmessageElement { value: 4000000000 },
            },
            from_bytes(&buf).unwrap()
        )
    }

    #[test]
    fn deserialize_info_timestamp_invalid_time() {
        #[rustfmt::skip]
        let buf = [
            0x09_u8, 0b_0000_0011, 0, 0, // Submessage header
        ];

        assert_eq!(
            InfoTimestampSubmessage {
                endianness_flag: true,
                invalidate_flag: true,
                timestamp: TimestampSubmessageElement { value: u64::MAX },
            },
            from_bytes(&buf).unwrap()
        )
    }
}
