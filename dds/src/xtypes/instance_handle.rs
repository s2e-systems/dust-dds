use super::{
    deserialize::XTypesDeserialize, deserializer::DeserializeSequence, dynamic_type::MemberDescriptor, serialize::XTypesSerializer, serializer::SerializeFinalStruct, type_object::TypeIdentifier, xcdr_deserializer::Xcdr1BeDeserializer, xcdr_serializer::Xcdr1LeSerializer
};
use crate::{
    infrastructure::instance::InstanceHandle,
    xtypes::{
        deserializer::XTypesDeserializer, dynamic_type::DynamicType, error::XTypesError,
        serialize::Write, xcdr_deserializer::Xcdr1LeDeserializer,
        xcdr_serializer::Xcdr2BeSerializer,
    },
};
use std::io::BufRead;

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
    type_identifier: &TypeIdentifier,
    is_key_field: bool,
    de: &mut T,
    serializer: &mut impl SerializeFinalStruct,
) -> Result<(), XTypesError>
where
    for<'b> &'b mut T: XTypesDeserializer<'a>,
{
    match type_identifier {
        TypeIdentifier::TkNone => todo!(),
        TypeIdentifier::TkBoolean => {
            let v = de.deserialize_boolean()?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        TypeIdentifier::TkByteType => todo!(),
        TypeIdentifier::TkInt8Type => todo!(),
        TypeIdentifier::TkInt16Type => todo!(),
        TypeIdentifier::TkInt32Type => {
            let v = de.deserialize_int32()?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        TypeIdentifier::TkInt64Type => {
            let v = de.deserialize_int64()?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        TypeIdentifier::TkUint8Type => {
            let v = de.deserialize_uint8()?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        TypeIdentifier::TkUint16Type => {
            let v = de.deserialize_uint16()?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        TypeIdentifier::TkUint32Type => {
            let v = de.deserialize_uint32()?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        TypeIdentifier::TkUint64Type => {
            let v = de.deserialize_uint64()?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        TypeIdentifier::TkFloat32Type => todo!(),
        TypeIdentifier::TkFloat64Type => todo!(),
        TypeIdentifier::TkFloat128Type => todo!(),
        TypeIdentifier::TkChar8Type => todo!(),
        TypeIdentifier::TkChar16Type => todo!(),
        TypeIdentifier::TiString8Small { .. } => {
            let v = de.deserialize_string()?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        TypeIdentifier::TiString16Small { .. } => todo!(),
        TypeIdentifier::TiString8Large { .. } => todo!(),
        TypeIdentifier::TiString16Large { .. } => todo!(),
        TypeIdentifier::TiPlainSequenceSmall { seq_sdefn } => {
            let len = de.deserialize_sequence()?.len() as u32;
            if is_key_field {
                serializer.serialize_field(&len, "")?;

                for _ in 0..len {
                    deserialize_and_serialize_if_key_field(
                        &seq_sdefn.element_identifier,
                        is_key_field,
                        de,
                        serializer,
                    )?;
                }
            }
        }
        TypeIdentifier::TiPlainSequenceLarge { .. } => todo!(),
        TypeIdentifier::TiPlainArraySmall { array_sdefn } => {
            for _ in 0..array_sdefn.array_bound_seq[0] {
                deserialize_and_serialize_if_key_field(
                    &array_sdefn.element_identifier,
                    is_key_field,
                    de,
                    serializer,
                )?;
            }
        }
        TypeIdentifier::TiPlainArrayLarge { .. } => todo!(),
        TypeIdentifier::TiPlainMapSmall { .. } => todo!(),
        TypeIdentifier::TiPlainMapLarge { .. } => todo!(),
        TypeIdentifier::TiStronglyConnectedComponent { .. } => todo!(),
        TypeIdentifier::EkComplete { complete } => {
            push_to_key(complete.as_ref(), serializer, de)?;
        }
        TypeIdentifier::EkMinimal { .. } => todo!(),
    }
    Ok(())
}

fn push_to_key<'a, T>(
    dynamic_type: &dyn DynamicType,
    serializer: &mut impl SerializeFinalStruct,
    de: &mut T,
) -> Result<(), XTypesError>
where
    for<'b> &'b mut T: XTypesDeserializer<'a>,
{
    for member_descriptor in dynamic_type.into_iter() {
        let member_descriptor = member_descriptor?;
        deserialize_and_serialize_if_key_field(
            member_descriptor.type_,
            member_descriptor.is_key,
            de,
            serializer,
        )?;
    }
    Ok(())
}

fn push_to_key_for_key<'a, T>(
    dynamic_type: &dyn DynamicType,
    serializer: &mut impl SerializeFinalStruct,
    de: &mut T,
) -> Result<(), XTypesError>
where
    for<'b> &'b mut T: XTypesDeserializer<'a>,
{
    for member_descriptor in dynamic_type.into_iter() {
        let member_descriptor = member_descriptor?;
        if member_descriptor.is_key {
            deserialize_and_serialize_if_key_field(member_descriptor.type_, true, de, serializer)?;
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
    dynamic_type: &'a dyn DynamicType,
    range: core::ops::Range<u32>,
}
impl<'a> Iterator for MemberDescriptorIter<'a> {
    type Item = Result<MemberDescriptor<'a>, XTypesError>;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.range.next()?;
        match self.dynamic_type.get_member_by_index(i) {
            Ok(member) => {
                match member.get_descriptor() {
                    Ok(descriptor) => Some(Ok(descriptor)),
                    Err(err) => return Some(Err(err)),
                }
            },
            Err(err) => return Some(Err(err)),
        }
    }
}

impl<'a> IntoIterator for &'a dyn DynamicType {
    type Item = Result<MemberDescriptor<'a>, XTypesError>;
    type IntoIter = MemberDescriptorIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        MemberDescriptorIter {
            dynamic_type: self,
            range: 0..self.get_member_count(),
        }
    }
}

fn push_to_key_parameter_list_le<'a>(
    dynamic_type: &dyn DynamicType,
    serializer: &mut impl SerializeFinalStruct,
    data: &[u8],
) -> Result<(), XTypesError> {
    for descriptor in dynamic_type.into_iter() {
        let descriptor = descriptor?;
        if descriptor.is_key {
            let buffer = go_to_pid_le(&data, descriptor.id)?;
            let mut de = Xcdr1LeDeserializer::new(buffer);
            deserialize_and_serialize_if_key_field(descriptor.type_, true, &mut de, serializer)?;
        }
    }
    Ok(())
}

fn push_to_key_parameter_list_be<'a>(
    dynamic_type: &dyn DynamicType,
    serializer: &mut impl SerializeFinalStruct,
    data: &[u8],
) -> Result<(), XTypesError> {
    for descriptor in dynamic_type.into_iter() {
        let descriptor = descriptor?;
        if descriptor.is_key {
            let buffer = go_to_pid_be(&data, descriptor.id)?;
            let mut de = Xcdr1BeDeserializer::new(buffer);
            deserialize_and_serialize_if_key_field(descriptor.type_, true, &mut de, serializer)?;
        }
    }
    Ok(())
}

type RepresentationIdentifier = [u8; 2];
const CDR_BE: RepresentationIdentifier = [0x00, 0x00];
const CDR_LE: RepresentationIdentifier = [0x00, 0x01];
const _CDR2_BE: RepresentationIdentifier = [0x00, 0x06];
const _CDR2_LE: RepresentationIdentifier = [0x00, 0x07];
const _D_CDR2_BE: RepresentationIdentifier = [0x00, 0x08];
const _D_CDR2_LE: RepresentationIdentifier = [0x00, 0x09];
const PL_CDR_BE: RepresentationIdentifier = [0x00, 0x02];
const PL_CDR_LE: RepresentationIdentifier = [0x00, 0x03];

pub fn get_instance_handle_from_serialized_key(
    mut data: &[u8],
    dynamic_type: &dyn DynamicType,
) -> Result<InstanceHandle, XTypesError> {
    let mut md5_collection = Md5 {
        key: [0; 16],
        context: md5::Context::new(),
        length: 0,
    };
    {
        let representation_identifier = [data[0], data[1]];
        data.consume(4);
        let mut serializer = Xcdr2BeSerializer::new(&mut md5_collection);
        let mut s = serializer.serialize_final_struct()?;
        match representation_identifier {
            CDR_BE => {
                push_to_key_for_key(dynamic_type, &mut s, &mut Xcdr1BeDeserializer::new(data))?
            }
            CDR_LE => {
                push_to_key_for_key(dynamic_type, &mut s, &mut Xcdr1LeDeserializer::new(data))?
            }
            _ => panic!("representation_identifier not supported"),
        }
    }
    Ok(InstanceHandle::new(md5_collection.into_key()))
}

pub fn get_instance_handle_from_serialized_foo(
    mut data: &[u8],
    dynamic_type: &dyn DynamicType,
) -> Result<InstanceHandle, XTypesError> {
    let mut md5_collection = Md5 {
        key: [0; 16],
        context: md5::Context::new(),
        length: 0,
    };
    {
        let representation_identifier = [data[0], data[1]];
        data.consume(4);
        let mut serializer = Xcdr2BeSerializer::new(&mut md5_collection);
        let mut s = serializer.serialize_final_struct()?;
        match representation_identifier {
            CDR_BE => push_to_key(dynamic_type, &mut s, &mut Xcdr1BeDeserializer::new(data))?,
            CDR_LE => push_to_key(dynamic_type, &mut s, &mut Xcdr1LeDeserializer::new(data))?,
            PL_CDR_BE => push_to_key_parameter_list_be(dynamic_type, &mut s, data)?,
            PL_CDR_LE => push_to_key_parameter_list_le(dynamic_type, &mut s, data)?,
            _ => panic!("representation_identifier not supported"),
        }
    }
    Ok(InstanceHandle::new(md5_collection.into_key()))
}

pub fn get_serialized_key_from_serialized_foo(
    mut data: &[u8],
    dynamic_type: &dyn DynamicType,
) -> Result<Vec<u8>, XTypesError> {
    let mut collection = Vec::new();
    {
        let representation_identifier = [data[0], data[1]];
        collection.extend_from_slice(&CDR_LE);
        collection.extend_from_slice(&[0, 0]);
        data.consume(4);

        let mut serializer = Xcdr1LeSerializer::new(&mut collection);
        let mut s = serializer.serialize_final_struct()?;

        match representation_identifier {
            CDR_BE => push_to_key(dynamic_type, &mut s, &mut Xcdr1BeDeserializer::new(data))?,
            CDR_LE => push_to_key(dynamic_type, &mut s, &mut Xcdr1LeDeserializer::new(data))?,
            PL_CDR_BE => push_to_key_parameter_list_be(dynamic_type, &mut s, data)?,
            PL_CDR_LE => push_to_key_parameter_list_le(dynamic_type, &mut s, data)?,
            _ => panic!("representation_identifier not supported"),
        }
    }
    Ok(collection)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topic_definition::type_support::TypeSupport;
    use dust_dds_derive::TypeSupport;

    #[derive(TypeSupport)]
    #[dust_dds(extensibility = "Mutable")]
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
    #[dust_dds(extensibility = "Final")]
    struct Nested {
        #[dust_dds(key)]
        _x: u8,
        #[dust_dds(key)]
        _y: u8,
    }
    #[derive(TypeSupport)]
    #[dust_dds(extensibility = "Final")]
    struct Complex {
        _field1: i64,
        #[dust_dds(key)]
        _key_field1: u16,
        _field2: u32,
        #[dust_dds(key)]
        _key_field2: Nested,
    }

    #[derive(TypeSupport)]
    #[dust_dds(extensibility = "Final")]
    struct Simple {
        #[dust_dds(key)]
        _key_field1: i64,
        #[dust_dds(key)]
        _key_field2: i64,
    }

    #[test]
    fn simple_key_be() {
        let data = [
            0, 0, 0, 0, //rtps header
            0, 0, 0, 0, 0, 0, 0, 1, //key_field1 (i64)
            0, 0, 0, 0, 0, 0, 0, 2, //key_field1 (i64)
        ];
        let expected_instance_handle =
            InstanceHandle::new([0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2]);
        assert_eq!(
            get_instance_handle_from_serialized_foo(&data, &Simple::get_type()).unwrap(),
            expected_instance_handle
        );
        let expected_key = vec![
            0, 1, 0, 0, // RTPS header
            1, 0, 0, 0, 0, 0, 0, 0, // key_field1
            2, 0, 0, 0, 0, 0, 0, 0, // key_field2
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
        let data = [
            0, 1, 0, 0, //rtps header
            1, 0, 0, 0, 0, 0, 0, 0, //key_field1 (i64)
            2, 0, 0, 0, 0, 0, 0, 0, //key_field1 (i64)
        ];
        let expected_instance_handle =
            InstanceHandle::new([0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2]);
        assert_eq!(
            get_instance_handle_from_serialized_foo(&data, &Simple::get_type()).unwrap(),
            expected_instance_handle
        );
        let expected_key = vec![
            0, 1, 0, 0, // RTPS header
            1, 0, 0, 0, 0, 0, 0, 0, // key_field1
            2, 0, 0, 0, 0, 0, 0, 0, // key_field2
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
    #[dust_dds(extensibility = "Final")]
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


    
    // #[derive(TypeSupport)]
    // #[dust_dds(extensibility = "Final")]
    // struct EveryBasicType {
    //     #[dust_dds(key)]
    //     _f2: bool,
    //     #[dust_dds(key)]
    //     _f4: i8,
    //     #[dust_dds(key)]
    //     _f5: i16,
    //     #[dust_dds(key)]
    //     _f6: i32,
    //     #[dust_dds(key)]
    //     _f7: i64,
    //     #[dust_dds(key)]
    //     _f8: u8,
    //     #[dust_dds(key)]
    //     _f9: u16,
    //     #[dust_dds(key)]
    //     _f10: u32,
    //     #[dust_dds(key)]
    //     _f11: u64,
    //     #[dust_dds(key)]
    //     _f12: f32,
    //     #[dust_dds(key)]
    //     _f13: f64,
    // }

    // #[test]
    // fn every_basic_type_be() {
    //     let data = [
    //         0, 0, 0, 0, //rtps header
    //         1, // f2: bool
    //         2, 0, // f4: i8
    //         3, 0, 0, 0, // f5: i16
    //         4, 0, 0, 0, 0, 0, 0, 0, // f6: i32
    //         5, 0, 0, 0, 0, 0, 0, 0, // f7: i64
    //         6, // f8: u8
    //         7, 0, 0, 0, // f9: u16
    //         8, 0, 0, 0, 0, 0, 0, 0, // f10: u32
    //         9, 0, 0, 0, 0, 0, 0, 0, // f11: u64
    //          // f12: f32
    //          // f13: f64
    //     ];
    //     let expected_instance_handle =
    //         InstanceHandle::new([0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2]);
    //     assert_eq!(
    //         get_instance_handle_from_serialized_foo(&data, &Simple::get_type()).unwrap(),
    //         expected_instance_handle
    //     );
    //     let expected_key = vec![
    //         0, 1, 0, 0, // RTPS header
    //         1, 0, 0, 0, 0, 0, 0, 0, // key_field1
    //         2, 0, 0, 0, 0, 0, 0, 0, // key_field2
    //     ];
    //     assert_eq!(
    //         get_serialized_key_from_serialized_foo(&data, &Simple::get_type()).unwrap(),
    //         expected_key
    //     );
    //     assert_eq!(
    //         get_instance_handle_from_serialized_key(&expected_key, &Simple::get_type()).unwrap(),
    //         expected_instance_handle
    //     )
    // }
}
