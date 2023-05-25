use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::implementation::{
    rtps::messages::types::SerializedPayload,
    rtps_udp_psm::mapping_traits::{MappingWriteByteOrdered, NumberOfBytes, MappingReadByteOrdered},
};

impl<'de> MappingReadByteOrdered<'de> for SerializedPayload<'de> {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        Ok(SerializedPayload::new(buf))
    }
}

impl MappingWriteByteOrdered for SerializedPayload<'_> {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        writer.write_all(self.into())?;
        Ok(())
    }
}

impl NumberOfBytes for SerializedPayload<'_> {
    fn number_of_bytes(&self) -> usize {
        <&[u8]>::from(self).len()
    }
}

#[cfg(test)]
mod tests {

    use crate::implementation::{
        rtps::messages::types::SerializedPayload, rtps_udp_psm::mapping_traits::to_bytes_le,
    };

    #[test]
    fn serialize_serialized_data() {
        let data = SerializedPayload::new(&[1, 2]);
        assert_eq!(to_bytes_le(&data).unwrap(), vec![1, 2,]);
    }
}
