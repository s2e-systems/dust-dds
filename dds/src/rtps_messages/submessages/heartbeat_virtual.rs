use crate::{
    rtps::types::SequenceNumber,
    rtps_messages::{
        overall_structure::{Submessage, SubmessageHeaderWrite, Write, WriteIntoBytes},
        types::{Count, SubmessageKind},
    },
    transport::types::EntityId,
};
use alloc::vec::Vec;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VirtualWriterInfo {
    pub writer_id: EntityId,
    pub first_virtual_sn: SequenceNumber,
    pub last_virtual_sn: SequenceNumber,
    pub first_rtps_sn: SequenceNumber,
    pub last_rtps_sn: SequenceNumber,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct HeartbeatVirtualSubmessage {
    reader_id: EntityId,
    writer_id: EntityId,
    virtual_guid: [u8; 16],
    writers: Vec<VirtualWriterInfo>,
    count: Count,
}

impl HeartbeatVirtualSubmessage {
    pub fn new(
        reader_id: EntityId,
        writer_id: EntityId,
        virtual_guid: [u8; 16],
        writers: Vec<VirtualWriterInfo>,
        count: Count,
    ) -> Self {
        Self {
            reader_id,
            writer_id,
            virtual_guid,
            writers,
            count,
        }
    }
}

impl Submessage for HeartbeatVirtualSubmessage {
    fn write_submessage_header_into_bytes(&self, octets_to_next_header: u16, buf: &mut dyn Write) {
        SubmessageHeaderWrite::new(
            SubmessageKind::HEARTBEAT_VIRTUAL,
            &[false, true],
            octets_to_next_header,
        )
        .write_into_bytes(buf);
    }

    fn write_submessage_elements_into_bytes(&self, buf: &mut dyn Write) {
        self.reader_id.write_into_bytes(buf);
        self.writer_id.write_into_bytes(buf);
        self.virtual_guid.write_into_bytes(buf);
        (self.writers.len() as u32).write_into_bytes(buf);
        for writer in &self.writers {
            writer.writer_id.write_into_bytes(buf);
            1_u32.write_into_bytes(buf);
            writer.first_virtual_sn.write_into_bytes(buf);
            writer.last_virtual_sn.write_into_bytes(buf);
            writer.first_rtps_sn.write_into_bytes(buf);
            writer.last_rtps_sn.write_into_bytes(buf);
        }
        self.count.write_into_bytes(buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rtps_messages::overall_structure::write_submessage_into_bytes_vec;

    #[test]
    fn serialize_heartbeat_virtual() {
        let submessage = HeartbeatVirtualSubmessage::new(
            EntityId::new([0x80, 0x00, 0x00], 0x07),
            EntityId::new([0x80, 0x00, 0x00], 0x02),
            [
                0xb4, 0x92, 0x28, 0xf8, 0xa4, 0xa9, 0x11, 0xab, 0x91, 0xc5, 0xbf, 0x35, 0x35,
                0x59, 0x42, 0xe6,
            ],
            vec![
                VirtualWriterInfo {
                    writer_id: EntityId::new([0x80, 0x00, 0x00], 0x02),
                    first_virtual_sn: 181,
                    last_virtual_sn: 199,
                    first_rtps_sn: 61,
                    last_rtps_sn: 69,
                },
                VirtualWriterInfo {
                    writer_id: EntityId::new([0x80, 0x00, 0x21], 0x02),
                    first_virtual_sn: 185,
                    last_virtual_sn: 203,
                    first_rtps_sn: 61,
                    last_rtps_sn: 69,
                },
                VirtualWriterInfo {
                    writer_id: EntityId::new([0x80, 0x00, 0x42], 0x02),
                    first_virtual_sn: 189,
                    last_virtual_sn: 207,
                    first_rtps_sn: 61,
                    last_rtps_sn: 69,
                },
            ],
            8,
        );

        let bytes = write_submessage_into_bytes_vec(&submessage);
        #[rustfmt::skip]
        let expected = vec![
            0x1e, 0x05, 0x98, 0x00, 0x80, 0x00, 0x00, 0x07, 0x80, 0x00, 0x00, 0x02, 0xb4, 0x92, 0x28, 0xf8,
            0xa4, 0xa9, 0x11, 0xab, 0x91, 0xc5, 0xbf, 0x35, 0x35, 0x59, 0x42, 0xe6, 0x03, 0x00, 0x00, 0x00,
            0x80, 0x00, 0x00, 0x02, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xb5, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0xc7, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3d, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x45, 0x00, 0x00, 0x00, 0x80, 0x00, 0x21, 0x02, 0x01, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0xb9, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xcb, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x3d, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x45, 0x00, 0x00, 0x00,
            0x80, 0x00, 0x42, 0x02, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xbd, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0xcf, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3d, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x45, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00,
        ];
        assert_eq!(bytes, expected);
    }
}
