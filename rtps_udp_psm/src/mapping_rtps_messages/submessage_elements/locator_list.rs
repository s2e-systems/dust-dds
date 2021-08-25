use std::{io::Write, iter::FromIterator};

use byteorder::ByteOrder;
use rust_rtps_pim::{messages::submessage_elements::LocatorListSubmessageElement, structure::types::Locator};

use crate::{
    deserialize::{self, Deserialize},
    serialize::{self, Serialize},
};


impl<T> Serialize for LocatorListSubmessageElement<T>
where
    for<'a> &'a T: IntoIterator<Item = &'a Locator>,
{
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        let locator_list: Vec<&Locator> = self.value.into_iter().collect();
        let num_locators = locator_list.len() as u32;
        num_locators.serialize::<_, B>(&mut writer)?;
        for locator in locator_list {
            locator.serialize::<_, B>(&mut writer)?;
        };
        Ok(())
    }
}

impl<'de, T> Deserialize<'de> for LocatorListSubmessageElement<T>
where
    T: FromIterator<Locator>,
{
    fn deserialize<B: ByteOrder>(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        let num_locators: u32 = Deserialize::deserialize::<B>(buf)?;
        let mut locator_list = Vec::new();
        for _ in 0..num_locators {
            locator_list.push(Deserialize::deserialize::<B>(buf)?);
        };
        Ok(Self{value: T::from_iter(locator_list.into_iter())})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deserialize::from_bytes_le;
    use crate::serialize::to_bytes_le;

    #[test]
    fn serialize_locator_list() {
        let locator_1 = Locator::new(1, 2, [3; 16]);
        let locator_2 = Locator::new(2, 2, [3; 16]);
        let locator_list = LocatorListSubmessageElement{ value: vec![locator_1, locator_2]};
        assert_eq!(to_bytes_le(&locator_list).unwrap(), vec![
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
        ]);
    }

    #[test]
    fn deserialize_locator_list() {
        let locator_1 = Locator::new(1, 2, [3; 16]);
        let locator_2 = Locator::new(2, 2, [3; 16]);
        let expected = LocatorListSubmessageElement{ value: vec![locator_1, locator_2]};
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
