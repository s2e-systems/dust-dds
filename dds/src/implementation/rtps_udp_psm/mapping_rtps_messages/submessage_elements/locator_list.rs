use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::implementation::{
    rtps::messages::submessage_elements::LocatorList,
    rtps_udp_psm::mapping_traits::{
        MappingReadByteOrdered, MappingWriteByteOrdered, NumberOfBytes,
    },
};

impl MappingWriteByteOrdered for LocatorList {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        let num_locators = self.value.len() as u32;
        num_locators.mapping_write_byte_ordered::<_, B>(&mut writer)?;
        for locator in self.value.iter() {
            locator.mapping_write_byte_ordered::<_, B>(&mut writer)?;
        }
        Ok(())
    }
}

impl<'de> MappingReadByteOrdered<'de> for LocatorList {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        let num_locators: u32 = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let mut locator_list = Vec::new();
        for _ in 0..num_locators {
            locator_list.push(MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?);
        }
        Ok(Self {
            value: locator_list,
        })
    }
}

impl NumberOfBytes for LocatorList {
    fn number_of_bytes(&self) -> usize {
        let num_locators_byte_size = 4;
        if self.value.is_empty() {
            num_locators_byte_size
        } else {
            self.value.len() * self.value[0].number_of_bytes() + num_locators_byte_size
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::implementation::{
        rtps::types::{Locator, LocatorAddress, LocatorKind, LocatorPort},
        rtps_udp_psm::mapping_traits::{from_bytes_le, to_bytes_le},
    };

    use super::*;

    #[test]
    fn serialize_locator_list() {
        let locator_1 = Locator::new(
            LocatorKind::new(1),
            LocatorPort::new(2),
            LocatorAddress::new([3; 16]),
        );
        let locator_2 = Locator::new(
            LocatorKind::new(2),
            LocatorPort::new(2),
            LocatorAddress::new([3; 16]),
        );
        let locator_list = LocatorList {
            value: vec![locator_1, locator_2],
        };
        assert_eq!(
            to_bytes_le(&locator_list).unwrap(),
            vec![
                2, 0, 0, 0, // numLocators (unsigned long)
                1, 0, 0, 0, // kind (long)
                2, 0, 0, 0, // port (unsigned long)
                3, 3, 3, 3, // address (octet[16])
                3, 3, 3, 3, // address (octet[16])
                3, 3, 3, 3, // address (octet[16])
                3, 3, 3, 3, // address (octet[16])
                2, 0, 0, 0, // kind (long)
                2, 0, 0, 0, // port (unsigned long)
                3, 3, 3, 3, // address (octet[16])
                3, 3, 3, 3, // address (octet[16])
                3, 3, 3, 3, // address (octet[16])
                3, 3, 3, 3, // address (octet[16])
            ]
        );
    }

    #[test]
    fn deserialize_locator_list() {
        let locator_1 = Locator::new(
            LocatorKind::new(1),
            LocatorPort::new(2),
            LocatorAddress::new([3; 16]),
        );
        let locator_2 = Locator::new(
            LocatorKind::new(2),
            LocatorPort::new(2),
            LocatorAddress::new([3; 16]),
        );
        let expected = LocatorList {
            value: vec![locator_1, locator_2],
        };
        #[rustfmt::skip]
        let result = from_bytes_le(&[
            2, 0, 0, 0,  // numLocators (unsigned long)
            1, 0, 0, 0, // kind (long)
            2, 0, 0, 0, // port (unsigned long)
            3, 3, 3, 3, // address (octet[16])
            3, 3, 3, 3, // address (octet[16])
            3, 3, 3, 3, // address (octet[16])
            3, 3, 3, 3, // address (octet[16])
            2, 0, 0, 0, // kind (long)
            2, 0, 0, 0, // port (unsigned long)
            3, 3, 3, 3, // address (octet[16])
            3, 3, 3, 3, // address (octet[16])
            3, 3, 3, 3, // address (octet[16])
            3, 3, 3, 3, // address (octet[16])

        ]).unwrap();
        assert_eq!(expected, result);
    }
}
