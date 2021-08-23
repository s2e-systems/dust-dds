use std::io::Write;

use byteorder::ByteOrder;
use rust_rtps_pim::messages::submessage_elements::SerializedDataSubmessageElement;

use crate::serialize::{self, NumberOfBytes, Serialize};

impl<'a> Serialize for SerializedDataSubmessageElement<'a> {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        writer.write_all(self.value)?;
        Ok(())
    }
}

impl<'a> NumberOfBytes for SerializedDataSubmessageElement<'a> {
    fn number_of_bytes(&self) -> usize {
        self.value.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialize::to_bytes_le;

    #[test]
    fn serialize_serialized_data() {
        let data = SerializedDataSubmessageElement { value: &[1, 2] };
        assert_eq!(to_bytes_le(&data).unwrap(), vec![1, 2,]);
    }
}
