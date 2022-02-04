use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::{
    mapping_traits::{MappingWriteByteOrdered, NumberOfBytes},
    messages::submessage_elements::SerializedDataSubmessageElementPsm,
};

impl MappingWriteByteOrdered for SerializedDataSubmessageElementPsm<'_> {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        writer.write_all(self.value.as_ref())?;
        Ok(())
    }
}

impl NumberOfBytes for SerializedDataSubmessageElementPsm<'_> {
    fn number_of_bytes(&self) -> usize {
        self.value.as_ref().len()
    }
}

#[cfg(test)]
mod tests {
    use crate::mapping_traits::to_bytes_le;

    use super::*;

    #[test]
    fn serialize_serialized_data() {
        let data = SerializedDataSubmessageElementPsm { value: &[1, 2] };
        assert_eq!(to_bytes_le(&data).unwrap(), vec![1, 2,]);
    }
}
