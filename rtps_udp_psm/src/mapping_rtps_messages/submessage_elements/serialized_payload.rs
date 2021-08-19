use std::io::Write;

use byteorder::ByteOrder;
use rust_rtps_pim::messages::submessage_elements::SerializedDataSubmessageElement;

use crate::{deserialize::{self, Deserialize}, serialize::{self, NumberofBytes, Serialize}};

impl<'a> Serialize for SerializedDataSubmessageElement<'a> {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        self.value.serialize::<_, B>(&mut writer)
    }
}

// impl<'de:'a, 'a> Deserialize<'de> for SerializedDataSubmessageElement<'a> {
//     fn deserialize<B: ByteOrder>(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
//         Ok(Self {
//             value: Deserialize::deserialize::<B>(buf)?,
//         })
//     }
// }

impl<'a> NumberofBytes for  SerializedDataSubmessageElement<'a> {
    fn number_of_bytes(&self) -> usize {
        self.value.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deserialize::from_bytes_le;
    use crate::serialize::to_bytes_le;

    #[test]
    fn serialize_serialized_data() {
        let data = SerializedDataSubmessageElement { value: &[1, 2] };
        assert_eq!(to_bytes_le(&data).unwrap(), vec![1, 2,]);
    }

    // #[test]
    // fn deserialize_serialized_data() {
    //     let expected = SerializedDataSubmessageElement { value: [1, 2] };
    //     assert_eq!(expected, from_bytes_le(&[1, 2,]).unwrap());
    // }
}
