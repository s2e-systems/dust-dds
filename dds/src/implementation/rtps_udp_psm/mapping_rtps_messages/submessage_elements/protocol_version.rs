use std::io::{Error, Write};

use byteorder::{ByteOrder};

use crate::implementation::{
    rtps::messages::submessage_elements::ProtocolVersionSubmessageElement,
    rtps_udp_psm::mapping_traits::{
        MappingReadByteOrdered, MappingWriteByteOrdered,
        NumberOfBytes,
    },
};

impl NumberOfBytes for ProtocolVersionSubmessageElement {
    fn number_of_bytes(&self) -> usize {
        2
    }
}

impl MappingWriteByteOrdered for ProtocolVersionSubmessageElement {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        self.value.mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for ProtocolVersionSubmessageElement {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        Ok(Self {
            value: MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::implementation::rtps_udp_psm::mapping_traits::{to_bytes_le, from_bytes_le};

    use super::*;

    #[test]
    fn serialize_protocol_version() {
        let data = ProtocolVersionSubmessageElement { value: [2, 3] };
        assert_eq!(to_bytes_le(&data).unwrap(), vec![2, 3]);
    }

    #[test]
    fn deserialize_protocol_version() {
        let expected = ProtocolVersionSubmessageElement { value: [2, 3] };
        assert_eq!(expected, from_bytes_le(&[2, 3]).unwrap());
    }
}
