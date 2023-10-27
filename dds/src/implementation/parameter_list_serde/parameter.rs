use crate::{
    cdr::{error::CdrResult, serialize::CdrSerialize, serializer::CdrSerializer},
    implementation::rtps::messages::types::ParameterId,
    infrastructure::error::DdsResult,
    topic_definition::cdr_type::{CdrDeserialize, CdrDeserializer},
};

#[derive(
    Debug, PartialEq, Eq, Clone, derive_more::From, derive_more::AsRef, derive_more::Constructor,
)]
pub struct Parameter<const PID: ParameterId, T>(T);

#[derive(
    Debug, PartialEq, Eq, Clone, derive_more::From, derive_more::AsRef, derive_more::Constructor,
)]
pub struct ParameterWithDefault<const PID: ParameterId, T>(T);

#[derive(
    Debug, PartialEq, Eq, Clone, derive_more::From, derive_more::AsRef, derive_more::Constructor,
)]
pub struct ParameterVector<const PID: ParameterId, T>(Vec<T>);

impl<const PID: ParameterId, T> CdrSerialize for Parameter<PID, T>
where
    T: CdrSerialize,
{
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> CdrResult<()> {
        let length_without_padding = calc_serialized_data_size(&self.0)? as i16;
        let padding_length = (4 - length_without_padding) & 3;
        let length = length_without_padding + padding_length;
        PID.serialize(serializer)?;
        length.serialize(serializer)?;
        self.0.serialize(serializer)?;

        match padding_length {
            1 => serializer.serialize_array(&[0u8; 1])?,
            2 => serializer.serialize_array(&[0u8; 2])?,
            3 => serializer.serialize_array(&[0u8; 3])?,
            _ => serializer.serialize_array(&[0u8; 0])?,
        }
        Ok(())
    }
}

impl<const PID: ParameterId, T> CdrSerialize for ParameterWithDefault<PID, T>
where
    T: CdrSerialize + Default + PartialEq,
{
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> CdrResult<()> {
        if self.0 == T::default() {
            Ok(())
        } else {
            let length_without_padding = calc_serialized_data_size(&self.0)? as i16;
            let padding_length = (4 - length_without_padding) & 3;
            let length = length_without_padding + padding_length;

            PID.serialize(serializer)?;
            length.serialize(serializer)?;
            self.0.serialize(serializer)?;

            match padding_length {
                1 => serializer.serialize_array(&[0u8; 1])?,
                2 => serializer.serialize_array(&[0u8; 2])?,
                3 => serializer.serialize_array(&[0u8; 3])?,
                _ => serializer.serialize_array(&[0u8; 0])?,
            }
            Ok(())
        }
    }
}

impl<const PID: ParameterId, T> CdrSerialize for ParameterVector<PID, T>
where
    T: CdrSerialize,
{
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> CdrResult<()> {
        if self.0.is_empty() {
            Ok(())
        } else {
            for value in &self.0 {
                let length_without_padding = calc_serialized_data_size(&value)? as i16;
                let padding_length = (4 - length_without_padding) & 3;
                let length = length_without_padding + padding_length;

                PID.serialize(serializer)?;
                length.serialize(serializer)?;
                value.serialize(serializer)?;

                match padding_length {
                    1 => serializer.serialize_array(&[0u8; 1])?,
                    2 => serializer.serialize_array(&[0u8; 2])?,
                    3 => serializer.serialize_array(&[0u8; 3])?,
                    _ => serializer.serialize_array(&[0u8; 0])?,
                }
            }
            Ok(())
        }
    }
}

