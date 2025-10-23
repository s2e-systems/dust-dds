use crate::xtypes::{
    data_representation::{endianness::EndiannessRead, read_write::Read},
    dynamic_type::{DynamicData, DynamicDataFactory, DynamicType, ExtensibilityKind},
    error::XTypesResult,
};

use super::{deserializer::XTypesDeserializer, dynamic_type::TypeKind, error::XTypesError};
use core::str;

pub fn deserialize_nested<'de>(
    dynamic_type: DynamicType,
    deserializer: &mut impl XTypesDeserializer<'de>,
) -> XTypesResult<DynamicData> {
    let mut dynamic_data = DynamicDataFactory::create_data(dynamic_type);
    match dynamic_data.type_ref().get_kind() {
        TypeKind::NONE => todo!(),
        TypeKind::BOOLEAN => todo!(),
        TypeKind::BYTE => todo!(),
        TypeKind::INT16 => todo!(),
        TypeKind::INT32 => todo!(),
        TypeKind::INT64 => todo!(),
        TypeKind::UINT16 => todo!(),
        TypeKind::UINT32 => todo!(),
        TypeKind::UINT64 => todo!(),
        TypeKind::FLOAT32 => todo!(),
        TypeKind::FLOAT64 => todo!(),
        TypeKind::FLOAT128 => todo!(),
        TypeKind::INT8 => todo!(),
        TypeKind::UINT8 => todo!(),
        TypeKind::CHAR8 => todo!(),
        TypeKind::CHAR16 => todo!(),
        TypeKind::STRING8 => todo!(),
        TypeKind::STRING16 => todo!(),
        TypeKind::ALIAS => todo!(),
        TypeKind::ENUM => todo!(),
        TypeKind::BITMASK => todo!(),
        TypeKind::ANNOTATION => todo!(),
        TypeKind::STRUCTURE => match dynamic_data.type_ref().get_descriptor().extensibility_kind {
            ExtensibilityKind::Final => {
                deserializer.deserialize_final_struct(&mut dynamic_data)?;
            }
            ExtensibilityKind::Appendable => todo!(),
            ExtensibilityKind::Mutable => {
                todo!()
                // let mut mutable_deserializer = deserializer.deserialize_mutable_struct()?;
                // for field_index in 0..dynamic_data.type_ref().get_member_count() {
                //     let type_member = dynamic_data.type_ref().get_member_by_index(field_index)?;
                //     let member_value =
                //         mutable_deserializer.deserialize_field(type_member.get_descriptor()?)?;
                //     let member_id = type_member.get_id();
                //     dynamic_data.set_value(member_id, member_value);
                // }
            }
        },
        TypeKind::UNION => todo!(),
        TypeKind::BITSET => todo!(),
        TypeKind::SEQUENCE => todo!(),
        TypeKind::ARRAY => todo!(),
        TypeKind::MAP => todo!(),
    }

    Ok(dynamic_data)
}

const PID_SENTINEL: u16 = 1;

pub struct Xcdr1Deserializer<'a, E> {
    reader: Reader<'a>,
    endianness: E,
}

impl<'a, E> Xcdr1Deserializer<'a, E> {
    pub fn new(buffer: &'a [u8], endianness: E) -> Self {
        Self {
            reader: Reader::new(buffer),
            endianness,
        }
    }
}

pub struct Xcdr2Deserializer<'a, E> {
    reader: Reader<'a>,
    endianness: E,
}

impl<'a, E> Xcdr2Deserializer<'a, E> {
    pub fn new(buffer: &'a [u8], endianness: E) -> Self {
        Self {
            reader: Reader::new(buffer),
            endianness,
        }
    }
}

struct Reader<'a> {
    buffer: &'a [u8],
    pos: usize,
}

impl<'a> Read for Reader<'a> {
    fn read_exact(&mut self, size: usize) -> XTypesResult<&[u8]> {
        self.read_all(size)
    }
}

impl<'a> Reader<'a> {
    fn new(buffer: &'a [u8]) -> Self {
        Self { buffer, pos: 0 }
    }
    fn buffer(&self) -> &'a [u8] {
        &self.buffer[self.pos..]
    }
    fn read<const N: usize>(&mut self) -> Result<&'a [u8; N], XTypesError> {
        if self.pos + N > self.buffer.len() {
            return Err(XTypesError::InvalidData);
        }
        let ret = core::convert::TryFrom::try_from(&self.buffer[self.pos..self.pos + N])
            .expect("length guaranteed");
        self.pos += N;
        Ok(ret)
    }
    fn read_all(&mut self, length: usize) -> Result<&'a [u8], XTypesError> {
        if self.pos + length > self.buffer.len() {
            return Err(XTypesError::InvalidData);
        }
        let ret = &self.buffer[self.pos..self.pos + length];
        self.pos += length;
        Ok(ret)
    }

    fn seek(&mut self, v: usize) {
        self.pos += v
    }

    fn seek_padding(&mut self, alignment: usize) {
        let mask = alignment - 1;
        self.seek(((self.pos + mask) & !mask) - self.pos)
    }
}

fn read_with_padding_v1<const N: usize>(reader: &mut Reader) -> Result<[u8; N], XTypesError> {
    reader.seek_padding(N);
    reader.read().cloned()
}

fn read_with_padding_v2<const N: usize>(reader: &mut Reader) -> Result<[u8; N], XTypesError> {
    reader.seek_padding(core::cmp::min(N, 4));
    reader.read().cloned()
}

fn seek_to_pid_be(reader: &mut Reader, pid: u16) -> Result<(), XTypesError> {
    loop {
        let current_pid = u16::from_be_bytes(*reader.read()?);
        let length = u16::from_be_bytes(*reader.read()?) as usize;
        if current_pid == pid {
            return Ok(());
        } else if current_pid == PID_SENTINEL {
            return Err(XTypesError::PidNotFound(pid));
        } else {
            reader.seek(length);
            reader.seek_padding(4);
        }
    }
}

