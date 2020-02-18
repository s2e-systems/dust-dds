use crate::types::{GuidPrefix, ProtocolVersion, VendorId};

use super::{deserialize, endianess, Result};

#[derive(PartialEq, Debug)]
pub struct InfoSrc {
    protocol_version: ProtocolVersion,
    vendor_id: VendorId,
    guid_prefix: GuidPrefix,
}

impl InfoSrc {
    pub fn new(
        protocol_version: ProtocolVersion,
        vendor_id: VendorId,
        guid_prefix: GuidPrefix,
    ) -> InfoSrc {
        InfoSrc {
            protocol_version,
            vendor_id,
            guid_prefix,
        }
    }

    pub fn get_protocol_version(&self) -> &ProtocolVersion {
        &self.protocol_version
    }

    pub fn get_vendor_id(&self) -> &VendorId {
        &self.vendor_id
    }

    pub fn get_guid_prefix(&self) -> &GuidPrefix {
        &self.guid_prefix
    }

    pub fn take(self) -> (ProtocolVersion, VendorId, GuidPrefix) {
        (self.protocol_version, self.vendor_id, self.guid_prefix)
    }
}

pub fn parse_info_source_submessage(submessage: &[u8], submessage_flags: &u8) -> Result<InfoSrc> {
    const PROTOCOL_VERSION_FIRST_INDEX: usize = 4;
    const PROTOCOL_VERSION_LAST_INDEX: usize = 5;
    const VENDOR_ID_FIRST_INDEX: usize = 6;
    const VENDOR_ID_LAST_INDEX: usize = 7;
    const GUID_PREFIX_FIRST_INDEX: usize = 8;
    const GUID_PREFIX_LAST_INDEX: usize = 19;

    let submessage_endianess = endianess(submessage_flags)?;
    let protocol_version = deserialize::<ProtocolVersion>(
        submessage,
        &PROTOCOL_VERSION_FIRST_INDEX,
        &PROTOCOL_VERSION_LAST_INDEX,
        &submessage_endianess,
    )?;
    let vendor_id = deserialize::<VendorId>(
        submessage,
        &VENDOR_ID_FIRST_INDEX,
        &VENDOR_ID_LAST_INDEX,
        &submessage_endianess,
    )?;
    let guid_prefix = deserialize::<GuidPrefix>(
        submessage,
        &GUID_PREFIX_FIRST_INDEX,
        &GUID_PREFIX_LAST_INDEX,
        &submessage_endianess,
    )?;

    Ok(InfoSrc {
        protocol_version,
        vendor_id,
        guid_prefix,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_info_source_submessage() {
        {
            let submessage = [
                0x00, 0x00, 0x00, 0x00, 0x02, 0x04, 0x10, 0x20, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06,
                0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C,
            ];

            let info_src_big_endian = parse_info_source_submessage(&submessage, &0).unwrap();

            assert_eq!(info_src_big_endian.protocol_version.major, 2);
            assert_eq!(info_src_big_endian.protocol_version.minor, 4);
            assert_eq!(info_src_big_endian.vendor_id, [0x10, 0x20]);
            assert_eq!(
                info_src_big_endian.guid_prefix,
                [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C,]
            );

            let info_src_little_endian = parse_info_source_submessage(&submessage, &1).unwrap();

            assert_eq!(info_src_little_endian.protocol_version.major, 2);
            assert_eq!(info_src_little_endian.protocol_version.minor, 4);
            assert_eq!(info_src_little_endian.vendor_id, [0x10, 0x20]);
            assert_eq!(
                info_src_little_endian.guid_prefix,
                [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C,]
            );
        }
    }
}