impl<'de, const PID: ParameterId, T> CdrDeserialize<'de> for Parameter<PID, T>
where
    T: CdrDeserialize<'de>,
{
    fn deserialize(deserializer: &mut impl CdrDeserializer<'de>) -> DdsResult<Self> {
        todo!()
    }

    // fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    // where
    //     D: serde::Deserializer<'de>,
    // {
    //     struct Visitor<'de, const PID: ParameterId, T>
    //     where
    //         T: serde::Deserialize<'de>,
    //     {
    //         marker: PhantomData<Parameter<PID, T>>,
    //         lifetime: PhantomData<&'de ()>,
    //     }
    //     impl<'de, const PID: ParameterId, T> serde::de::Visitor<'de> for Visitor<'de, PID, T>
    //     where
    //         T: serde::Deserialize<'de>,
    //     {
    //         type Value = Parameter<PID, T>;
    //         fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    //             formatter.write_str("struct Parameter")
    //         }

    //         fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    //         where
    //             A: de::MapAccess<'de>,
    //         {
    //             while let Some(key) = map.next_key::<ParameterId>()? {
    //                 if key == PID {
    //                     return Ok(Parameter(map.next_value()?));
    //                 } else if key == PID_SENTINEL {
    //                     break;
    //                 }
    //             }
    //             Err(A::Error::custom(format!("PID {} not found", PID)))
    //         }
    //     }
    //     deserializer.deserialize_newtype_struct(
    //         "Parameter",
    //         Visitor {
    //             marker: PhantomData::<Parameter<PID, T>>,
    //             lifetime: PhantomData,
    //         },
    //     )
    // }
}

impl<'de, const PID: ParameterId, T> CdrDeserialize<'de> for ParameterWithDefault<PID, T>
where
    T: CdrDeserialize<'de> + Default,
{
    fn deserialize(deserializer: &mut impl CdrDeserializer<'de>) -> DdsResult<Self> {
        todo!()
    }

    // fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    // where
    //     D: serde::Deserializer<'de>,
    // {
    //     struct Visitor<'de, const PID: ParameterId, T>
    //     where
    //         T: serde::Deserialize<'de>,
    //     {
    //         marker: PhantomData<ParameterWithDefault<PID, T>>,
    //         lifetime: PhantomData<&'de ()>,
    //     }
    //     impl<'de, const PID: ParameterId, T> serde::de::Visitor<'de> for Visitor<'de, PID, T>
    //     where
    //         T: serde::Deserialize<'de> + Default,
    //     {
    //         type Value = ParameterWithDefault<PID, T>;
    //         fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    //             formatter.write_str("struct ParameterWithDefault")
    //         }

    //         fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    //         where
    //             A: de::MapAccess<'de>,
    //         {
    //             while let Some(key) = map.next_key::<ParameterId>()? {
    //                 if key == PID {
    //                     return Ok(ParameterWithDefault(map.next_value()?));
    //                 } else if key == PID_SENTINEL {
    //                     break;
    //                 }
    //             }
    //             Ok(ParameterWithDefault(T::default()))
    //         }
    //     }
    //     deserializer.deserialize_newtype_struct(
    //         "ParameterWithDefault",
    //         Visitor {
    //             marker: PhantomData::<ParameterWithDefault<PID, T>>,
    //             lifetime: PhantomData,
    //         },
    //     )
    // }
}

impl<'de, const PID: ParameterId, T> CdrDeserialize<'de> for ParameterVector<PID, T>
where
    T: CdrDeserialize<'de>,
{
    fn deserialize(deserializer: &mut impl CdrDeserializer<'de>) -> DdsResult<Self> {
        todo!()
    }

    // fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    // where
    //     D: serde::Deserializer<'de>,
    // {
    //     struct Visitor<'de, const PID: ParameterId, T>
    //     where
    //         T: serde::Deserialize<'de>,
    //     {
    //         marker: PhantomData<ParameterVector<PID, T>>,
    //         lifetime: PhantomData<&'de ()>,
    //     }
    //     impl<'de, const PID: ParameterId, T> serde::de::Visitor<'de> for Visitor<'de, PID, T>
    //     where
    //         T: serde::Deserialize<'de>,
    //     {
    //         type Value = ParameterVector<PID, T>;
    //         fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    //             formatter.write_str("struct ParameterVector")
    //         }

    //         fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    //         where
    //             A: de::MapAccess<'de>,
    //         {
    //             let mut values = vec![];
    //             while let Some(key) = map.next_key::<ParameterId>()? {
    //                 if key == PID {
    //                     values.push(map.next_value()?);
    //                 } else if key == PID_SENTINEL {
    //                     break;
    //                 }
    //             }
    //             Ok(ParameterVector(values))
    //         }
    //     }
    //     deserializer.deserialize_newtype_struct(
    //         "ParameterVector",
    //         Visitor {
    //             marker: PhantomData::<ParameterVector<PID, T>>,
    //             lifetime: PhantomData,
    //         },
    //     )
    // }
}

