use std::io::{Error, Write};

use byteorder::ByteOrder;
use rust_rtps_pim::messages::submessage_elements::SerializedDataSubmessageElement;

use crate::mapping_traits::{MappingWriteByteOrdered, NumberOfBytes};

impl<D> MappingWriteByteOrdered for SerializedDataSubmessageElement<D>
where
    D: AsRef<[u8]>,
{
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        writer.write_all(self.value.as_ref())?;
        Ok(())
    }
}

impl<D> NumberOfBytes for SerializedDataSubmessageElement<D>
where
    D: AsRef<[u8]>,
{
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
        let data = SerializedDataSubmessageElement { value: &[1, 2] };
        assert_eq!(to_bytes_le(&data).unwrap(), vec![1, 2,]);
    }
}
