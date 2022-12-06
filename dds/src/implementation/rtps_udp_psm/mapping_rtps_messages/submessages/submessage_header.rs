use std::io::{Error, Write};

use byteorder::{BigEndian, ByteOrder, LittleEndian, WriteBytesExt};

use crate::implementation::{
    rtps::messages::{
        overall_structure::RtpsSubmessageHeader,
        types::{SubmessageFlag, SubmessageKind},
    },
    rtps_udp_psm::mapping_traits::{
        MappingReadByteOrderInfoInData, MappingReadByteOrdered, MappingWriteByteOrderInfoInData,
        MappingWriteByteOrdered,
    },
};

pub trait Submessage {
    fn submessage_header(&self) -> RtpsSubmessageHeader;
}

pub const DATA: u8 = 0x15;
pub const GAP: u8 = 0x08;
pub const HEARTBEAT: u8 = 0x07;
pub const ACKNACK: u8 = 0x06;
pub const PAD: u8 = 0x01;
pub const INFO_TS: u8 = 0x09;
pub const INFO_REPLY: u8 = 0x0f;
pub const INFO_DST: u8 = 0x0e;
pub const INFO_SRC: u8 = 0x0c;
pub const DATA_FRAG: u8 = 0x16;
pub const NACK_FRAG: u8 = 0x12;
pub const HEARTBEAT_FRAG: u8 = 0x13;

impl MappingWriteByteOrdered for [SubmessageFlag; 8] {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        let mut flags = 0b_0000_0000_u8;
        for (i, &item) in self.iter().enumerate() {
            if item {
                flags |= 0b_0000_0001 << i
            }
        }
        flags.mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl MappingWriteByteOrderInfoInData for [SubmessageFlag; 8] {
    fn mapping_write_byte_order_info_in_data<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        let mut flags = 0b_0000_0000_u8;
        for (i, &item) in self.iter().enumerate() {
            if item {
                flags |= 0b_0000_0001 << i
            }
        }
        writer.write_u8(flags)
    }
}

impl<'de> MappingReadByteOrdered<'de> for [SubmessageFlag; 8] {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        let value: u8 = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let mut flags = [false; 8];
        for (index, flag) in flags.iter_mut().enumerate() {
            *flag = value & (0b_0000_0001 << index) != 0;
        }
        Ok(flags)
    }
}

impl MappingWriteByteOrderInfoInData for RtpsSubmessageHeader {
    fn mapping_write_byte_order_info_in_data<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        let submessage_id = match self.submessage_id {
            SubmessageKind::DATA => DATA,
            SubmessageKind::GAP => GAP,
            SubmessageKind::HEARTBEAT => HEARTBEAT,
            SubmessageKind::ACKNACK => ACKNACK,
            SubmessageKind::PAD => PAD,
            SubmessageKind::INFO_TS => INFO_TS,
            SubmessageKind::INFO_REPLY => INFO_REPLY,
            SubmessageKind::INFO_DST => INFO_DST,
            SubmessageKind::INFO_SRC => INFO_SRC,
            SubmessageKind::DATA_FRAG => DATA_FRAG,
            SubmessageKind::NACK_FRAG => NACK_FRAG,
            SubmessageKind::HEARTBEAT_FRAG => HEARTBEAT_FRAG,
            SubmessageKind::UNKNOWN => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Unknow submessage kind not allowed",
                ))
            }
        };
        if self.flags[0] {
            submessage_id.mapping_write_byte_ordered::<_, LittleEndian>(&mut writer)?;
            self.flags
                .mapping_write_byte_ordered::<_, LittleEndian>(&mut writer)?;
            self.submessage_length
                .mapping_write_byte_ordered::<_, LittleEndian>(&mut writer)
        } else {
            submessage_id.mapping_write_byte_ordered::<_, BigEndian>(&mut writer)?;
            self.flags
                .mapping_write_byte_ordered::<_, BigEndian>(&mut writer)?;
            self.submessage_length
                .mapping_write_byte_ordered::<_, BigEndian>(&mut writer)
        }
    }
}