struct SizeChecker {
    counter: usize,
    pos: usize,
}

impl SizeChecker {
    fn new() -> Self {
        Self { counter: 0, pos: 0 }
    }

    fn size(&self) -> usize {
        self.counter
    }

    fn add_padding_of<T>(&mut self) -> CdrResult<()> {
        let alignment = std::mem::size_of::<T>();
        let rem_mask = alignment - 1; // mask like 0x0, 0x1, 0x3, 0x7
        match self.pos & rem_mask {
            0 => Ok(()),
            n @ 1..=7 => {
                let amt = alignment - n;
                self.add_size(amt)
            }
            _ => unreachable!(),
        }
    }

    fn add_size(&mut self, size: usize) -> CdrResult<()> {
        self.pos += size;
        self.counter += size;
        Ok(())
    }

    fn add_value<T>(&mut self, _v: T) -> CdrResult<()> {
        self.add_padding_of::<T>()?;
        self.add_size(std::mem::size_of::<T>())
    }
}

impl CdrSerializer for SizeChecker {
    fn serialize_bool(&mut self, v: bool) -> CdrResult<()> {
        self.add_value(v as u8)
    }

    fn serialize_i8(&mut self, v: i8) -> CdrResult<()> {
        self.add_value(v)
    }

    fn serialize_i16(&mut self, v: i16) -> CdrResult<()> {
        self.add_value(v)
    }

    fn serialize_i32(&mut self, v: i32) -> CdrResult<()> {
        self.add_value(v)
    }

    fn serialize_i64(&mut self, v: i64) -> CdrResult<()> {
        self.add_value(v)
    }

    fn serialize_u8(&mut self, v: u8) -> CdrResult<()> {
        self.add_value(v)
    }

    fn serialize_u16(&mut self, v: u16) -> CdrResult<()> {
        self.add_value(v)
    }

    fn serialize_u32(&mut self, v: u32) -> CdrResult<()> {
        self.add_value(v)
    }

    fn serialize_u64(&mut self, v: u64) -> CdrResult<()> {
        self.add_value(v)
    }

    fn serialize_f32(&mut self, v: f32) -> CdrResult<()> {
        self.add_value(v)
    }

    fn serialize_f64(&mut self, v: f64) -> CdrResult<()> {
        self.add_value(v)
    }

    fn serialize_char(&mut self, _v: char) -> CdrResult<()> {
        self.add_size(1)
    }

    fn serialize_str(&mut self, v: &str) -> CdrResult<()> {
        self.add_value(0_u32)?;
        self.add_size(v.len() + 1) // adds the length 1 of a terminating character
    }

    fn serialize_seq(&mut self, v: &[impl CdrSerialize]) -> CdrResult<()> {
        let len = v.len() as u32;
        self.add_value(len)?;
        for element in v {
            element.serialize(self)?;
        }
        Ok(())
    }

    fn serialize_array<const N: usize>(&mut self, v: &[impl CdrSerialize; N]) -> CdrResult<()> {
        for element in v {
            element.serialize(self)?;
        }
        Ok(())
    }

    fn serialize_unit(&mut self) -> CdrResult<()> {
        Ok(())
    }
}

