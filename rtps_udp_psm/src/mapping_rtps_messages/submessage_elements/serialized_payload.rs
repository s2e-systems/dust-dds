use std::io::{Error, Write};

use byteorder::ByteOrder;
use rtps_pim::messages::submessage_elements::SerializedDataSubmessageElement;

use crate::mapping_traits::{MappingWriteByteOrdered, NumberOfBytes};

impl MappingWriteByteOrdered for SerializedDataSubmessageElement<'_> {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        writer.write_all(self.value)?;
        Ok(())
    }
}

impl NumberOfBytes for SerializedDataSubmessageElement<'_> {
    fn number_of_bytes(&self) -> usize {
        self.value.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::mapping_traits::to_bytes_le;

    use super::*;

    #[test]
    fn serialize_serialized_data() {
        let data = SerializedDataSubmessageElement { value: &[1, 2][..] };
        assert_eq!(to_bytes_le(&data).unwrap(), vec![1, 2,]);
    }
}
