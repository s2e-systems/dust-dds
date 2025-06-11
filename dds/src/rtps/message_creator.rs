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