fn calc_serialized_data_size(value: &impl CdrSerialize) -> CdrResult<usize> {
    let mut size_checker = SizeChecker::new();
    value.serialize(&mut size_checker)?;
    Ok(size_checker.size())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn serialize_data_size<T>(v: &T) -> CdrResult<usize>
    where
        T: CdrSerialize + ?Sized,
    {
        let mut serializer = SizeChecker::new();
        v.serialize(&mut serializer)?;
        Ok(serializer.size())
    }

    #[test]
    fn serialize_octet() {
        let v = 32u8;
        assert_eq!(serialize_data_size(&v).unwrap(), 1);
    }

    #[test]
    fn serialize_char() {
        let v = 'Z';
        assert_eq!(serialize_data_size(&v).unwrap(), 1);
    }

    #[test]
    fn serialize_ushort() {
        let v = 65500u16;
        assert_eq!(serialize_data_size(&v).unwrap(), 2);
    }

    #[test]
    fn serialize_short() {
        let v = -32700i16;
        assert_eq!(serialize_data_size(&v).unwrap(), 2);
    }

    #[test]
    fn serialize_ulong() {
        let v = 4294967200u32;
        assert_eq!(serialize_data_size(&v).unwrap(), 4);
    }

    #[test]
    fn serialize_long() {
        let v = -2147483600i32;
        assert_eq!(serialize_data_size(&v).unwrap(), 4);
    }

    #[test]
    fn serialize_ulonglong() {
        let v = 18446744073709551600u64;
        assert_eq!(serialize_data_size(&v).unwrap(), 8);
    }

    #[test]
    fn serialize_longlong() {
        let v = -9223372036800i64;
        assert_eq!(serialize_data_size(&v).unwrap(), 8);
    }

    #[test]
    fn serialize_float() {
        let v = std::f32::MIN_POSITIVE;
        assert_eq!(serialize_data_size(&v).unwrap(), 4);
    }

    #[test]
    fn serialize_double() {
        let v = std::f64::MIN_POSITIVE;
        assert_eq!(serialize_data_size(&v).unwrap(), 8);
    }

    #[test]
    fn serialize_bool() {
        let v = true;
        assert_eq!(serialize_data_size(&v).unwrap(), 1);
    }

    #[test]
    fn serialize_string() {
        let v = "ABCDE";
        assert_eq!(serialize_data_size(&v).unwrap(), 10);
    }

    #[test]
    fn serialize_empty_string() {
        let v = "";
        assert_eq!(serialize_data_size(&v).unwrap(), 5);
    }

    #[test]
    fn serialize_octet_array() {
        let v = [1u8, 2, 3, 4, 5];
        assert_eq!(serialize_data_size(&v).unwrap(), 5);
    }

    #[test]
    fn serialize_char_array() {
        let v = ['A', 'B', 'C', 'D', 'E'];
        assert_eq!(serialize_data_size(&v).unwrap(), 5);
    }

    #[test]
    fn serialize_ushort_array() {
        let v = [65500u16, 65501, 65502, 65503, 65504];
        assert_eq!(serialize_data_size(&v).unwrap(), 10);
    }

    #[test]
    fn serialize_short_array() {
        let v = [-32700i16, -32701, -32702, -32703, -32704];
        assert_eq!(serialize_data_size(&v).unwrap(), 10);
    }

    #[test]
    fn serialize_ulong_array() {
        let v = [
            4294967200u32,
            4294967201,
            4294967202,
            4294967203,
            4294967204,
        ];
        assert_eq!(serialize_data_size(&v).unwrap(), 20);
    }

    #[test]
    fn serialize_long_array() {
        let v = [
            -2147483600,
            -2147483601,
            -2147483602,
            -2147483603,
            -2147483604,
        ];
        assert_eq!(serialize_data_size(&v).unwrap(), 20);
    }

    #[test]
    fn serialize_ulonglong_array() {
        let v = [
            18446744073709551600u64,
            18446744073709551601,
            18446744073709551602,
            18446744073709551603,
            18446744073709551604,
        ];
        assert_eq!(serialize_data_size(&v).unwrap(), 40);
    }

    #[test]
    fn serialize_longlong_array() {
        let v = [
            -9223372036800i64,
            -9223372036801,
            -9223372036802,
            -9223372036803,
            -9223372036804,
        ];
        assert_eq!(serialize_data_size(&v).unwrap(), 40);
    }

    #[test]
    fn serialize_float_array() {
        let f = std::f32::MIN_POSITIVE;

        let v = [f, f + 1., f + 2., f + 3., f + 4.];
        assert_eq!(serialize_data_size(&v).unwrap(), 20);
    }

    #[test]
    fn serialize_double_array() {
        let f = std::f64::MIN_POSITIVE;

        let v = [f, f + 1., f + 2., f + 3., f + 4.];
        assert_eq!(serialize_data_size(&v).unwrap(), 40);
    }

    #[test]
    fn serialize_bool_array() {
        let v = [true, false, true, false, true];
        assert_eq!(serialize_data_size(&v).unwrap(), 5);
    }
}