impl MappingWriteByteOrdered for RtpsSubmessageHeader {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        let submessage_id = match self.submessage_id {
            SubmessageKind::DATA => DATA,
            SubmessageKind::GAP => GAP,
            SubmessageKind::HEARTBEAT => HEARTBEAT,
            SubmessageKind::ACKNACK => ACKNACK,
            SubmessageKind::PAD => PAD,
            SubmessageKind::INFO_TS => INFO_TS,
            SubmessageKind::INFO_REPLY => INFO_REPLY,
            SubmessageKind::INFO_DST => INFO_DST,
            SubmessageKind::INFO_SRC => INFO_SRC,
            SubmessageKind::DATA_FRAG => DATA_FRAG,
            SubmessageKind::NACK_FRAG => NACK_FRAG,
            SubmessageKind::HEARTBEAT_FRAG => HEARTBEAT_FRAG,
            SubmessageKind::UNKNOWN => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Unknow submessage kind not allowed",
                ))
            }
        };
        submessage_id.mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.flags.mapping_write_byte_ordered::<_, B>(&mut writer)?;
        writer.write_u16::<B>(self.submessage_length)
    }
}

impl<'de> MappingReadByteOrderInfoInData<'de> for RtpsSubmessageHeader {
    fn mapping_read_byte_order_info_in_data(buf: &mut &'de [u8]) -> Result<Self, Error> {
        // The byteorder is determined by the one after next element. Since the submessage_id
        // is not byteorder dependent, just use a specific one (to avoid look ahead)
        let submessage_id: u8 =
            MappingReadByteOrdered::mapping_read_byte_ordered::<LittleEndian>(buf)?;
        let submessage_id = match submessage_id {
            DATA => SubmessageKind::DATA,
            GAP => SubmessageKind::GAP,
            HEARTBEAT => SubmessageKind::HEARTBEAT,
            ACKNACK => SubmessageKind::ACKNACK,
            PAD => SubmessageKind::PAD,
            INFO_TS => SubmessageKind::INFO_TS,
            INFO_REPLY => SubmessageKind::INFO_REPLY,
            INFO_DST => SubmessageKind::INFO_DST,
            INFO_SRC => SubmessageKind::INFO_SRC,
            DATA_FRAG => SubmessageKind::DATA_FRAG,
            NACK_FRAG => SubmessageKind::NACK_FRAG,
            HEARTBEAT_FRAG => SubmessageKind::HEARTBEAT_FRAG,
            _ => SubmessageKind::UNKNOWN,
        };
        // Also decide byteorder here
        let flags: [SubmessageFlag; 8] =
            MappingReadByteOrdered::mapping_read_byte_ordered::<LittleEndian>(buf)?;
        let submessage_length = if flags[0] {
            MappingReadByteOrdered::mapping_read_byte_ordered::<LittleEndian>(buf)?
        } else {
            MappingReadByteOrdered::mapping_read_byte_ordered::<BigEndian>(buf)?
        };
        Ok(Self {
            submessage_id,
            flags,
            submessage_length,
        })
    }
}


#[cfg(test)]
mod tests {

    use crate::implementation::rtps_udp_psm::mapping_traits::{from_bytes_le, to_bytes_le, from_bytes};

    use super::*;

    #[test]
    fn serialize_submessage_flags() {
        let result = to_bytes_le(&[true, false, true, false, false, false, false, false]).unwrap();
        assert_eq!(result, vec![0b_0000_0101]);
    }

    #[test]
    fn deserialize_submessage_flags() {
        let expected = [false, false, false, false, false, true, false, true];
        let result: [SubmessageFlag; 8] = from_bytes_le(&[0b_1010_0000]).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn serialize_rtps_submessage_header() {
        let value = RtpsSubmessageHeader {
            submessage_id: SubmessageKind::ACKNACK,
            flags: [true; 8],
            submessage_length: 16,
        };
        assert_eq!(
            to_bytes_le(&value).unwrap(),
            vec![0x06, 0b_1111_1111, 16, 0]
        );
    }

    #[test]
    fn deserialize_rtps_header() {
        let expected = RtpsSubmessageHeader {
            submessage_id: SubmessageKind::ACKNACK,
            flags: [true; 8],
            submessage_length: 16,
        };
        let result = from_bytes(&[0x06, 0b_1111_1111, 16, 0]).unwrap();
        assert_eq!(expected, result);
    }
}
