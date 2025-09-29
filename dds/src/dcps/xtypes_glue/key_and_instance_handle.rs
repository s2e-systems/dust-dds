use crate::{
    infrastructure::instance::InstanceHandle,
    xtypes::{
        deserializer::{DeserializeAppendableStruct, DeserializeSequence, XTypesDeserializer},
        dynamic_type::{
            DynamicType, MemberDescriptor, TK_ARRAY, TK_BOOLEAN, TK_CHAR8, TK_FLOAT32, TK_FLOAT64,
            TK_INT16, TK_INT32, TK_INT64, TK_INT8, TK_SEQUENCE, TK_STRING8, TK_STRUCTURE,
            TK_UINT16, TK_UINT32, TK_UINT64, TK_UINT8,
        },
        error::XTypesError,
        serialize::{Write, XTypesSerializer},
        serializer::SerializeFinalStruct,
        xcdr_deserializer::{
            Xcdr1BeDeserializer, Xcdr1LeDeserializer, Xcdr2BeDeserializer, Xcdr2LeDeserializer,
        },
        xcdr_serializer::{Xcdr1LeSerializer, Xcdr2BeSerializer},
    },
};
use alloc::{string::String, vec::Vec};

struct Md5 {
    key: [u8; 16],
    context: md5::Context,
    length: usize,
}

impl Md5 {
    fn into_key(mut self) -> [u8; 16] {
        const ZEROS: [u8; 16] = [0; 16];
        if self.length < ZEROS.len() {
            self.context.consume(&ZEROS[self.length..]);
        }
        if self.length <= 16 {
            self.key
        } else {
            self.context.compute().into()
        }
    }
}

impl Write for Md5 {
    fn write(&mut self, buf: &[u8]) {
        let total_new_length = self.length + buf.len();
        if total_new_length <= self.key.len() {
            self.key[self.length..total_new_length].copy_from_slice(buf);
        }
        self.context.consume(buf);
        self.length += buf.len();
    }
}

