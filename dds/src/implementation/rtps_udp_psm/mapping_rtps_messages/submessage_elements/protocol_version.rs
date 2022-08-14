use std::io::{Error, Write};

use crate::implementation::{
    rtps::messages::submessage_elements::ProtocolVersionSubmessageElement,
    rtps_udp_psm::mapping_traits::{MappingRead, MappingWrite, NumberOfBytes},
};

impl NumberOfBytes for ProtocolVersionSubmessageElement {
    fn number_of_bytes(&self) -> usize {
        2
    }
}

impl MappingWrite for ProtocolVersionSubmessageElement {
    fn mapping_write<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        self.value.mapping_write(&mut writer)
    }
}

impl<'de> MappingRead<'de> for ProtocolVersionSubmessageElement {
    fn mapping_read(buf: &mut &'de [u8]) -> Result<Self, Error> {
        Ok(Self {
            value: MappingRead::mapping_read(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::implementation::rtps_udp_psm::mapping_traits::{from_bytes, to_bytes};

    use super::*;

    #[test]
    fn serialize_protocol_version() {
        let data = ProtocolVersionSubmessageElement { value: [2, 3] };
        assert_eq!(to_bytes(&data).unwrap(), vec![2, 3]);
    }

    #[test]
    fn deserialize_protocol_version() {
        let expected = ProtocolVersionSubmessageElement { value: [2, 3] };
        assert_eq!(expected, from_bytes(&[2, 3]).unwrap());
    }
}
