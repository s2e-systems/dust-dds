use crate::implementation::rtps::{
    messages::{
        overall_structure::{
            RtpsMap, Submessage, SubmessageHeader, SubmessageHeaderRead, SubmessageHeaderWrite,
        },
        submessage_elements::SubmessageElement,
        types::{SubmessageFlag, SubmessageKind},
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
pub struct InfoSourceSubmessageWrite<'a> {
    endianness_flag: SubmessageFlag,
    submessage_elements: [SubmessageElement<'a>; 4],
}

impl InfoSourceSubmessageWrite<'_> {
    pub fn new(
        protocol_version: ProtocolVersion,
        vendor_id: VendorId,
        guid_prefix: GuidPrefix,
    ) -> Self {
        Self {
            endianness_flag: true,
            submessage_elements: [
                SubmessageElement::Long(0),
                SubmessageElement::ProtocolVersion(protocol_version),
                SubmessageElement::VendorId(vendor_id),
                SubmessageElement::GuidPrefix(guid_prefix),
            ],
        }
    }
}

impl Submessage for InfoSourceSubmessageWrite<'_> {
    fn submessage_header(&self, octets_to_next_header: u16) -> SubmessageHeaderWrite {
        SubmessageHeaderWrite::new(
            SubmessageKind::INFO_SRC,
            &[self.endianness_flag],
            octets_to_next_header,
        )
    }

    fn submessage_elements(&self) -> &[SubmessageElement] {
        &self.submessage_elements
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::rtps::{
        messages::overall_structure::into_bytes_vec,
        types::{GUIDPREFIX_UNKNOWN, PROTOCOLVERSION_1_0, _VENDOR_ID_UNKNOWN},
    };

    #[test]
    fn serialize_info_source() {
        let submessage = InfoSourceSubmessageWrite::new(
            PROTOCOLVERSION_1_0,
            _VENDOR_ID_UNKNOWN,
            GUIDPREFIX_UNKNOWN,
        );
        #[rustfmt::skip]
        assert_eq!(into_bytes_vec(submessage), vec![
                0x0c, 0b_0000_0001, 20, 0, // Submessage header
                0, 0, 0, 0, // unused
                1, 0, 0, 0, //protocol_version | vendor_id
                0, 0, 0, 0, //guid_prefix
                0, 0, 0, 0, //guid_prefix
                0, 0, 0, 0, //guid_prefix
            ]
        );
    }

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

        let expected_protocol_version = PROTOCOLVERSION_1_0;
        let expected_vendor_id = _VENDOR_ID_UNKNOWN;
        let expected_guid_prefix = GUIDPREFIX_UNKNOWN;

        assert_eq!(expected_protocol_version, submessage.protocol_version());
        assert_eq!(expected_vendor_id, submessage.vendor_id());
        assert_eq!(expected_guid_prefix, submessage.guid_prefix());
    }
}
