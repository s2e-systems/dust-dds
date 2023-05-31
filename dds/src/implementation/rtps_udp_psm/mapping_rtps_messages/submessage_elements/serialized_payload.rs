use crate::implementation::{
    rtps::messages::types::SerializedPayload,
    rtps_udp_psm::mapping_traits::{MappingWriteByteOrdered, NumberOfBytes},
};
use byteorder::ByteOrder;
use std::io::{Error, Write};

impl MappingWriteByteOrdered for SerializedPayload {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        // writer.write_all(self.into())?;
        // Ok(())
        todo!()
    }
}

impl NumberOfBytes for SerializedPayload {
    fn number_of_bytes(&self) -> usize {
        //<&[u8]>::from(self).len()
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::rtps_udp_psm::mapping_traits::to_bytes_le;

    #[test]
    fn serialize_serialized_data() {
        let data = SerializedPayload::new(&[1, 2]);
        assert_eq!(to_bytes_le(&data).unwrap(), vec![1, 2,]);
    }
}
