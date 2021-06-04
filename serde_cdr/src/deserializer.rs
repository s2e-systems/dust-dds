use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Read;


pub struct Deserializer<R> {
    reader: R,
}

impl<'de, 'a, R: Read> serde::de::Deserializer<'de> for &'a mut Deserializer<R> {
    type Error = crate::error::Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let value: u8 = serde::de::Deserialize::deserialize(self)?;
        match value {
            1 => visitor.visit_bool(true),
            0 => visitor.visit_bool(false),
            value => Err(Self::Error::InvalidBoolEncoding(value)),
        }
    }

    fn deserialize_i8<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_i16<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_i32(self.reader.read_i32::<LittleEndian>()?)
    }

    fn deserialize_i64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_u8(self.reader.read_u8()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_u16(self.reader.read_u16::<LittleEndian>()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_u32(self.reader.read_u32::<LittleEndian>()?)
    }

    fn deserialize_u64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_str<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_string<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let len: u32 = serde::de::Deserialize::deserialize(&mut *self)?;
        self.deserialize_tuple(len as usize, visitor)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        struct Access<'a, R: Read + 'a> {
            deserializer: &'a mut Deserializer<R>,
            remaining_items: usize,
            len: usize
        }

        impl<'de, 'a, R: Read + 'a> serde::de::SeqAccess<'de> for Access<'a, R>{
            type Error = crate::error::Error;

            fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
                where T: serde::de::DeserializeSeed<'de>,
            {
                if self.remaining_items > 0 {
                    self.remaining_items -= 1;
                    let value = serde::de::DeserializeSeed::deserialize(seed, &mut *self.deserializer)?;
                    Ok(Some(value))
                } else {
                    Ok(None)
                }
            }

            fn size_hint(&self) -> Option<usize> {
                Some(self.len)
            }
        }
        visitor.visit_seq(Access{deserializer: self, remaining_items: len, len})
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        struct Access<'a, R: Read + 'a> {
            deserializer: &'a mut Deserializer<R>,
            len: usize,
        }

        impl<'de, 'a, R: Read + 'a> serde::de::SeqAccess<'de> for Access<'a, R>{
            type Error = crate::error::Error;

            fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
                where T: serde::de::DeserializeSeed<'de>,
            {
                if self.len > 0 {
                    self.len -= 1;
                    let value = serde::de::DeserializeSeed::deserialize(seed, &mut *self.deserializer)?;
                    Ok(Some(value))
                } else {
                    Ok(None)
                }
            }

            fn size_hint(&self) -> Option<usize> {
                Some(self.len)
            }
        }

        let len = fields.len();
        visitor.visit_seq(Access{deserializer: self, len,})
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deserialize<'de, T: serde::Deserialize<'de>, const N: usize>(buffer: [u8; N]) -> T {
        let mut de = Deserializer {
            reader: buffer.as_ref(),
        };
        serde::de::Deserialize::deserialize(&mut de).unwrap()
    }
    #[test]
    fn deserialize_u8() {
        let result: u8 = deserialize([1]);
        assert_eq!(result, 1);
    }

    #[test]
    fn deserialize_multiple_u8() {
        let buffer = [1, 2];
        let mut de = Deserializer {
            reader: buffer.as_ref(),
        };
        let result: u8 = serde::de::Deserialize::deserialize(&mut de).unwrap();
        assert_eq!(result, 1);
        let result: u8 = serde::de::Deserialize::deserialize(&mut de).unwrap();
        assert_eq!(result, 2);
    }

    #[test]
    fn deserialize_u16() {
        let result: u16 = deserialize([0x03, 0x42]);
        assert_eq!(result, 0x4203);
    }

    #[test]
    fn deserialize_bool() {
        let result: bool = deserialize([1]);
        assert_eq!(result, true);
        let result: bool = deserialize([0]);
        assert_eq!(result, false);
    }

    #[test]
    #[should_panic]
    fn deserialize_bool_invalid() {
        let _result: bool = deserialize([2]);
    }

    #[test]
    fn deserialize_tuple() {
        let result: (u8, u8) = deserialize([0x03, 0x42]);
        assert_eq!(result, (0x03, 0x42));
    }

    #[test]
    fn deserialze_sequence() {
        let result: Vec<u8> = deserialize([0x02, 0x00, 1, 2]);
        assert_eq!(result, vec![1, 2]);
    }

    #[derive(PartialEq, Debug, serde::Deserialize)]
    struct PrimitiveStruct {
        id: u16,
        flags: u8,
    }

    #[test]
    fn deserialze_primitive_struct() {
        let result: PrimitiveStruct = deserialize([0x03, 0x00, 0b00000001]);
        assert_eq!(result, PrimitiveStruct{id: 0x0003, flags: 0b00000001});
    }

    #[derive(PartialEq, Debug, serde::Deserialize)]
    struct ComplexStruct {
        header: PrimitiveStruct,
        data: [u16; 2],
    }

    #[test]
    fn deserialze_complex_struct() {
        let result: ComplexStruct = deserialize([0x03, 0x00, 0b00000001, 0x04, 0x00, 0x05, 0x00]);
        assert_eq!(result, ComplexStruct{header: PrimitiveStruct{id: 0x0003, flags: 0b00000001}, data: [0x0004, 0x0005]});
    }


    struct CustomStructVisitor;

    impl<'de> serde::de::Visitor<'de> for CustomStructVisitor {
        type Value = CustomStruct;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a CustomStruct struct")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where A: serde::de::SeqAccess<'de>,
        {
            let data_length: u16 = seq.next_element()?.ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
            let mut data: [u8; 32] = seq.next_element()?.ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
            // let data = [1; N];
            //Ok(CustomStruct{data_length, data})
            todo!()
        }
    }

    fn deserialize_vecu8<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: serde::de::Deserializer<'de>,
        &'de D: serde::de::Deserializer<'de> + 'de
    {
        struct MaxVisitor;

        impl<'de> serde::de::Visitor<'de> for MaxVisitor
        {
            /// Return type of this visitor. This visitor computes the max of a
            /// sequence of values of type T, so the type of the maximum is T.
            type Value = Vec<u8>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a nonempty sequence of numbers")
            }

            fn visit_seq<S>(self, mut seq: S) -> Result<Vec<u8>, S::Error>
            where
                S: serde::de::SeqAccess<'de>,
            {
                // Start with max equal to the first value in the seq.
                let mut max = seq.next_element()?.ok_or_else(||
                    // Cannot take the maximum of an empty seq.
                    serde::de::Error::custom("no values in seq when looking for maximum")
                )?;

                // Update the max while there are additional values.
                while let Some(value) = seq.next_element()? {
                    max = std::cmp::max(max, value);
                }

                Ok(max)
            }
        }

        // Create the visitor and ask the deserializer to drive it. The
        // deserializer will call visitor.visit_seq() if a seq is present in
        // the input data.
        let visitor = MaxVisitor;

        let len: u16 = serde::de::Deserialize::deserialize(&deserializer).unwrap();
        deserializer.deserialize_seq(visitor)
    }

    #[derive(PartialEq, Debug, serde::Deserialize)]
    struct CustomStruct {
        data_length: u16,
        //#[serde(deserialize_with = "deserialize_vecu8")]
        data: Vec<u8>,
    }
    // impl<'de> serde::Deserialize<'de> for CustomStruct {
    //     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    //     where
    //         D: serde::Deserializer<'de> {
    //             const FIELDS: &'static [&'static str] = &["data_length", "data"];
    //             deserializer.deserialize_struct("CustomStruct", FIELDS, CustomStructVisitor {})
    //     }
    // }

    #[test]
    fn deserialze_custom_struct() {
        let result: CustomStruct = deserialize([0x02, 0x00, 0x00, 0x00, 4, 5, 2]);
        let mut expected_data = vec![5];
        assert_eq!(result, CustomStruct{data_length: 2, data: expected_data});
    }
}