fn deserialize_and_serialize_if_key_field<'a, T>(
    dynamic_type: &DynamicType,
    is_key_field: bool,
    de: &mut T,
    serializer: &mut impl SerializeFinalStruct,
) -> Result<(), XTypesError>
where
    for<'b> &'b mut T: XTypesDeserializer<'a>,
{
    match dynamic_type.get_kind() {
        TK_BOOLEAN => {
            let v = de.deserialize_boolean()?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        TK_INT8 => {
            let v = de.deserialize_int8()?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        TK_INT16 => {
            let v = de.deserialize_int16()?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        TK_INT32 => {
            let v = de.deserialize_int32()?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        TK_INT64 => {
            let v = de.deserialize_int64()?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        TK_UINT8 => {
            let v = de.deserialize_uint8()?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        TK_UINT16 => {
            let v = de.deserialize_uint16()?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        TK_UINT32 => {
            let v = de.deserialize_uint32()?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        TK_UINT64 => {
            let v = de.deserialize_uint64()?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        TK_FLOAT32 => {
            let v = de.deserialize_float32()?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        TK_FLOAT64 => {
            let v = de.deserialize_float64()?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        TK_CHAR8 => {
            let v = de.deserialize_char8()?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        TK_STRING8 => {
            let v = de.deserialize_string()?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        TK_SEQUENCE => {
            let len = de.deserialize_sequence()?.len() as u32;
            if is_key_field {
                serializer.serialize_field(&len, "")?;

                for _ in 0..len {
                    deserialize_and_serialize_if_key_field(
                        dynamic_type.get_descriptor().element_type.as_ref().unwrap(),
                        is_key_field,
                        de,
                        serializer,
                    )?;
                }
            }
        }
        TK_ARRAY => {
            for _ in 0..dynamic_type.get_descriptor().bound[0] {
                deserialize_and_serialize_if_key_field(
                    dynamic_type.get_descriptor().element_type.as_ref().unwrap(),
                    is_key_field,
                    de,
                    serializer,
                )?;
            }
        }
        TK_STRUCTURE => {
            push_to_key(&dynamic_type, serializer, de)?;
        }
        _ => todo!(),
    }
    Ok(())
}

fn deserialize_and_serialize_if_key_field_for_appendable_cdr<'a>(
    dynamic_type: &DynamicType,
    is_key_field: bool,
    de: &mut impl DeserializeAppendableStruct<'a>,
    serializer: &mut impl SerializeFinalStruct,
) -> Result<(), XTypesError> {
    let name = "";
    match dynamic_type.get_kind() {
        TK_BOOLEAN => {
            let v = de.deserialize_field::<bool>(name)?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        TK_INT8 => {
            let v = de.deserialize_field::<i8>(name)?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        TK_INT16 => {
            let v = de.deserialize_field::<i16>(name)?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        TK_INT32 => {
            let v = de.deserialize_field::<i32>(name)?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        TK_INT64 => {
            let v = de.deserialize_field::<i64>(name)?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        TK_UINT8 => {
            let v = de.deserialize_field::<u8>(name)?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        TK_UINT16 => {
            let v = de.deserialize_field::<u16>(name)?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        TK_UINT32 => {
            let v = de.deserialize_field::<u32>(name)?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        TK_UINT64 => {
            let v = de.deserialize_field::<u64>(name)?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        TK_FLOAT32 => {
            let v = de.deserialize_field::<f32>(name)?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        TK_FLOAT64 => {
            let v = de.deserialize_field::<f64>(name)?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        TK_CHAR8 => {
            let v = de.deserialize_field::<u8>(name)?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        TK_STRING8 => {
            let v = de.deserialize_field::<String>(name)?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        TK_SEQUENCE => {
            let len = de.deserialize_field::<u32>(name)?;
            if is_key_field {
                serializer.serialize_field(&len, "")?;

                for _ in 0..len {
                    deserialize_and_serialize_if_key_field_for_appendable_cdr(
                        dynamic_type.get_descriptor().element_type.as_ref().unwrap(),
                        is_key_field,
                        de,
                        serializer,
                    )?;
                }
            }
        }
        _ => todo!(),
    }
    Ok(())
}

fn push_to_key<'a, T>(
    dynamic_type: &DynamicType,
    serializer: &mut impl SerializeFinalStruct,
    de: &mut T,
) -> Result<(), XTypesError>
where
    for<'b> &'b mut T: XTypesDeserializer<'a>,
{
    match dynamic_type.get_descriptor().extensibility_kind {
        crate::xtypes::dynamic_type::ExtensibilityKind::Final => {
            for member_descriptor in dynamic_type.into_iter() {
                let member_descriptor = member_descriptor?;
                deserialize_and_serialize_if_key_field(
                    &member_descriptor.r#type,
                    member_descriptor.is_key,
                    de,
                    serializer,
                )?;
            }
        }
        crate::xtypes::dynamic_type::ExtensibilityKind::Appendable => {
            let mut appendable_struct_deserializer = de.deserialize_appendable_struct()?;
            for member_descriptor in dynamic_type.into_iter() {
                let member_descriptor = member_descriptor?;
                deserialize_and_serialize_if_key_field_for_appendable_cdr(
                    &member_descriptor.r#type,
                    member_descriptor.is_key,
                    &mut appendable_struct_deserializer,
                    serializer,
                )?;
            }
        }
        crate::xtypes::dynamic_type::ExtensibilityKind::Mutable => (),
    }

    Ok(())
}

fn push_to_key_for_key<'a, T>(
    dynamic_type: &DynamicType,
    serializer: &mut impl SerializeFinalStruct,
    de: &mut T,
) -> Result<(), XTypesError>
where
    for<'b> &'b mut T: XTypesDeserializer<'a>,
{
    for member_descriptor in dynamic_type.into_iter() {
        let member_descriptor = member_descriptor?;
        if member_descriptor.is_key {
            deserialize_and_serialize_if_key_field(
                &member_descriptor.r#type,
                true,
                de,
                serializer,
            )?;
        }
    }
    Ok(())
}

fn go_to_pid_le(mut reader: &[u8], pid: u32) -> Result<&[u8], XTypesError> {
    const PID_SENTINEL: u16 = 1;
    loop {
        let current_pid = u16::from_le_bytes([reader[0], reader[1]]);
        if current_pid == pid as u16 {
            return Ok(&reader[4..]);
        } else if current_pid == PID_SENTINEL {
            return Err(XTypesError::PidNotFound(pid as u16));
        } else {
            let length = u16::from_le_bytes([reader[2], reader[3]]) as usize;
            reader = &reader[length + 4..];
        }
    }
}

fn go_to_pid_be(mut reader: &[u8], pid: u32) -> Result<&[u8], XTypesError> {
    const PID_SENTINEL: u16 = 1;
    loop {
        let current_pid = u16::from_be_bytes([reader[0], reader[1]]);
        if current_pid == pid as u16 {
            return Ok(&reader[4..]);
        } else if current_pid == PID_SENTINEL {
            return Err(XTypesError::PidNotFound(pid as u16));
        } else {
            let length = u16::from_be_bytes([reader[2], reader[3]]) as usize;
            reader = &reader[length + 4..];
        }
    }
}

pub struct MemberDescriptorIter<'a> {
    dynamic_type: &'a DynamicType,
    range: core::ops::Range<u32>,
}
impl<'a> Iterator for MemberDescriptorIter<'a> {
    type Item = Result<&'a MemberDescriptor, XTypesError>;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.range.next()?;
        match self.dynamic_type.get_member_by_index(i) {
            Ok(member) => match member.get_descriptor() {
                Ok(descriptor) => Some(Ok(descriptor)),
                Err(err) => Some(Err(err)),
            },
            Err(err) => Some(Err(err)),
        }
    }
}

impl<'a> IntoIterator for &'a DynamicType {
    type Item = Result<&'a MemberDescriptor, XTypesError>;
    type IntoIter = MemberDescriptorIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        MemberDescriptorIter {
            dynamic_type: self,
            range: 0..self.get_member_count(),
        }
    }
}

fn push_to_key_parameter_list_le(
    dynamic_type: &DynamicType,
    serializer: &mut impl SerializeFinalStruct,
    data: &[u8],
) -> Result<(), XTypesError> {
    for descriptor in dynamic_type.into_iter() {
        let descriptor = descriptor?;
        if descriptor.is_key {
            let buffer = go_to_pid_le(data, descriptor.id)?;
            let mut de = Xcdr1LeDeserializer::new(buffer);
            deserialize_and_serialize_if_key_field(&descriptor.r#type, true, &mut de, serializer)?;
        }
    }
    Ok(())
}

fn push_to_key_parameter_list_be(
    dynamic_type: &DynamicType,
    serializer: &mut impl SerializeFinalStruct,
    data: &[u8],
) -> Result<(), XTypesError> {
    for descriptor in dynamic_type.into_iter() {
        let descriptor = descriptor?;
        if descriptor.is_key {
            let buffer = go_to_pid_be(data, descriptor.id)?;
            let mut de = Xcdr1BeDeserializer::new(buffer);
            deserialize_and_serialize_if_key_field(&descriptor.r#type, true, &mut de, serializer)?;
        }
    }
    Ok(())
}

type RepresentationIdentifier = [u8; 2];
const CDR_BE: RepresentationIdentifier = [0x00, 0x00];
const CDR_LE: RepresentationIdentifier = [0x00, 0x01];
const CDR2_BE: RepresentationIdentifier = [0x00, 0x06];
const CDR2_LE: RepresentationIdentifier = [0x00, 0x07];
const D_CDR2_BE: RepresentationIdentifier = [0x00, 0x08];
const D_CDR2_LE: RepresentationIdentifier = [0x00, 0x09];
const PL_CDR_BE: RepresentationIdentifier = [0x00, 0x02];
const PL_CDR_LE: RepresentationIdentifier = [0x00, 0x03];

pub fn get_instance_handle_from_serialized_key(
    mut data: &[u8],
    dynamic_type: &DynamicType,
) -> Result<InstanceHandle, XTypesError> {
    let mut md5_collection = Md5 {
        key: [0; 16],
        context: md5::Context::new(),
        length: 0,
    };
    {
        let representation_identifier = [data[0], data[1]];
        data = &data[4..];
        let mut serializer = Xcdr2BeSerializer::new(&mut md5_collection);
        let mut s = serializer.serialize_final_struct()?;
        match representation_identifier {
            CDR_BE => {
                push_to_key_for_key(dynamic_type, &mut s, &mut Xcdr1BeDeserializer::new(data))?
            }
            CDR_LE => {
                push_to_key_for_key(dynamic_type, &mut s, &mut Xcdr1LeDeserializer::new(data))?
            }
            CDR2_BE | D_CDR2_BE => {
                push_to_key_for_key(dynamic_type, &mut s, &mut Xcdr2BeDeserializer::new(data))?
            }
            CDR2_LE | D_CDR2_LE => {
                push_to_key_for_key(dynamic_type, &mut s, &mut Xcdr2LeDeserializer::new(data))?
            }
            _ => return Err(XTypesError::InvalidData),
        }
    }
    Ok(InstanceHandle::new(md5_collection.into_key()))
}

pub fn get_instance_handle_from_serialized_foo(
    mut data: &[u8],
    dynamic_type: &DynamicType,
) -> Result<InstanceHandle, XTypesError> {
    let mut md5_collection = Md5 {
        key: [0; 16],
        context: md5::Context::new(),
        length: 0,
    };
    {
        let representation_identifier = [data[0], data[1]];
        data = &data[4..];
        let mut serializer = Xcdr2BeSerializer::new(&mut md5_collection);
        let mut s = serializer.serialize_final_struct()?;
        match representation_identifier {
            CDR_BE => push_to_key(dynamic_type, &mut s, &mut Xcdr1BeDeserializer::new(data))?,
            CDR_LE => push_to_key(dynamic_type, &mut s, &mut Xcdr1LeDeserializer::new(data))?,
            CDR2_BE | D_CDR2_BE => {
                push_to_key(dynamic_type, &mut s, &mut Xcdr2BeDeserializer::new(data))?
            }
            CDR2_LE | D_CDR2_LE => {
                push_to_key(dynamic_type, &mut s, &mut Xcdr2LeDeserializer::new(data))?
            }
            PL_CDR_BE => push_to_key_parameter_list_be(dynamic_type, &mut s, data)?,
            PL_CDR_LE => push_to_key_parameter_list_le(dynamic_type, &mut s, data)?,
            _ => panic!("representation_identifier not supported"),
        }
    }
    Ok(InstanceHandle::new(md5_collection.into_key()))
}

pub fn get_serialized_key_from_serialized_foo(
    mut data: &[u8],
    dynamic_type: &DynamicType,
) -> Result<Vec<u8>, XTypesError> {
    let mut collection = Vec::new();
    {
        let representation_identifier = [data[0], data[1]];
        collection.extend_from_slice(&CDR_LE);
        collection.extend_from_slice(&[0, 0]);
        data = &data[4..];

        let mut serializer = Xcdr1LeSerializer::new(&mut collection);
        let mut s = serializer.serialize_final_struct()?;

        match representation_identifier {
            CDR_BE => push_to_key(dynamic_type, &mut s, &mut Xcdr1BeDeserializer::new(data))?,
            CDR_LE => push_to_key(dynamic_type, &mut s, &mut Xcdr1LeDeserializer::new(data))?,
            CDR2_BE => push_to_key(dynamic_type, &mut s, &mut Xcdr2BeDeserializer::new(data))?,
            CDR2_LE => push_to_key(dynamic_type, &mut s, &mut Xcdr2LeDeserializer::new(data))?,
            PL_CDR_BE => push_to_key_parameter_list_be(dynamic_type, &mut s, data)?,
            PL_CDR_LE => push_to_key_parameter_list_le(dynamic_type, &mut s, data)?,
            _ => panic!("representation_identifier not supported"),
        }
    }
    let padding_len = collection.len().div_ceil(4) * 4 - collection.len();
    const ZEROS: [u8; 4] = [0; 4];
    collection.extend_from_slice(&ZEROS[..padding_len]);
    collection[3] |= padding_len as u8;
    Ok(collection)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::type_support::TypeSupport;

    #[derive(TypeSupport)]
    #[dust_dds(extensibility = "mutable")]
    struct MutableStruct {
        #[dust_dds(key, id = 10)]
        _key_field1: u8,
        #[dust_dds(id = 20)]
        _field_inbetween: u32,
        #[dust_dds(key, id = 11)]
        _key_field2: u16,
    }

    #[test]
    fn key_from_mutable_struct_be() {
        let data = [
            0, 2, 0, 0, //rtps header (PL_CDR_BE: version 1)
            0, 10, 0, 4, // PID | length (CDR1: incl padding)
            1, 0, 0, 0, //key_field1 (u8) | padding (3bytes)
            0, 77, 0, 4, // PID | length (CDR1: incl padding)
            0, 0, 0, 7, //field_inbetween (u32)
            0, 11, 0, 4, // PID | length (CDR1: incl padding)
            0, 2, 0, 0, //key_field2 (u16) | padding (2bytes)
            0, 1, 0, 0, // Sentinel
        ];
        let expected_instance_handle =
            InstanceHandle::new([1, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(
            get_instance_handle_from_serialized_foo(&data, &MutableStruct::get_type()).unwrap(),
            expected_instance_handle
        );
        let expected_key = vec![
            0, 1, 0, 0, // RTPS header
            1, 0, 2, 0, // key_field1 (u8) | padding (1byte) | key_field2 (u16)
        ];
        assert_eq!(
            get_serialized_key_from_serialized_foo(&data, &MutableStruct::get_type()).unwrap(),
            expected_key
        );
        assert_eq!(
            get_instance_handle_from_serialized_key(&expected_key, &MutableStruct::get_type())
                .unwrap(),
            expected_instance_handle
        )
    }

    #[test]
    fn key_from_mutable_struct_le() {
        let data = [
            0, 3, 0, 0, //rtps header (PL_CDR_LE: version 1)
            10, 0, 4, 0, // PID | length (CDR1: incl padding)
            1, 0, 0, 0, //key_field1 (u8) | padding (3bytes)
            77, 0, 4, 0, // PID | length (CDR1: incl padding)
            7, 0, 0, 0, //field_inbetween (u32)
            11, 0, 4, 0, // PID | length (CDR1: incl padding)
            2, 0, 0, 0, //key_field2 (u16) | padding (2bytes)
            1, 0, 0, 0, // Sentinel
        ];
        let expected_instance_handle =
            InstanceHandle::new([1, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(
            get_instance_handle_from_serialized_foo(&data, &MutableStruct::get_type()).unwrap(),
            expected_instance_handle
        );
        let expected_key = vec![
            0, 1, 0, 0, // RTPS header
            1, 0, 2, 0, // key_field1 (u8) | padding (1byte) | key_field2 (u16)
        ];
        assert_eq!(
            get_serialized_key_from_serialized_foo(&data, &MutableStruct::get_type()).unwrap(),
            expected_key
        );
        assert_eq!(
            get_instance_handle_from_serialized_key(&expected_key, &MutableStruct::get_type())
                .unwrap(),
            expected_instance_handle
        )
    }

    #[derive(TypeSupport)]
    #[dust_dds(extensibility = "final")]
    struct Nested {
        #[dust_dds(key)]
        _x: u8,
        #[dust_dds(key)]
        _y: u8,
    }
    #[derive(TypeSupport)]
    #[dust_dds(extensibility = "final")]
    struct Complex {
        _field1: i64,
        #[dust_dds(key)]
        _key_field1: u16,
        _field2: u32,
        #[dust_dds(key)]
        _key_field2: Nested,
    }

    #[derive(TypeSupport)]
    #[dust_dds(extensibility = "final")]
    struct Simple {
        #[dust_dds(key)]
        _key_field1: i64,
        #[dust_dds(key)]
        _key_field2: i16,
    }

    #[test]
    fn simple_key_be() {
        #[rustfmt::skip]
        let data = [
            0, 0, 0, 0b0000_0010, //rtps header
            0, 0, 0, 0, 0, 0, 0, 1, //key_field1 (i64)
            0, 2, 0, 0, //key_field1 (i16) | padding 2 bytes
        ];
        let expected_instance_handle =
            InstanceHandle::new([0, 0, 0, 0, 0, 0, 0, 1, 0, 2, 0, 0, 0, 0, 0, 0]);
        assert_eq!(
            get_instance_handle_from_serialized_foo(&data, &Simple::get_type()).unwrap(),
            expected_instance_handle
        );
        #[rustfmt::skip]
        let expected_key = vec![
            0, 1, 0, 0b0000_0010, // RTPS header
            1, 0, 0, 0, 0, 0, 0, 0, // key_field1
            2, 0, 0, 0, // key_field2 | padding 2 bytes
        ];
        assert_eq!(
            get_serialized_key_from_serialized_foo(&data, &Simple::get_type()).unwrap(),
            expected_key
        );
        assert_eq!(
            get_instance_handle_from_serialized_key(&expected_key, &Simple::get_type()).unwrap(),
            expected_instance_handle
        )
    }

    #[test]
    fn simple_key_le() {
        #[rustfmt::skip]
        let data = [
            0, 1, 0, 0b0000_0010, //rtps header
            1, 0, 0, 0, 0, 0, 0, 0, //key_field1 (i64)
            2, 0, 0, 0, //key_field1 (i16) | padding 2 bytes
        ];
        let expected_instance_handle =
            InstanceHandle::new([0, 0, 0, 0, 0, 0, 0, 1, 0, 2, 0, 0, 0, 0, 0, 0]);
        assert_eq!(
            get_instance_handle_from_serialized_foo(&data, &Simple::get_type()).unwrap(),
            expected_instance_handle
        );
        let expected_key = vec![
            0,
            1,
            0,
            0b0000_0010, // RTPS header
            1,
            0,
            0,
            0,
            0,
            0,
            0,
            0, // key_field1
            2,
            0,
            0,
            0, // key_field2 | padding 2 bytes
        ];
        assert_eq!(
            get_serialized_key_from_serialized_foo(&data, &Simple::get_type()).unwrap(),
            expected_key
        );
        assert_eq!(
            get_instance_handle_from_serialized_key(&expected_key, &Simple::get_type()).unwrap(),
            expected_instance_handle
        )
    }

    #[test]
    fn from_serialized_foo_complex_be() {
        let data = [
            0, 0, 0, 0, //rtps header
            0, 0, 0, 0, 0, 0, 0, 2, //field1 (i64)
            0, 3, 0, 0, //key_field1 (u16) | padding (2B)
            0, 0, 0, 4, //field2 (u32)
            5, 6, //key_field2 (u8, u8)
        ];
        let expected_instance_handle =
            InstanceHandle::new([0, 3, 5, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(
            get_instance_handle_from_serialized_foo(&data, &Complex::get_type()).unwrap(),
            expected_instance_handle
        );
        let expected_key = vec![0, 1, 0, 0, 3, 0, 5, 6];
        assert_eq!(
            get_serialized_key_from_serialized_foo(&data, &Complex::get_type()).unwrap(),
            expected_key
        );
        assert_eq!(
            get_instance_handle_from_serialized_key(&expected_key, &Complex::get_type()).unwrap(),
            expected_instance_handle
        )
    }

    #[test]
    fn from_serialized_foo_complex_le() {
        let data = [
            0, 1, 0, 0, //rtps header
            2, 0, 0, 0, 0, 0, 0, 0, //field1 (i64)
            3, 0, 0, 0, //key_field1 (u16) | padding (2B)
            4, 0, 0, 0, //field2 (u32)
            5, 6, //key_field2 (u8, u8)
        ];
        let expected_instance_handle =
            InstanceHandle::new([0, 3, 5, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(
            get_instance_handle_from_serialized_foo(&data, &Complex::get_type()).unwrap(),
            expected_instance_handle
        );
        let expected_key = vec![0, 1, 0, 0, 3, 0, 5, 6];
        assert_eq!(
            get_serialized_key_from_serialized_foo(&data, &Complex::get_type()).unwrap(),
            expected_key
        );
        assert_eq!(
            get_instance_handle_from_serialized_key(&expected_key, &Complex::get_type()).unwrap(),
            expected_instance_handle
        )
    }

    #[derive(TypeSupport)]
    #[dust_dds(extensibility = "final")]
    struct Large {
        #[dust_dds(key)]
        _key_field1: i64,
        #[dust_dds(key)]
        _key_field2: i64,
        #[dust_dds(key)]
        _key_field3: i64,
    }

    #[test]
    fn large_key_be() {
        let data = [
            0, 0, 0, 0, //rtps header
            0, 0, 0, 0, 0, 0, 0, 1, //key_field1 (i64)
            0, 0, 0, 0, 0, 0, 0, 2, //key_field1 (i64)
            0, 0, 0, 0, 0, 0, 0, 3, //key_field1 (i64)
        ];
        let expected_instance_handle = InstanceHandle::new(
            md5::compute([
                0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3,
            ])
            .into(),
        );
        assert_eq!(
            get_instance_handle_from_serialized_foo(&data, &Large::get_type()).unwrap(),
            expected_instance_handle
        );
        let expected_key = [
            0, 1, 0, 0, // RTPS header
            1, 0, 0, 0, 0, 0, 0, 0, //key_field1 (i64)
            2, 0, 0, 0, 0, 0, 0, 0, //key_field1 (i64)
            3, 0, 0, 0, 0, 0, 0, 0, //key_field1 (i64)
        ];
        assert_eq!(
            get_serialized_key_from_serialized_foo(&data, &Large::get_type()).unwrap(),
            expected_key
        );
        assert_eq!(
            get_instance_handle_from_serialized_key(&expected_key, &Large::get_type()).unwrap(),
            expected_instance_handle
        )
    }

    #[test]
    fn large_key_le() {
        let data = [
            0, 1, 0, 0, //rtps header
            1, 0, 0, 0, 0, 0, 0, 0, //key_field1 (i64)
            2, 0, 0, 0, 0, 0, 0, 0, //key_field1 (i64)
            3, 0, 0, 0, 0, 0, 0, 0, //key_field1 (i64)
        ];
        let expected_instance_handle = InstanceHandle::new(
            md5::compute([
                0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3,
            ])
            .into(),
        );
        assert_eq!(
            get_instance_handle_from_serialized_foo(&data, &Large::get_type()).unwrap(),
            expected_instance_handle
        );
        let expected_key = [
            0, 1, 0, 0, // RTPS header
            1, 0, 0, 0, 0, 0, 0, 0, //key_field1 (i64)
            2, 0, 0, 0, 0, 0, 0, 0, //key_field1 (i64)
            3, 0, 0, 0, 0, 0, 0, 0, //key_field1 (i64)
        ];
        assert_eq!(
            get_serialized_key_from_serialized_foo(&data, &Large::get_type()).unwrap(),
            expected_key
        );
        assert_eq!(
            get_instance_handle_from_serialized_key(&expected_key, &Large::get_type()).unwrap(),
            expected_instance_handle
        )
    }

    #[derive(TypeSupport)]
    struct NestedArray {
        #[dust_dds(key)]
        _key_field: [Nested; 2],
    }

    #[test]
    fn from_serialized_foo_nested_array() {
        let data = [
            0, 1, 0, 0, //rtps header
            1, 2, 3, 4, //key_field (u8, u8) | (u8, u8)
        ];
        let expected_instance_handle =
            InstanceHandle::new([1, 2, 3, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(
            get_instance_handle_from_serialized_foo(&data, &NestedArray::get_type()).unwrap(),
            expected_instance_handle
        );
        let expected_key = vec![0, 1, 0, 0, 1, 2, 3, 4];
        assert_eq!(
            get_serialized_key_from_serialized_foo(&data, &NestedArray::get_type()).unwrap(),
            expected_key
        );
        assert_eq!(
            get_instance_handle_from_serialized_key(&expected_key, &NestedArray::get_type())
                .unwrap(),
            expected_instance_handle
        )
    }

    #[derive(TypeSupport)]
    struct NestedSequence {
        #[dust_dds(key)]
        _key_field: Vec<Nested>,
    }

    #[test]
    fn from_serialized_foo_nested_sequence() {
        let data = [
            0, 1, 0, 0, //rtps header
            2, 0, 0, 0, // sequence length
            1, 2, 3, 4, //key_field (u8, u8) | (u8, u8)
        ];
        let expected_instance_handle =
            InstanceHandle::new([0, 0, 0, 2, 1, 2, 3, 4, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(
            get_instance_handle_from_serialized_foo(&data, &NestedSequence::get_type()).unwrap(),
            expected_instance_handle
        );
        let expected_key = vec![0, 1, 0, 0, 2, 0, 0, 0, 1, 2, 3, 4];
        assert_eq!(
            get_serialized_key_from_serialized_foo(&data, &NestedSequence::get_type()).unwrap(),
            expected_key
        );
        assert_eq!(
            get_instance_handle_from_serialized_key(&expected_key, &NestedSequence::get_type())
                .unwrap(),
            expected_instance_handle
        )
    }

    #[derive(TypeSupport)]
    #[dust_dds(extensibility = "final")]
    struct BasicTypes {
        #[dust_dds(key)]
        _f1: bool,
        #[dust_dds(key)]
        _f2: i8,
        #[dust_dds(key)]
        _f3: i16,
        #[dust_dds(key)]
        _f4: i32,
        #[dust_dds(key)]
        _f5: i64,
        #[dust_dds(key)]
        _f6: u8,
        #[dust_dds(key)]
        _f7: u16,
        #[dust_dds(key)]
        _f8: u32,
        #[dust_dds(key)]
        _f9: u64,
        #[dust_dds(key)]
        _f10: f32,
        #[dust_dds(key)]
        _f11: f64,
        #[dust_dds(key)]
        _f12: char,
    }

    #[test]
    fn basic_types_be() {
        let data = [
            0, 0, 0, 3, //rtps header (incl. padding length)
            1, 2, 0, 3, 0, 0, 0, 4, // f1: bool | f2: i8 | f3: i16 | f4: i32
            0, 0, 0, 0, 0, 0, 0, 5, // f5: i64
            6, 0, 0, 7, 0, 0, 0, 8, // f6: u8 | padding (1 byte) | f7: u16 | f8: u32
            0, 0, 0, 0, 0, 0, 0, 9, // f9: u64
            0x3F, 0x80, 0x00, 0x00, 0, 0, 0, 0, // f10: f32 | padding (4 bytes)
            0x3F, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // f11: f64
            b'a', 0, 0, 0, // f12: char | padding 3 bytes
        ];
        let expected_instance_handle = InstanceHandle::new(
            md5::compute(&[
                1, 2, 0, 3, // f1: bool | f2: i8 | f3: i16
                0, 0, 0, 4, // f4: i32
                0, 0, 0, 0, // f5-1: i64
                0, 0, 0, 5, // f5-2: i64
                6, 0, 0, 7, // f6: u8 | padding (1 byte) | f7: u16
                0, 0, 0, 8, // f8: u32
                0, 0, 0, 0, // f9-1: u64
                0, 0, 0, 9, // f9-2: u64
                0x3F, 0x80, 0x00, 0x00, // f10: f32
                0x3F, 0xF0, 0x00, 0x00, // f11-1: f64
                0x00, 0x00, 0x00, 0x00, // f11-2: f64
                b'a', // f12: char
            ])
            .into(),
        );
        assert_eq!(
            get_instance_handle_from_serialized_foo(&data, &BasicTypes::get_type()).unwrap(),
            expected_instance_handle
        );
        let expected_key = vec![
            0, 1, 0, 3, // RTPS header (incl. padding length)
            1, 2, 3, 0, 4, 0, 0, 0, // f1: bool | f2: i8 | f3: i16 | f4: i32
            5, 0, 0, 0, 0, 0, 0, 0, // f5: i64
            6, 0, 7, 0, 8, 0, 0, 0, // f6: u8 | padding (1 byte) | f7: u16 | f8: u32
            9, 0, 0, 0, 0, 0, 0, 0, // f9: u64
            0x00, 0x00, 0x80, 0x3F, 0, 0, 0, 0, // f10: f32 | padding (4 bytes)
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x3F, // f11: f64
            b'a', 0, 0, 0, // f12: char
        ];
        assert_eq!(
            get_serialized_key_from_serialized_foo(&data, &BasicTypes::get_type()).unwrap(),
            expected_key
        );
        assert_eq!(
            get_instance_handle_from_serialized_key(&expected_key, &BasicTypes::get_type())
                .unwrap(),
            expected_instance_handle
        )
    }

    #[test]
    fn basic_types_le() {
        let data = [
            0, 1, 0, 3, //rtps header (incl. padding length)
            1, 2, 3, 0, 4, 0, 0, 0, // f1: bool | f2: i8 | f3: i16 | f4: i32
            5, 0, 0, 0, 0, 0, 0, 0, // f5: i64
            6, 0, 7, 0, 8, 0, 0, 0, // f6: u8 | padding (1 byte) | f7: u16 | f8: u32
            9, 0, 0, 0, 0, 0, 0, 0, // f9: u64
            0x00, 0x00, 0x80, 0x3F, 0, 0, 0, 0, // f10: f32 | padding (4 bytes)
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x3F, // f11: f64
            b'a', 0, 0, 0, // f12: char
        ];
        let expected_instance_handle = InstanceHandle::new(
            md5::compute(&[
                1, 2, 0, 3, // f1: bool | f2: i8 | f3: i16
                0, 0, 0, 4, // f4: i32
                0, 0, 0, 0, // f5-1: i64
                0, 0, 0, 5, // f5-2: i64
                6, 0, 0, 7, // f6: u8 | padding (1 byte) | f7: u16
                0, 0, 0, 8, // f8: u32
                0, 0, 0, 0, // f9-1: u64
                0, 0, 0, 9, // f9-2: u64
                0x3F, 0x80, 0x00, 0x00, // f10: f32
                0x3F, 0xF0, 0x00, 0x00, // f11-1: f64
                0x00, 0x00, 0x00, 0x00, // f11-2: f64
                b'a', // f12: char
            ])
            .into(),
        );
        assert_eq!(
            get_instance_handle_from_serialized_foo(&data, &BasicTypes::get_type()).unwrap(),
            expected_instance_handle
        );
        let expected_key = vec![
            0, 1, 0, 3, // RTPS header (incl. padding length)
            1, 2, 3, 0, 4, 0, 0, 0, // f1: bool | f2: i8 | f3: i16 | f4: i32
            5, 0, 0, 0, 0, 0, 0, 0, // f5: i64
            6, 0, 7, 0, 8, 0, 0, 0, // f6: u8 | padding (1 byte) | f7: u16 | f8: u32
            9, 0, 0, 0, 0, 0, 0, 0, // f9: u64
            0x00, 0x00, 0x80, 0x3F, 0, 0, 0, 0, // f10: f32 | padding (4 bytes)
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x3F, // f11: f64
            b'a', 0, 0, 0, // f12: char
        ];
        assert_eq!(
            get_serialized_key_from_serialized_foo(&data, &BasicTypes::get_type()).unwrap(),
            expected_key
        );
        assert_eq!(
            get_instance_handle_from_serialized_key(&expected_key, &BasicTypes::get_type())
                .unwrap(),
            expected_instance_handle
        )
    }
}
