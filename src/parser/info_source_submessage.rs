use super::{Result, InfoSrc, deserialize, endianess};

pub fn parse_info_source_submessage(submessage: &[u8], submessage_flags: &u8) -> Result<InfoSrc> {
    const INFO_SRC_FIRST_INDEX: usize = 4;
    let info_src_last_index = submessage.len()-1;

    let submessage_endianess = endianess(submessage_flags)?;

    deserialize::<InfoSrc>(submessage, &INFO_SRC_FIRST_INDEX, &info_src_last_index, &submessage_endianess)
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_parse_info_source_submessage() {
        {
            let submessage = [
                0x00, 0x00, 0x00, 0x00,
                0x02, 0x04, 0x10, 0x20,
                0x01, 0x02, 0x03, 0x04,
                0x05, 0x06, 0x07, 0x08,
                0x09, 0x0A, 0x0B, 0x0C,
            ];

            let info_src_big_endian = parse_info_source_submessage(&submessage, &0).unwrap();

            assert_eq!(info_src_big_endian.protocol_version.major, 2);
            assert_eq!(info_src_big_endian.protocol_version.minor, 4);
            assert_eq!(info_src_big_endian.vendor_id, [0x10, 0x20]);
            assert_eq!(info_src_big_endian.guid_prefix, [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C,]);

            let info_src_little_endian = parse_info_source_submessage(&submessage, &1).unwrap();

            assert_eq!(info_src_little_endian.protocol_version.major, 2);
            assert_eq!(info_src_little_endian.protocol_version.minor, 4);
            assert_eq!(info_src_little_endian.vendor_id, [0x10, 0x20]);
            assert_eq!(info_src_little_endian.guid_prefix, [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C,]);
        }
        
    }

}