fn seek_to_pid_le(reader: &mut Reader, pid: u16) -> Result<(), XTypesError> {
    loop {
        let current_pid = u16::from_le_bytes(*reader.read()?);
        let length = u16::from_le_bytes(*reader.read()?) as usize;
        if current_pid == pid {
            return Ok(());
        } else if current_pid == PID_SENTINEL {
            return Err(XTypesError::PidNotFound(pid));
        } else {
            reader.seek(length);
            reader.seek_padding(4);
        }
    }
}

fn seek_to_optional_pid_be(reader: &mut Reader, pid: u16) -> Result<bool, XTypesError> {
    loop {
        let current_pid = u16::from_be_bytes(*reader.read()?);
        let length = u16::from_be_bytes(*reader.read()?) as usize;
        if current_pid == pid {
            return Ok(true);
        } else if current_pid == PID_SENTINEL {
            return Ok(false);
        } else {
            reader.seek(length);
            reader.seek_padding(4);
        }
    }
}

fn seek_to_optional_pid_le(reader: &mut Reader, pid: u16) -> Result<bool, XTypesError> {
    loop {
        let current_pid = u16::from_le_bytes(*reader.read()?);
        let length = u16::from_le_bytes(*reader.read()?) as usize;
        if current_pid == pid {
            return Ok(true);
        } else if current_pid == PID_SENTINEL {
            return Ok(false);
        } else {
            reader.seek(length);
            reader.seek_padding(4);
        }
    }
}

impl<'de, E: EndiannessRead> XTypesDeserializer<'de> for Xcdr1Deserializer<'de, E> {
    fn deserialize_final_struct(&mut self, v: &mut DynamicData) -> Result<(), XTypesError> {
        for field_index in 0..v.type_ref().get_member_count() {
            let type_member = v.type_ref().get_member_by_index(field_index)?;
            match type_member.get_descriptor()?.r#type.get_kind() {
                TypeKind::NONE => todo!(),
                TypeKind::BOOLEAN => {
                    let value = E::read_bool(&mut self.reader)?;
                    v.set_boolean_value(type_member.get_id(), value)?;
                }
                TypeKind::BYTE => todo!(),
                TypeKind::INT16 => todo!(),
                TypeKind::INT32 => todo!(),
                TypeKind::INT64 => todo!(),
                TypeKind::UINT16 => {
                    self.reader.seek_padding(2);
                    let value = E::read_u16(&mut self.reader)?;
                    v.set_uint16_value(type_member.get_id(), value)?;
                }
                TypeKind::UINT32 => {
                    self.reader.seek_padding(4);
                    let value = E::read_u32(&mut self.reader)?;
                    v.set_uint32_value(type_member.get_id(), value)?;
                }
                TypeKind::UINT64 => {
                    self.reader.seek_padding(8);
                    let value = E::read_u64(&mut self.reader)?;
                    v.set_uint64_value(type_member.get_id(), value)?;
                }
                TypeKind::FLOAT32 => {
                    self.reader.seek_padding(4);
                    let value = E::read_f32(&mut self.reader)?;
                    v.set_float32_value(type_member.get_id(), value)?;
                }
                TypeKind::FLOAT64 => {
                    self.reader.seek_padding(8);
                    let value = E::read_f64(&mut self.reader)?;
                    v.set_float64_value(type_member.get_id(), value)?;
                }
                TypeKind::FLOAT128 => todo!(),
                TypeKind::INT8 => {
                    let value = E::read_i8(&mut self.reader)?;
                    v.set_int8_value(type_member.get_id(), value)?;
                }
                TypeKind::UINT8 => {
                    let value = E::read_u8(&mut self.reader)?;
                    v.set_uint8_value(type_member.get_id(), value)?;
                }
                TypeKind::CHAR8 => todo!(),
                TypeKind::CHAR16 => todo!(),
                TypeKind::STRING8 => {
                    self.reader.seek_padding(4);
                    let length = E::read_u32(&mut self.reader)? as usize;
                    let bytes = self.reader.read_all(length - 1)?.to_vec();
                    let _ = self.reader.read_exact(1); // Read the terminating 0
                    let value = String::from_utf8(bytes).map_err(|_| XTypesError::InvalidData)?;
                    v.set_string_value(type_member.get_id(), value)?;
                }
                TypeKind::STRING16 => todo!(),
                TypeKind::ALIAS => todo!(),
                TypeKind::ENUM => todo!(),
                TypeKind::BITMASK => todo!(),
                TypeKind::ANNOTATION => todo!(),
                TypeKind::STRUCTURE => {
                    let mut value = DynamicDataFactory::create_data(
                        type_member.get_descriptor()?.r#type.clone(),
                    );
                    self.deserialize_final_struct(&mut value)?;
                    v.set_complex_value(type_member.get_id(), value)?;
                }
                TypeKind::UNION => todo!(),
                TypeKind::BITSET => todo!(),
                TypeKind::SEQUENCE => {
                    self.reader.seek_padding(4);
                    let length = E::read_u32(&mut self.reader)? as usize;

                    match type_member
                        .get_descriptor()?
                        .r#type
                        .get_descriptor()
                        .element_type
                        .as_ref()
                        .unwrap()
                        .get_kind()
                    {
                        TypeKind::NONE => todo!(),
                        TypeKind::BOOLEAN => {
                            let mut value = Vec::with_capacity(length);
                            for _ in 0..length {
                                value.push(E::read_bool(&mut self.reader)?);
                            }
                            v.set_boolean_values(type_member.get_id(), value)?;
                        }
                        TypeKind::BYTE => todo!(),
                        TypeKind::INT16 => todo!(),
                        TypeKind::INT32 => todo!(),
                        TypeKind::INT64 => todo!(),
                        TypeKind::UINT16 => todo!(),
                        TypeKind::UINT32 => todo!(),
                        TypeKind::UINT64 => todo!(),
                        TypeKind::FLOAT32 => todo!(),
                        TypeKind::FLOAT64 => todo!(),
                        TypeKind::FLOAT128 => todo!(),
                        TypeKind::INT8 => todo!(),
                        TypeKind::UINT8 => {
                            let value = self.reader.read_exact(length as usize)?.to_vec();
                            v.set_uint8_values(type_member.get_id(), value)?;
                        }
                        TypeKind::CHAR8 => todo!(),
                        TypeKind::CHAR16 => todo!(),
                        TypeKind::STRING8 => todo!(),
                        TypeKind::STRING16 => todo!(),
                        TypeKind::ALIAS => todo!(),
                        TypeKind::ENUM => todo!(),
                        TypeKind::BITMASK => todo!(),
                        TypeKind::ANNOTATION => todo!(),
                        TypeKind::STRUCTURE => todo!(),
                        TypeKind::UNION => todo!(),
                        TypeKind::BITSET => todo!(),
                        TypeKind::SEQUENCE => todo!(),
                        TypeKind::ARRAY => todo!(),
                        TypeKind::MAP => todo!(),
                    }
                }
                TypeKind::ARRAY => todo!(),
                TypeKind::MAP => todo!(),
            }
        }
        Ok(())
    }
    fn deserialize_appendable_struct(&mut self) -> Result<(), XTypesError> {
        // Ok(PlainCdrLeDecoder { deserializer: self })
        todo!()
    }
    fn deserialize_mutable_struct(&mut self) -> Result<(), XTypesError> {
        // Ok(PlCdrLeDecoder {
        //     buffer: self.reader.buffer(),
        // })
        todo!()
    }
    fn deserialize_array(&mut self) -> Result<(), XTypesError> {
        // Ok(ArrayDecoder { deserializer: self })
        todo!()
    }
    fn deserialize_sequence(&mut self) -> Result<(), XTypesError> {
        // let len = self.deserialize_uint32()? as usize;
        // Ok(SequenceDecoder {
        //     deserializer: self,
        //     len,
        // })
        todo!()
    }
    fn deserialize_string(&mut self) -> Result<&'de str, XTypesError> {
        str::from_utf8(
            self.deserialize_byte_sequence()?
                .split_last()
                .ok_or(XTypesError::InvalidData)?
                .1,
        )
        .map_err(|_| XTypesError::InvalidData)
    }

    fn deserialize_byte_sequence(&mut self) -> Result<&'de [u8], XTypesError> {
        // let length = self.deserialize_uint32()? as usize;
        // self.reader.read_all(length)
        todo!()
    }
    fn deserialize_byte_array<const N: usize>(&mut self) -> Result<&'de [u8; N], XTypesError> {
        self.reader.read()
    }
}

