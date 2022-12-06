use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::implementation::{
    rtps::messages::submessage_elements::VendorIdSubmessageElement,
    rtps_udp_psm::mapping_traits::{
        MappingRead, MappingReadByteOrdered, MappingWriteByteOrderInfoInData, MappingWriteByteOrdered,
    },
};

impl MappingWriteByteOrderInfoInData for VendorIdSubmessageElement {
    fn mapping_write_byte_order_info_in_data<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        self.value.mapping_write_byte_order_info_in_data(&mut writer)
    }
}

impl<'de> MappingRead<'de> for VendorIdSubmessageElement {
    fn mapping_read(buf: &mut &'de [u8]) -> Result<Self, Error> {
        Ok(Self {
            value: MappingRead::mapping_read(buf)?,
        })
    }
}

impl MappingWriteByteOrdered for VendorIdSubmessageElement {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        self.value.mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for VendorIdSubmessageElement {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        Ok(Self {
            value: MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::implementation::rtps_udp_psm::mapping_traits::{from_bytes_le, to_bytes_le};

    use super::*;

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
