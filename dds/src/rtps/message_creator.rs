use crate::{
    rtps_messages::overall_structure::{RtpsMessageHeader, RtpsMessageWrite, Submessage},
    transport::types::GuidPrefix,
};

use super::types::{PROTOCOLVERSION_2_4, VENDOR_ID_S2E};
use alloc::boxed::Box;

impl RtpsMessageWrite {
    pub fn from_submessages(
        submessages: &[Box<dyn Submessage + Send>],
        guid_prefix: GuidPrefix,
    ) -> Self {
        let header = RtpsMessageHeader::new(PROTOCOLVERSION_2_4, VENDOR_ID_S2E, guid_prefix);
        RtpsMessageWrite::new(&header, submessages)
    }
}

macro_rules! rtps_message_write {
    ($guid_prefix:expr, $($submessage:expr),*) => {{
        let header = dust_dds::rtps_messages::overall_structure::RtpsMessageHeader::new(dust_dds::rtps::types::PROTOCOLVERSION_2_4, dust_dds::rtps::types::VENDOR_ID_S2E, $guid_prefix);
        let buffer = Vec::with_capacity(1500);
        let mut cursor = dust_dds::rtps_messages::overall_structure::Cursor::new(buffer);
        dust_dds::rtps_messages::overall_structure::WriteIntoBytes::write_into_bytes(&header, &mut cursor);
        $(
            let submsg: &dyn dust_dds::rtps_messages::overall_structure::Submessage = &$submessage;
            let header_position: u64 = cursor.position();
            let elements_position = header_position + 4;
            cursor.set_position(elements_position);
            submsg.write_submessage_elements_into_bytes(&mut cursor);
            let pos = cursor.position();
            cursor.set_position(header_position);
            let len = pos - elements_position;
            submsg.write_submessage_header_into_bytes(len as u16, &mut cursor);
            cursor.set_position(pos);
        )*
        RtpsMessageWrite::from_prepared_arc(alloc::sync::Arc::from(cursor.into_inner().into_boxed_slice()))
    }};
}

#[cfg(test)]
mod tests {
    use crate::rtps_messages::overall_structure::Write;

    use super::*;

    use core::cell::Cell;

    // Dummy Submessage for testing
    struct DummySubmessage {
        id: u8,
        content: Cell<Vec<u8>>,
    }

    impl Submessage for DummySubmessage {
        fn write_submessage_header_into_bytes(&self, _body_length: u16, cursor: &mut dyn Write) {
            // trivial header: just the id, for test
            cursor.write_all(&[self.id]).unwrap();
        }
        fn write_submessage_elements_into_bytes(&self, cursor: &mut dyn Write) {
            // write some bytes for test
            let content = self.content.take();
            cursor.write_all(&content).unwrap();
            self.content.set(content); // Put back for possible checks
        }
    }

    #[test]
    fn test_rtps_message_write_macro_basic() {
        let guid_prefix: GuidPrefix = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let sm1 = DummySubmessage {
            id: 0xAA,
            content: Cell::new(vec![0x10, 0x20]),
        };
        let sm2 = DummySubmessage {
            id: 0xBB,
            content: Cell::new(vec![0x21, 0x23, 0x24]),
        };

        // Should not panic or error; returns RtpsMessageWrite
        let rtps_msg = rtps_message_write!(guid_prefix, sm1, sm2);
        let bytes = rtps_msg.buffer();

        // Check overall length: must include header (20) + sum of submessage lengths
        let expected_length = 20 + (4 + 2) + (4 + 3);
        assert_eq!(
            bytes.len(),
            expected_length,
            "Unexpected message buffer length"
        );
        // Check header: 'R', 'T', 'P', 'S'
        assert_eq!(&bytes[0..4], b"RTPS");
        // Check protocol version: major/minor
        assert_eq!(
            bytes[4],
            dust_dds::rtps::types::PROTOCOLVERSION_2_4._major()
        );
        assert_eq!(
            bytes[5],
            dust_dds::rtps::types::PROTOCOLVERSION_2_4._minor()
        );
        assert_eq!(bytes[6..8], dust_dds::rtps::types::VENDOR_ID_S2E);
        assert_eq!(&bytes[8..20], guid_prefix.as_ref());

        assert_eq!(bytes[20], 0xAA, "sm1 id missing");
        assert_eq!(bytes[24], 0x10, "sm1 content missing");
        assert_eq!(bytes[25], 0x20, "sm1 content missing");

        assert_eq!(bytes[26], 0xBB, "sm2 id missing");
        assert_eq!(bytes[30], 0x21, "sm2 content missing");
        assert_eq!(bytes[31], 0x23, "sm2 content missing");
        assert_eq!(bytes[32], 0x24, "sm2 content missing");
    }
}