impl<'de, E> XTypesDeserializer<'de> for &mut Xcdr2Deserializer<'de, E> {
    fn deserialize_final_struct(&mut self, v: &mut DynamicData) -> Result<(), XTypesError> {
        // Ok(PlainCdr2Decoder { deserializer: self })
        todo!()
    }
    fn deserialize_appendable_struct(&mut self) -> Result<(), XTypesError> {
        // let _dheader = self.deserialize_uint32()?;
        // Ok(DelimitedCdrDecoder { deserializer: self })
        todo!()
    }
    fn deserialize_mutable_struct(&mut self) -> Result<(), XTypesError> {
        // Ok(PlCdr2BeDecoder {
        //     buffer: self.reader.buffer(),
        // })
        todo!()
    }
    fn deserialize_array(&mut self) -> Result<(), XTypesError> {
        // Ok(ArrayDecoder { deserializer: self })\
        todo!()
    }
    fn deserialize_sequence(&mut self) -> Result<(), XTypesError> {
        // let len = self.deserialize_uint32()? as usize;
        // Ok(SequenceDecoder {
        //     deserializer: self,
        //     len,
        // })
        todo!()
    }

    fn deserialize_string(&mut self) -> Result<&'de str, XTypesError> {
        str::from_utf8(
            self.deserialize_byte_sequence()?
                .split_last()
                .ok_or(XTypesError::InvalidData)?
                .1,
        )
        .map_err(|_| XTypesError::InvalidData)
    }
    fn deserialize_byte_sequence(&mut self) -> Result<&'de [u8], XTypesError> {
        todo!()
        // let length = self.deserialize_uint32()? as usize;
        // self.reader.read_all(length)
    }
    fn deserialize_byte_array<const N: usize>(&mut self) -> Result<&'de [u8; N], XTypesError> {
        self.reader.read()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        infrastructure::type_support::TypeSupport,
        xtypes::{
            bytes::ByteBuf,
            data_representation::endianness::{BigEndian, LittleEndian},
        },
    };

    use super::*;

    // #[test]
    // fn padding_reader() {
    //     let buffer = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    //     let mut reader = Reader::new(buffer);
    //     reader.seek(1);
    //     reader.seek_padding(4);
    //     assert_eq!(reader.pos, 4);
    //     let mut reader = Reader::new(buffer);
    //     reader.seek(2);
    //     reader.seek_padding(4);
    //     assert_eq!(reader.pos, 4);
    //     let mut reader = Reader::new(buffer);
    //     reader.seek(3);
    //     reader.seek_padding(4);
    //     assert_eq!(reader.pos, 4);
    //     let mut reader = Reader::new(buffer);
    //     reader.seek(4);
    //     reader.seek_padding(4);
    //     assert_eq!(reader.pos, 4);
    //     let mut reader = Reader::new(buffer);
    //     reader.seek(5);
    //     reader.seek_padding(4);
    //     assert_eq!(reader.pos, 8);

    //     let mut reader = Reader::new(buffer);
    //     reader.seek(1);
    //     reader.seek_padding(8);
    //     assert_eq!(reader.pos, 8);
    //     let mut reader = Reader::new(buffer);
    //     reader.seek(2);
    //     reader.seek_padding(8);
    //     assert_eq!(reader.pos, 8);
    //     let mut reader = Reader::new(buffer);
    //     reader.seek(7);
    //     reader.seek_padding(8);
    //     assert_eq!(reader.pos, 8);
    //     let mut reader = Reader::new(buffer);
    //     reader.seek(8);
    //     reader.seek_padding(8);
    //     assert_eq!(reader.pos, 8);
    //     let mut reader = Reader::new(buffer);
    //     reader.seek(9);
    //     reader.seek_padding(8);
    //     assert_eq!(reader.pos, 16);
    // }

    fn deserialize_v1_be<'de>(
        dynamic_type: DynamicType,
        data: &'de [u8],
    ) -> Result<DynamicData, XTypesError> {
        deserialize_nested(dynamic_type, &mut Xcdr1Deserializer::new(data, BigEndian))
    }
    fn deserialize_v1_le<'de>(
        dynamic_type: DynamicType,
        data: &'de [u8],
    ) -> Result<DynamicData, XTypesError> {
        deserialize_nested(
            dynamic_type,
            &mut Xcdr1Deserializer::new(data, LittleEndian),
        )
    }
    // fn deserialize_v2_be<'de, T: XTypesDeserialize<'de>>(
    //     data: &'de [u8],
    // ) -> Result<T, XTypesError> {
    //     T::deserialize(&mut Xcdr2BeDeserializer {
    //         reader: Reader::new(data),
    //     })
    // }
    // fn deserialize_v2_le<'de, T: XTypesDeserialize<'de>>(
    //     data: &'de [u8],
    // ) -> Result<T, XTypesError> {
    //     T::deserialize(&mut Xcdr2LeDeserializer {
    //         reader: Reader::new(data),
    //     })
    // }
    // #[test]
    // fn deserialize_bool() {
    //     let expected = Ok(true);
    //     assert_eq!(deserialize_v1_be(&[1]), expected);
    //     assert_eq!(deserialize_v1_le(&[1]), expected);
    //     assert_eq!(deserialize_v2_be(&[1]), expected);
    //     assert_eq!(deserialize_v2_le(&[1]), expected);
    //     let expected = Ok(false);
    //     assert_eq!(deserialize_v1_be(&[0]), expected);
    //     assert_eq!(deserialize_v1_le(&[0]), expected);
    //     assert_eq!(deserialize_v2_be(&[0]), expected);
    //     assert_eq!(deserialize_v2_le(&[0]), expected);
    //     let expected = Err(XTypesError::InvalidData);
    //     assert_eq!(deserialize_v1_be::<bool>(&[2]), expected);
    //     assert_eq!(deserialize_v1_le::<bool>(&[2]), expected);
    //     assert_eq!(deserialize_v2_be::<bool>(&[2]), expected);
    //     assert_eq!(deserialize_v2_le::<bool>(&[2]), expected);
    // }
    // #[test]
    // fn deserialize_u8() {
    //     let expected = Ok(1_u8);
    //     assert_eq!(deserialize_v1_be(&[1]), expected);
    //     assert_eq!(deserialize_v1_le(&[1]), expected);
    //     assert_eq!(deserialize_v2_be(&[1]), expected);
    //     assert_eq!(deserialize_v2_le(&[1]), expected);
    // }
    // #[test]
    // fn deserialize_u16() {
    //     let expected = Ok(1_u16);
    //     assert_eq!(deserialize_v1_be(&[0, 1]), expected);
    //     assert_eq!(deserialize_v1_le(&[1, 0]), expected);
    //     assert_eq!(deserialize_v2_be(&[0, 1]), expected);
    //     assert_eq!(deserialize_v2_le(&[1, 0]), expected);
    // }
    // #[test]
    // fn deserialize_u32() {
    //     let expected = Ok(1_u32);
    //     assert_eq!(deserialize_v1_be(&[0, 0, 0, 1]), expected);
    //     assert_eq!(deserialize_v1_le(&[1, 0, 0, 0]), expected);
    //     assert_eq!(deserialize_v2_be(&[0, 0, 0, 1]), expected);
    //     assert_eq!(deserialize_v2_le(&[1, 0, 0, 0]), expected);
    // }
    // #[test]
    // fn deserialize_u64() {
    //     let expected = Ok(7_u64);
    //     assert_eq!(deserialize_v1_be(&[0, 0, 0, 0, 0, 0, 0, 7,]), expected);
    //     assert_eq!(deserialize_v1_le(&[7, 0, 0, 0, 0, 0, 0, 0,]), expected);
    //     assert_eq!(deserialize_v2_be(&[0, 0, 0, 0, 0, 0, 0, 7,]), expected);
    //     assert_eq!(deserialize_v2_le(&[7, 0, 0, 0, 0, 0, 0, 0,]), expected);
    // }
    // #[test]
    // fn deserialize_i8() {
    //     let expected = Ok(1_i8);
    //     assert_eq!(deserialize_v1_be(&[1]), expected);
    //     assert_eq!(deserialize_v1_le(&[1]), expected);
    //     assert_eq!(deserialize_v2_be(&[1]), expected);
    //     assert_eq!(deserialize_v2_le(&[1]), expected);
    // }
    // #[test]
    // fn deserialize_i16() {
    //     let expected = Ok(1_i16);
    //     assert_eq!(deserialize_v1_be(&[0, 1]), expected);
    //     assert_eq!(deserialize_v1_le(&[1, 0]), expected);
    //     assert_eq!(deserialize_v2_be(&[0, 1]), expected);
    //     assert_eq!(deserialize_v2_le(&[1, 0]), expected);
    // }
    // #[test]
    // fn deserialize_i32() {
    //     let expected = Ok(1_i32);
    //     assert_eq!(deserialize_v1_be(&[0, 0, 0, 1]), expected);
    //     assert_eq!(deserialize_v1_le(&[1, 0, 0, 0]), expected);
    //     assert_eq!(deserialize_v2_be(&[0, 0, 0, 1]), expected);
    //     assert_eq!(deserialize_v2_le(&[1, 0, 0, 0]), expected);
    // }
    // #[test]
    // fn deserialize_i64() {
    //     let expected = Ok(7_i64);
    //     assert_eq!(deserialize_v1_be(&[0, 0, 0, 0, 0, 0, 0, 7,]), expected);
    //     assert_eq!(deserialize_v1_le(&[7, 0, 0, 0, 0, 0, 0, 0,]), expected);
    //     assert_eq!(deserialize_v2_be(&[0, 0, 0, 0, 0, 0, 0, 7,]), expected);
    //     assert_eq!(deserialize_v2_le(&[7, 0, 0, 0, 0, 0, 0, 0,]), expected);
    // }
    // #[test]
    // fn deserialize_f32() {
    //     let expected = Ok(core::f32::MIN_POSITIVE);
    //     assert_eq!(deserialize_v1_be(&[0x00, 0x80, 0x00, 0x00]), expected);
    //     assert_eq!(deserialize_v1_le(&[0x00, 0x00, 0x80, 0x00]), expected);
    //     assert_eq!(deserialize_v2_be(&[0x00, 0x80, 0x00, 0x00]), expected);
    //     assert_eq!(deserialize_v2_le(&[0x00, 0x00, 0x80, 0x00]), expected);
    // }
    // #[test]
    // fn deserialize_f64() {
    //     let expected = Ok(core::f64::MIN_POSITIVE);
    //     assert_eq!(deserialize_v1_be(&[0, 0x10, 0, 0, 0, 0, 0, 0]), expected);
    //     assert_eq!(deserialize_v1_le(&[0, 0, 0, 0, 0, 0, 0x10, 0]), expected);
    //     assert_eq!(deserialize_v2_be(&[0, 0x10, 0, 0, 0, 0, 0, 0]), expected);
    //     assert_eq!(deserialize_v2_le(&[0, 0, 0, 0, 0, 0, 0x10, 0]), expected);
    // }
    // #[test]
    // fn deserialize_char() {
    //     let expected = Ok('A');
    //     assert_eq!(deserialize_v1_be(&[0x41]), expected);
    //     assert_eq!(deserialize_v1_le(&[0x41]), expected);
    //     assert_eq!(deserialize_v2_be(&[0x41]), expected);
    //     assert_eq!(deserialize_v2_le(&[0x41]), expected);
    // }

    // #[test]
    // fn deserialize_string() {
    //     let expected = Ok("Hola");
    //     assert_eq!(
    //         deserialize_v1_be(&[
    //             0, 0, 0, 5, //length
    //             b'H', b'o', b'l', b'a', // str
    //             0x00, // terminating 0
    //         ]),
    //         expected
    //     );
    //     assert_eq!(
    //         deserialize_v1_le(&[
    //             5, 0, 0, 0, //length
    //             b'H', b'o', b'l', b'a', // str
    //             0x00, // terminating 0
    //         ]),
    //         expected
    //     );
    //     assert_eq!(
    //         deserialize_v2_be(&[
    //             0, 0, 0, 5, //length
    //             b'H', b'o', b'l', b'a', // str
    //             0x00, // terminating 0
    //         ]),
    //         expected
    //     );
    //     assert_eq!(
    //         deserialize_v2_le(&[
    //             5, 0, 0, 0, //length
    //             b'H', b'o', b'l', b'a', // str
    //             0x00, // terminating 0
    //         ]),
    //         expected
    //     );
    // }

    // #[test]
    // fn deserialize_bytes() {
    //     let expected = Ok(&[1u8, 2, 3, 4, 5][..]);
    //     assert_eq!(
    //         deserialize_v1_be(&[
    //             0, 0, 0, 5, // length
    //             1, 2, 3, 4, 5 // data
    //         ]),
    //         expected
    //     );
    //     assert_eq!(
    //         deserialize_v1_le(&[
    //             5, 0, 0, 0, // length
    //             1, 2, 3, 4, 5 // data
    //         ]),
    //         expected
    //     );
    //     assert_eq!(
    //         deserialize_v2_be(&[
    //             0, 0, 0, 5, // length
    //             1, 2, 3, 4, 5 // data
    //         ]),
    //         expected
    //     );
    //     assert_eq!(
    //         deserialize_v2_le(&[
    //             5, 0, 0, 0, // length
    //             1, 2, 3, 4, 5 // data
    //         ]),
    //         expected
    //     );
    // }

    // #[test]
    // fn deserialize_byte_array() {
    //     let expected = Ok(&[1u8, 2, 3, 4, 5]);
    //     assert_eq!(deserialize_v1_be(&[1, 2, 3, 4, 5, 77]), expected);
    //     assert_eq!(deserialize_v1_le(&[1, 2, 3, 4, 5, 77]), expected);
    //     assert_eq!(deserialize_v2_be(&[1, 2, 3, 4, 5, 77]), expected);
    //     assert_eq!(deserialize_v2_le(&[1, 2, 3, 4, 5, 77]), expected);
    // }

    // #[test]
    // fn deserialize_sequence() {
    //     #[derive(Debug, PartialEq, TypeSupport)]
    //     struct Atype(u8);
    //     impl<'de> XTypesDeserialize<'de> for Atype {
    //         fn deserialize(
    //             deserializer: impl XTypesDeserializer<'de>,
    //         ) -> Result<Self, XTypesError> {
    //             Ok(Atype(deserializer.deserialize_uint8()?))
    //         }
    //     }

    //     let expected = Ok([Atype(1), Atype(2)]);
    //     assert_eq!(deserialize_v1_be(&[1, 2, 77]), expected);
    // }

    #[derive(TypeSupport, Debug, PartialEq)]
    struct FinalType {
        field_u16: u16,
        field_u64: u64,
    }

    #[test]
    fn deserialize_final_struct() {
        let expected = FinalType {
            field_u16: 7,
            field_u64: 9,
        }
        .create_dynamic_sample();
        // PLAIN_CDR:
        assert_eq!(
            deserialize_v1_be(
                FinalType::get_type(),
                &[
                    0, 7, 0, 0, 0, 0, 0, 0, // field_u16 | padding (6 bytes)
                    0, 0, 0, 0, 0, 0, 0, 9, // field_u64
                ]
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            deserialize_v1_le(
                FinalType::get_type(),
                &[
                    7, 0, 0, 0, 0, 0, 0, 0, // field_u16 | padding (6 bytes)
                    9, 0, 0, 0, 0, 0, 0, 0, // field_u64
                ]
            )
            .unwrap(),
            expected
        );
        // PLAIN_CDR2:
        // assert_eq!(
        //     deserialize_v2_be::<FinalType>(&[
        //         0, 7, 0, 0, // field_u16 | padding (2 bytes)
        //         0, 0, 0, 0, 0, 0, 0, 9, // field_u64
        //     ]),
        //     expected
        // );
        // assert_eq!(
        //     deserialize_v2_le::<FinalType>(&[
        //         7, 0, 0, 0, // field_u16 | padding (2 bytes)
        //         9, 0, 0, 0, 0, 0, 0, 0, // field_u64
        //     ]),
        //     expected
        // );
    }

    #[derive(Debug, PartialEq, TypeSupport)]
    //@extensibility(FINAL)
    struct TypeWithString {
        field_str: String,
        field_u16: u16,
        field_slice: ByteBuf,
    }

    #[test]
    fn deserialize_struct_with_str() {
        let expected = TypeWithString {
            field_str: String::from("xt"),
            field_u16: 9,
            field_slice: ByteBuf(vec![10, 11]),
        }
        .create_dynamic_sample();
        // PLAIN_CDR:
        assert_eq!(
            deserialize_v1_be(
                TypeWithString::get_type(),
                &[
                    0, 0, 0, 3, // field_str: length
                    b'x', b't', 0, 0, //field_str: data | padding (1 bytes)
                    0, 9, 0, 0, // field_u16 | padding (2 bytes)
                    0, 0, 0, 2, // field_slice: length
                    10, 11, //field_slice: data
                ]
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            deserialize_v1_le(
                TypeWithString::get_type(),
                &[
                    3, 0, 0, 0, // field_str: length
                    b'x', b't', 0, 0, //field_str: data | padding (1 bytes)
                    9, 0, 0, 0, // field_u16 | padding (2 bytes)
                    2, 0, 0, 0, // field_slice: length
                    10, 11, //field_slice: data
                ]
            )
            .unwrap(),
            expected
        );
        // // PLAIN_CDR2:
        // assert_eq!(
        //     deserialize_v2_be(&[
        //         0, 0, 0, 3, // field_str: length
        //         b'x', b't', 0, 0, //field_str: data | padding (1 bytes)
        //         0, 9, 0, 0, // field_u16 | padding (2 bytes)
        //         0, 0, 0, 2, // field_slice: length
        //         10, 11, //field_slice: data
        //     ]),
        //     expected
        // );
        // assert_eq!(
        //     deserialize_v2_le(&[
        //         3, 0, 0, 0, // field_str: length
        //         b'x', b't', 0, 0, //field_str: data | padding (1 bytes)
        //         9, 0, 0, 0, // field_u16 | padding (2 bytes)
        //         2, 0, 0, 0, // field_slice: length
        //         10, 11, //field_slice: data
        //     ]),
        //     expected
        // );
    }

    #[derive(Debug, PartialEq, TypeSupport)]
    //@extensibility(FINAL)
    struct NestedFinalType {
        field_nested: FinalType,
        field_u8: u8,
    }

    // #[derive(Debug, PartialEq)]
    // //@extensibility(FINAL)
    // struct FinalOptionalType {
    //     field: u8,
    //     optional_field: Option<u16>,
    // }

    // impl<'de> XTypesDeserialize<'de> for FinalOptionalType {
    //     fn deserialize(deserializer: impl XTypesDeserializer<'de>) -> Result<Self, XTypesError> {
    //         let mut d = deserializer.deserialize_final_struct()?;
    //         Ok(Self {
    //             field: d.deserialize_field("field")?,
    //             optional_field: d.deserialize_optional_field("optional_field")?,
    //         })
    //     }
    // }
    // #[test]
    // fn deserialize_final_optional_struct() {
    //     let some = Ok(FinalOptionalType {
    //         field: 6,
    //         optional_field: Some(7),
    //     });
    //     // PLAIN_CDR:
    //     assert_eq!(
    //         deserialize_v1_be(&[
    //             6, 0, 0, 0, // u8 | padding
    //             0, 0, 0, 2, // HEADER (FLAGS+ID | length)
    //             0, 7 // optional_field value
    //         ]),
    //         some
    //     );
    //     assert_eq!(
    //         deserialize_v1_le(&[
    //             6, 0, 0, 0, // u8 | padding
    //             0, 0, 2, 0, // HEADER (FLAGS+ID | length)
    //             7, 0 // optional_field value
    //         ]),
    //         some
    //     );
    //     // PLAIN_CDR2:
    //     assert_eq!(
    //         deserialize_v2_be(&[
    //             6, 1, // u8 | boolean for option
    //             0, 7 // optional_field value
    //         ]),
    //         some
    //     );
    //     assert_eq!(
    //         deserialize_v2_le(&[
    //             6, 1, // u8 | boolean for option
    //             7, 0 // optional_field value
    //         ]),
    //         some
    //     );

    //     let none = Ok(FinalOptionalType {
    //         field: 6,
    //         optional_field: None,
    //     });
    //     // PLAIN_CDR:
    //     assert_eq!(
    //         deserialize_v1_be(&[
    //             6, 0, 0, 0, // u8 | padding
    //             0, 0, 0, 0, // HEADER (FLAGS+ID | length)
    //         ]),
    //         none
    //     );
    //     assert_eq!(
    //         deserialize_v1_le(&[
    //             6, 0, 0, 0, // u8 | padding
    //             0, 0, 0, 0, // HEADER (FLAGS+ID | length)
    //         ]),
    //         none
    //     );
    //     // PLAIN_CDR2:
    //     assert_eq!(
    //         deserialize_v2_be(&[
    //         6, 0, // u8 | boolean for option
    //     ]),
    //         none
    //     );
    //     assert_eq!(
    //         deserialize_v2_le(&[
    //         6, 0, // u8 | boolean for option
    //     ]),
    //         none
    //     );
    // }

    #[test]
    fn deserialize_nested_final_struct() {
        let expected = NestedFinalType {
            field_nested: FinalType {
                field_u16: 7,
                field_u64: 9,
            },
            field_u8: 10,
        }
        .create_dynamic_sample();
        // PLAIN_CDR:
        assert_eq!(
            deserialize_v1_be(
                NestedFinalType::get_type(),
                &[
                    0, 7, 0, 0, 0, 0, 0, 0, // nested FinalType (u16) | padding (6 bytes)
                    0, 0, 0, 0, 0, 0, 0, 9,  // nested FinalType (u64)
                    10, //u8
                ]
            )
            .unwrap(),
            expected
        );
        // assert_eq!(
        //     deserialize_v1_le(
        //         NestedFinalType::get_type(),
        //         &[
        //             7, 0, 0, 0, 0, 0, 0, 0, // nested FinalType (u16) | padding (6 bytes)
        //             9, 0, 0, 0, 0, 0, 0, 0,  // nested FinalType (u64)
        //             10, //u8
        //         ]
        //     )
        //     .unwrap(),
        //     expected
        // );
        //     //PLAIN_CDR2:
        //     assert_eq!(
        //         deserialize_v2_le::<NestedFinalType>(&[
        //             7, 0, 0, 0, // nested FinalType (u16) | padding (2 bytes)
        //             9, 0, 0, 0, 0, 0, 0, 0,  // nested FinalType (u64)
        //             10, //u8
        //         ]),
        //         expected
        //     );
        //     assert_eq!(
        //         deserialize_v2_be::<NestedFinalType>(&[
        //             0, 7, 0, 0, // nested FinalType (u16) | padding
        //             0, 0, 0, 0, 0, 0, 0, 9,  // nested FinalType (u64)
        //             10, //u8
        //         ]),
        //         expected
        //     );
    }

    // #[derive(Debug, PartialEq)]
    // // @extensibility(APPENDABLE) @nested
    // struct AppendableType {
    //     value: u16,
    //     next_value: u16,
    // }
    // impl<'de> XTypesDeserialize<'de> for AppendableType {
    //     fn deserialize(deserializer: impl XTypesDeserializer<'de>) -> Result<Self, XTypesError> {
    //         let mut deserializer = deserializer.deserialize_appendable_struct()?;
    //         Ok(Self {
    //             value: deserializer.deserialize_field("value")?,
    //             next_value: deserializer.deserialize_field("next_value")?,
    //         })
    //     }
    // }

    // #[test]
    // fn deserialize_appendable_struct() {
    //     let expected = Ok(AppendableType {
    //         value: 7,
    //         next_value: 8,
    //     });
    //     // PLAIN_CDR:
    //     assert_eq!(deserialize_v1_be::<AppendableType>(&[0, 7, 0, 8]), expected);
    //     assert_eq!(deserialize_v1_le::<AppendableType>(&[7, 0, 8, 0]), expected);
    //     // DELIMITED_CDR:
    //     assert_eq!(
    //         deserialize_v2_be::<AppendableType>(&[
    //             0, 0, 0, 4, // DHEADER
    //             0, 7, 0, 8 // value | next value
    //         ]),
    //         expected
    //     );
    //     assert_eq!(
    //         deserialize_v2_le::<AppendableType>(&[
    //             4, 0, 0, 0, // DHEADER
    //             7, 0, 8, 0 // value | next value
    //         ]),
    //         expected
    //     );
    // }

    // #[derive(Debug, PartialEq)]
    // //@extensibility(MUTABLE)
    // struct MutableType {
    //     // @id(0x005A) @key
    //     key: u8,
    //     // @id(0x0050)
    //     participant_key: u32,
    // }

    // impl<'de> XTypesDeserialize<'de> for MutableType {
    //     fn deserialize(deserializer: impl XTypesDeserializer<'de>) -> Result<Self, XTypesError> {
    //         let mut des = deserializer.deserialize_mutable_struct()?;
    //         Ok(Self {
    //             key: des.deserialize_field(0x005A, "key")?,
    //             participant_key: des.deserialize_field(0x0050, "participant_key")?,
    //         })
    //     }
    // }

    // #[test]
    // fn deserialize_mutable_struct() {
    //     let expected = Ok(MutableType {
    //         key: 7,
    //         participant_key: 8,
    //     });
    //     // PL_CDR:
    //     assert_eq!(
    //         deserialize_v1_be::<MutableType>(&[
    //             0x00, 0x05A, 0, 1, // PID | length
    //             7, 0, 0, 0, // key | padding
    //             0x00, 0x050, 0, 4, // PID | length
    //             0, 0, 0, 8, // participant_key
    //             0, 0, 0, 0, // Sentinel
    //         ]),
    //         expected
    //     );
    //     assert_eq!(
    //         deserialize_v1_le::<MutableType>(&[
    //             0x05A, 0x00, 1, 0, // PID | length
    //             7, 0, 0, 0, // key | padding
    //             0x050, 0x00, 4, 0, // PID | length
    //             8, 0, 0, 0, // participant_key
    //             0, 0, 0, 0, // Sentinel
    //         ]),
    //         expected
    //     );
    //     // PL_CDR2:
    //     assert_eq!(
    //         deserialize_v2_be::<MutableType>(&[
    //             0x00, 0x05A, 0, 1, // PID | length
    //             7, 0, 0, 0, // key | padding
    //             0x00, 0x050, 0, 4, // PID | length
    //             0, 0, 0, 8, // participant_key
    //             0, 0, 0, 0, // Sentinel
    //         ]),
    //         expected
    //     );
    //     assert_eq!(
    //         deserialize_v2_le::<MutableType>(&[
    //             0x05A, 0x00, 1, 0, // PID | length
    //             7, 0, 0, 0, // key | padding
    //             0x050, 0x00, 4, 0, // PID | length
    //             8, 0, 0, 0, // participant_key
    //             0, 0, 0, 0, // Sentinel
    //         ]),
    //         expected
    //     );
    // }

    // #[derive(Debug, PartialEq)]
    // struct BasicTypes {
    //     f1: bool,
    //     f2: i8,
    //     f3: i16,
    //     f4: i32,
    //     f5: i64,
    //     f6: u8,
    //     f7: u16,
    //     f8: u32,
    //     f9: u64,
    //     f10: f32,
    //     f11: f64,
    //     f12: char,
    // }
    // impl<'de> XTypesDeserialize<'de> for BasicTypes {
    //     fn deserialize(deserializer: impl XTypesDeserializer<'de>) -> Result<Self, XTypesError> {
    //         let mut deserializer = deserializer.deserialize_final_struct()?;
    //         Ok(Self {
    //             f1: deserializer.deserialize_field("f1")?,
    //             f2: deserializer.deserialize_field("f2")?,
    //             f3: deserializer.deserialize_field("f3")?,
    //             f4: deserializer.deserialize_field("f4")?,
    //             f5: deserializer.deserialize_field("f5")?,
    //             f6: deserializer.deserialize_field("f6")?,
    //             f7: deserializer.deserialize_field("f7")?,
    //             f8: deserializer.deserialize_field("f8")?,
    //             f9: deserializer.deserialize_field("f9")?,
    //             f10: deserializer.deserialize_field("f10")?,
    //             f11: deserializer.deserialize_field("f11")?,
    //             f12: deserializer.deserialize_field("f12")?,
    //         })
    //     }
    // }

    // #[test]
    // fn deserialize_basic_types_struct() {
    //     let expected = Ok(BasicTypes {
    //         f1: true,
    //         f2: 2,
    //         f3: 3,
    //         f4: 4,
    //         f5: 5,
    //         f6: 6,
    //         f7: 7,
    //         f8: 8,
    //         f9: 9,
    //         f10: 1.0,
    //         f11: 1.0,
    //         f12: 'a',
    //     });
    //     // PLAIN_CDR:
    //     assert_eq!(
    //         deserialize_v1_be(&[
    //             1, 2, 0, 3, 0, 0, 0, 4, // f1: bool | f2: i8 | f3: i16 | f4: i32
    //             0, 0, 0, 0, 0, 0, 0, 5, // f5: i64
    //             6, 0, 0, 7, 0, 0, 0, 8, // f6: u8 | padding (1 byte) | f7: u16 | f8: u32
    //             0, 0, 0, 0, 0, 0, 0, 9, // f9: u64
    //             0x3F, 0x80, 0x00, 0x00, 0, 0, 0, 0, // f10: f32 | padding (4 bytes)
    //             0x3F, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // f11: f64
    //             b'a', // f12: char
    //         ]),
    //         expected
    //     );
    //     assert_eq!(
    //         deserialize_v1_le(&[
    //             1, 2, 3, 0, 4, 0, 0, 0, // f1: bool | f2: i8 | f3: i16 | f4: i32
    //             5, 0, 0, 0, 0, 0, 0, 0, // f5: i64
    //             6, 0, 7, 0, 8, 0, 0, 0, // f6: u8 | padding (1 byte) | f7: u16 | f8: u32
    //             9, 0, 0, 0, 0, 0, 0, 0, // f9: u64
    //             0x00, 0x00, 0x80, 0x3F, 0, 0, 0, 0, // f10: f32 | padding (4 bytes)
    //             0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x3F, // f11: f64
    //             b'a', // f12: char
    //         ]),
    //         expected
    //     );
    //     //PLAIN_CDR2:
    //     assert_eq!(
    //         deserialize_v2_be(&[
    //             1, 2, 0, 3, // f1: bool | f2: i8 | f3: i16
    //             0, 0, 0, 4, // f4: i32
    //             0, 0, 0, 0, // f5-1: i64
    //             0, 0, 0, 5, // f5-2: i64
    //             6, 0, 0, 7, // f6: u8 | padding (1 byte) | f7: u16
    //             0, 0, 0, 8, // f8: u32
    //             0, 0, 0, 0, // f9-1: u64
    //             0, 0, 0, 9, // f9-2: u64
    //             0x3F, 0x80, 0x00, 0x00, // f10: f32
    //             0x3F, 0xF0, 0x00, 0x00, // f11-1: f64
    //             0x00, 0x00, 0x00, 0x00, // f11-2: f64
    //             b'a', // f12: char
    //         ]),
    //         expected
    //     );
    //     assert_eq!(
    //         deserialize_v2_le(&[
    //             1, 2, 3, 0, // f1: bool | f2: i8 | f3: i16
    //             4, 0, 0, 0, // f4: i32
    //             5, 0, 0, 0, // f5-1: i64
    //             0, 0, 0, 0, // f5-2: i64
    //             6, 0, 7, 0, // f6: u8 | padding (1 byte) | f7: u16
    //             8, 0, 0, 0, // f8: u32
    //             9, 0, 0, 0, // f9-1: u64
    //             0, 0, 0, 0, // f9-2: u64
    //             0x00, 0x00, 0x80, 0x3F, // f10: f32
    //             0x00, 0x00, 0x00, 0x00, // f11-1: f64
    //             0x00, 0x00, 0xF0, 0x3F, // f11-2: f64
    //             b'a', // f12: char
    //         ]),
    //         expected
    //     );
    // }
}
