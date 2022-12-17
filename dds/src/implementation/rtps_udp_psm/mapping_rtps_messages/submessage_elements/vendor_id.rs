use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::implementation::{
    rtps::types::VendorId,
    rtps_udp_psm::mapping_traits::{MappingReadByteOrdered, MappingWriteByteOrdered},
};

impl MappingWriteByteOrdered for VendorId {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        <[u8; 2]>::from(*self).mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for VendorId {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        Ok(Self::new(
            MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::implementation::{
        rtps::types::VendorId,
        rtps_udp_psm::mapping_traits::{from_bytes_le, to_bytes_le},
    };

    #[test]
    fn serialize_vendor_id() {
        let data = VendorId::new([1, 2]);
        assert_eq!(to_bytes_le(&data).unwrap(), vec![1, 2,]);
    }

    #[test]
    fn deserialize_vendor_id() {
        let expected = VendorId::new([1, 2]);
        assert_eq!(expected, from_bytes_le(&[1, 2,]).unwrap());
    }
}
