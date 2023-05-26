use crate::implementation::rtps::{
    messages::{
        overall_structure::SubmessageHeaderRead, types::SubmessageFlag, RtpsMap, SubmessageHeader,
    },
    types::{GuidPrefix, ProtocolVersion, VendorId},
};

#[derive(Debug, PartialEq, Eq)]
pub struct InfoSourceSubmessageRead<'a> {
    data: &'a [u8],
}

impl SubmessageHeader for InfoSourceSubmessageRead<'_> {
    fn submessage_header(&self) -> SubmessageHeaderRead {
        SubmessageHeaderRead::new(self.data)
    }
}

impl<'a> InfoSourceSubmessageRead<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    pub fn protocol_version(&self) -> ProtocolVersion {
        self.map(&self.data[8..])
    }

    pub fn vendor_id(&self) -> VendorId {
        self.map(&self.data[10..])
    }

    pub fn guid_prefix(&self) -> GuidPrefix {
        self.map(&self.data[12..])
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct InfoSourceSubmessageWrite {
    pub endianness_flag: SubmessageFlag,
    pub protocol_version: ProtocolVersion,
    pub vendor_id: VendorId,
    pub guid_prefix: GuidPrefix,
}

#[cfg(test)]
mod tests {
    use crate::implementation::rtps::types::{
        GUIDPREFIX_UNKNOWN, PROTOCOLVERSION_1_0, VENDOR_ID_UNKNOWN,
    };

    use super::*;

    #[test]
    fn deserialize_info_source() {
        #[rustfmt::skip]
        let submessage = InfoSourceSubmessageRead::new(&[
            0x0c, 0b_0000_0001, 20, 0, // Submessage header
            0, 0, 0, 0, // unused
            1, 0, 0, 0, //protocol_version | vendor_id
            0, 0, 0, 0, //guid_prefix
            0, 0, 0, 0, //guid_prefix
            0, 0, 0, 0, //guid_prefix
        ]);

        let expected_endianness_flag = true;
        let expected_protocol_version = PROTOCOLVERSION_1_0;
        let expected_vendor_id = VENDOR_ID_UNKNOWN;
        let expected_guid_prefix = GUIDPREFIX_UNKNOWN;

        assert_eq!(expected_protocol_version, submessage.protocol_version());
        assert_eq!(expected_vendor_id, submessage.vendor_id());
        assert_eq!(expected_guid_prefix, submessage.guid_prefix());
    }
}
