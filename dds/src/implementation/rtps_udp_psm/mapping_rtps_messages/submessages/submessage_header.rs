use std::io::{Error, Write};

use byteorder::{ ByteOrder,  WriteBytesExt};

use crate::implementation::{
    rtps::messages::{
        overall_structure::SubmessageHeaderWrite,
        types::{SubmessageFlag, },
    },
    rtps_udp_psm::mapping_traits::{MappingWriteByteOrderInfoInData, MappingWriteByteOrdered},
};

pub trait Submessage {
    fn submessage_header(&self) -> SubmessageHeaderWrite;
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

impl MappingWriteByteOrderInfoInData for SubmessageHeaderWrite {
    fn mapping_write_byte_order_info_in_data<W: Write>(&self, mut _writer: W) -> Result<(), Error> {
        // let submessage_id = match self.submessage_id {
        //     SubmessageKind::DATA => DATA,
        //     SubmessageKind::GAP => GAP,
        //     SubmessageKind::HEARTBEAT => HEARTBEAT,
        //     SubmessageKind::ACKNACK => ACKNACK,
        //     SubmessageKind::PAD => PAD,
        //     SubmessageKind::INFO_TS => INFO_TS,
        //     SubmessageKind::INFO_REPLY => INFO_REPLY,
        //     SubmessageKind::INFO_DST => INFO_DST,
        //     SubmessageKind::INFO_SRC => INFO_SRC,
        //     SubmessageKind::DATA_FRAG => DATA_FRAG,
        //     SubmessageKind::NACK_FRAG => NACK_FRAG,
        //     SubmessageKind::HEARTBEAT_FRAG => HEARTBEAT_FRAG,
        // };
        // if self.flags[0] {
        //     submessage_id.mapping_write_byte_ordered::<_, LittleEndian>(&mut writer)?;
        //     self.flags
        //         .mapping_write_byte_ordered::<_, LittleEndian>(&mut writer)?;
        //     self.submessage_length
        //         .mapping_write_byte_ordered::<_, LittleEndian>(&mut writer)
        // } else {
        //     submessage_id.mapping_write_byte_ordered::<_, BigEndian>(&mut writer)?;
        //     self.flags
        //         .mapping_write_byte_ordered::<_, BigEndian>(&mut writer)?;
        //     self.submessage_length
        //         .mapping_write_byte_ordered::<_, BigEndian>(&mut writer)
        // }
        todo!()
    }
}

impl MappingWriteByteOrdered for SubmessageHeaderWrite {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut _writer: W,
    ) -> Result<(), Error> {
        // let submessage_id = match self.submessage_id {
        //     SubmessageKind::DATA => DATA,
        //     SubmessageKind::GAP => GAP,
        //     SubmessageKind::HEARTBEAT => HEARTBEAT,
        //     SubmessageKind::ACKNACK => ACKNACK,
        //     SubmessageKind::PAD => PAD,
        //     SubmessageKind::INFO_TS => INFO_TS,
        //     SubmessageKind::INFO_REPLY => INFO_REPLY,
        //     SubmessageKind::INFO_DST => INFO_DST,
        //     SubmessageKind::INFO_SRC => INFO_SRC,
        //     SubmessageKind::DATA_FRAG => DATA_FRAG,
        //     SubmessageKind::NACK_FRAG => NACK_FRAG,
        //     SubmessageKind::HEARTBEAT_FRAG => HEARTBEAT_FRAG,
        // };
        // submessage_id.mapping_write_byte_ordered::<_, B>(&mut writer)?;
        // self.flags.mapping_write_byte_ordered::<_, B>(&mut writer)?;
        // writer.write_u16::<B>(self.submessage_length)
        todo!()
    }
}

#[cfg(test)]
mod tests {

    use crate::implementation::{rtps_udp_psm::mapping_traits::to_bytes_le, rtps::messages::types::SubmessageKind};

    use super::*;

    #[test]
    fn serialize_submessage_flags() {
        let result = to_bytes_le(&[true, false, true, false, false, false, false, false]).unwrap();
        assert_eq!(result, vec![0b_0000_0101]);
    }

    #[test]
    fn serialize_rtps_submessage_header() {
        let value = SubmessageHeaderWrite::new(SubmessageKind::ACKNACK, &[true; 8], 16);
        assert_eq!(
            to_bytes_le(&value).unwrap(),
            vec![0x06, 0b_1111_1111, 16, 0]
        );
    }
}
