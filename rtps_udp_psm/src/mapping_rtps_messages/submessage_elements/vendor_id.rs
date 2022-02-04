use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::{
    mapping_traits::{MappingReadByteOrdered, MappingWriteByteOrdered},
    messages::submessage_elements::VendorIdSubmessageElementPsm,
};

impl MappingWriteByteOrdered for VendorIdSubmessageElementPsm {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        self.value.mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for VendorIdSubmessageElementPsm {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        Ok(Self {
            value: MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapping_traits::{from_bytes_le, to_bytes_le};

    #[test]
    fn serialize_vendor_id() {
        let data = VendorIdSubmessageElementPsm { value: [1, 2] };
        assert_eq!(to_bytes_le(&data).unwrap(), vec![1, 2,]);
    }

    #[test]
    fn deserialize_vendor_id() {
        let expected = VendorIdSubmessageElementPsm { value: [1, 2] };
        assert_eq!(expected, from_bytes_le(&[1, 2,]).unwrap());
    }
}
