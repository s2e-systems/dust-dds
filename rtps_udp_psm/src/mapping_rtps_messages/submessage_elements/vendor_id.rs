use std::io::Write;

use byteorder::ByteOrder;
use rust_rtps_pim::messages::submessage_elements::VendorIdSubmessageElement;

use crate::{
    deserialize::{self, MappingReadByteOrdered},
    serialize::{self, MappingWriteByteOrdered},
};

impl MappingWriteByteOrdered for VendorIdSubmessageElement {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        self.value.mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for VendorIdSubmessageElement {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        Ok(Self {
            value: MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deserialize::from_bytes_le;
    use crate::serialize::to_bytes_le;

    #[test]
    fn serialize_vendor_id() {
        let data = VendorIdSubmessageElement { value: [1, 2] };
        assert_eq!(to_bytes_le(&data).unwrap(), vec![1, 2,]);
    }

    #[test]
    fn deserialize_vendor_id() {
        let expected = VendorIdSubmessageElement { value: [1, 2] };
        assert_eq!(expected, from_bytes_le(&[1, 2,]).unwrap());
    }
}
