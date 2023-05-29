use crate::implementation::{
    rtps::types::VendorId, rtps_udp_psm::mapping_traits::MappingWriteByteOrdered,
};
use byteorder::ByteOrder;
use std::io::{Error, Write};

impl MappingWriteByteOrdered for VendorId {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        <[u8; 2]>::from(*self).mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::rtps_udp_psm::mapping_traits::to_bytes_le;

    #[test]
    fn serialize_vendor_id() {
        let data = VendorId::new([1, 2]);
        assert_eq!(to_bytes_le(&data).unwrap(), vec![1, 2,]);
    }
}
