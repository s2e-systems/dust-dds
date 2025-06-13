use crate::{
    rtps_messages::overall_structure::{RtpsMessageHeader, RtpsMessageWrite, Submessage},
    transport::types::GuidPrefix,
};

use super::types::{PROTOCOLVERSION_2_4, VENDOR_ID_S2E};

impl RtpsMessageWrite {
    pub fn from_submessages(
        submessages: &[&(dyn Submessage + Send)],
        guid_prefix: GuidPrefix,
    ) -> Self {
        let header = RtpsMessageHeader::new(PROTOCOLVERSION_2_4, VENDOR_ID_S2E, guid_prefix);
        RtpsMessageWrite::new(&header, submessages)
    }
}
