use super::helpers::{deserialize, endianess};

use super::{Result, InfoDst};

pub fn parse_info_dst_submessage(submessage: &[u8], submessage_flags: &u8) -> Result<InfoDst> {
    const GUID_PREFIX_FIRST_INDEX: usize = 0;
    const GUID_PREFIX_LAST_INDEX: usize = 11;
    let submessage_endianess = endianess(submessage_flags)?;
    deserialize::<InfoDst>(submessage, &GUID_PREFIX_FIRST_INDEX, &GUID_PREFIX_LAST_INDEX, &submessage_endianess)
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_parse_info_dst_submessage() {
        {
            let submessage_big_endian = [
                10,11,12,13,
                14,15,16,17,
                18,19,20,21,
            ];
            let info_dst_big_endian = parse_info_dst_submessage(&submessage_big_endian, &0).unwrap();
            assert_eq!(info_dst_big_endian.guid_prefix,[10,11,12,13,14,15,16,17,18,19,20,21]);
        }

        {
            let submessage_little_endian = [
                10,11,12,13,
                14,15,16,17,
                18,19,20,21,
            ];
            let info_dst_little_endian = parse_info_dst_submessage(&submessage_little_endian, &1).unwrap();
            assert_eq!(info_dst_little_endian.guid_prefix,[10,11,12,13,14,15,16,17,18,19,20,21]);
        }
    }
}