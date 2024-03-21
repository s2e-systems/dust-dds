use crate::{
    implementation::rtps::{
        messages::{
            overall_structure::{Submessage, SubmessageHeaderWrite},
            submessage_elements::SubmessageElement,
            types::SubmessageKind,
        },
        types::{Endianness, GuidPrefix, ProtocolVersion, TryFromBytes, VendorId},
    },
    infrastructure::error::{DdsError, DdsResult},
};

#[derive(Debug, PartialEq, Eq)]
pub struct InfoSourceSubmessageRead {
    protocol_version: ProtocolVersion,
    vendor_id: VendorId,
    guid_prefix: GuidPrefix,
}

impl InfoSourceSubmessageRead {
    pub fn try_from_bytes(data: &[u8]) -> DdsResult<Self> {
        if data.len() >= 24 {
            let flags = data[1];
            let endianness = &Endianness::from_flags(flags);
            let mut data = &data[8..];
            Ok(Self {
                protocol_version: ProtocolVersion::try_from_bytes(&mut data, endianness)?,
                vendor_id: VendorId::try_from_bytes(data, endianness)?,
                guid_prefix: GuidPrefix::try_from_bytes(&data[2..], endianness)?,
            })
        } else {
            Err(DdsError::Error("InfoSource submessage invalid".to_string()))
        }
    }

    pub fn protocol_version(&self) -> ProtocolVersion {
        self.protocol_version
    }

    pub fn vendor_id(&self) -> VendorId {
        self.vendor_id
    }

    pub fn guid_prefix(&self) -> GuidPrefix {
        self.guid_prefix
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct InfoSourceSubmessageWrite<'a> {
    submessage_elements: [SubmessageElement<'a>; 4],
}

impl InfoSourceSubmessageWrite<'_> {
    pub fn _new(
        protocol_version: ProtocolVersion,
        vendor_id: VendorId,
        guid_prefix: GuidPrefix,
    ) -> Self {
        Self {
            submessage_elements: [
                SubmessageElement::Long(0),
                SubmessageElement::ProtocolVersion(protocol_version),
                SubmessageElement::VendorId(vendor_id),
                SubmessageElement::GuidPrefix(guid_prefix),
            ],
        }
    }
}

impl<'a> Submessage<'a> for InfoSourceSubmessageWrite<'a> {
    type SubmessageList = &'a [SubmessageElement<'a>];

    fn submessage_header(&self, octets_to_next_header: u16) -> SubmessageHeaderWrite {
        SubmessageHeaderWrite::new(SubmessageKind::INFO_SRC, &[], octets_to_next_header)
    }

    fn submessage_elements(&'a self) -> Self::SubmessageList {
        &self.submessage_elements
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::rtps::{
        messages::overall_structure::{into_bytes_vec, RtpsSubmessageWriteKind},
        types::{GUIDPREFIX_UNKNOWN, PROTOCOLVERSION_1_0, VENDOR_ID_UNKNOWN},
    };

    #[test]
    fn serialize_info_source() {
        let submessage = RtpsSubmessageWriteKind::InfoSource(InfoSourceSubmessageWrite::_new(
            PROTOCOLVERSION_1_0,
            VENDOR_ID_UNKNOWN,
            GUIDPREFIX_UNKNOWN,
        ));
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
        let submessage = InfoSourceSubmessageRead::try_from_bytes(&[
            0x0c, 0b_0000_0001, 20, 0, // Submessage header
            0, 0, 0, 0, // unused
            1, 0, 0, 0, //protocol_version | vendor_id
            0, 0, 0, 0, //guid_prefix
            0, 0, 0, 0, //guid_prefix
            0, 0, 0, 0, //guid_prefix
        ]).unwrap();

        let expected_protocol_version = PROTOCOLVERSION_1_0;
        let expected_vendor_id = VENDOR_ID_UNKNOWN;
        let expected_guid_prefix = GUIDPREFIX_UNKNOWN;

        assert_eq!(expected_protocol_version, submessage.protocol_version());
        assert_eq!(expected_vendor_id, submessage.vendor_id());
        assert_eq!(expected_guid_prefix, submessage.guid_prefix());
    }
}
