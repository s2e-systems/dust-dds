use super::submessage::MappingWriteSubmessage;
use crate::implementation::{
    rtps::messages::{
        overall_structure::SubmessageHeaderWrite,
        submessages::info_timestamp::InfoTimestampSubmessageWrite, types::SubmessageKind,
    },
    rtps_udp_psm::mapping_traits::MappingWriteByteOrdered,
};
use std::io::{Error, Write};

impl MappingWriteSubmessage for InfoTimestampSubmessageWrite {
    fn submessage_header(&self) -> SubmessageHeaderWrite {
        // let submessage_length = match self.invalidate_flag {
        //     true => 0,
        //     false => 8,
        // };
        // SubmessageHeaderWrite {
        //     submessage_id: SubmessageKind::INFO_TS,
        //     flags: [
        //         self.endianness_flag,
        //         self.invalidate_flag,
        //         false,
        //         false,
        //         false,
        //         false,
        //         false,
        //         false,
        //     ],
        //     submessage_length,
        // }
        todo!()
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

#[cfg(test)]
mod tests {

    use crate::implementation::{
        rtps::messages::types::{Time, TIME_INVALID},
        rtps_udp_psm::mapping_traits::to_bytes,
    };

    use super::*;

    #[test]
    fn serialize_info_timestamp_valid_time() {
        let submessage = InfoTimestampSubmessageWrite {
            endianness_flag: true,
            invalidate_flag: false,
            timestamp: Time::new(4, 0),
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
        let submessage = InfoTimestampSubmessageWrite {
            endianness_flag: true,
            invalidate_flag: true,
            timestamp: TIME_INVALID,
        };
        #[rustfmt::skip]
        assert_eq!(to_bytes(&submessage).unwrap(), vec![
                0x09_u8, 0b_0000_0011, 0, 0, // Submessage header
            ]
        );
    }

}
