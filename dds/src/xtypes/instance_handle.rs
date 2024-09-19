use super::{
    deserializer::{DeserializeMutableStruct, DeserializeSequence},
    serialize::XTypesSerializer,
    serializer::SerializeFinalStruct,
    type_object::TypeKind,
    xcdr_deserializer::Xcdr1BeDeserializer,
};
use crate::{
    implementation::data_representation_builtin_endpoints::parameter_id_values::PID_PARTICIPANT_GUID,
    infrastructure::instance::InstanceHandle,
    topic_definition::type_support::TypeSupport,
    xtypes::{
        deserializer::XTypesDeserializer,
        dynamic_type::DynamicType,
        error::XTypesError,
        serialize::{Write, XTypesSerialize},
        type_object,
        xcdr_deserializer::Xcdr1LeDeserializer,
        xcdr_serializer::{Xcdr1LeSerializer, Xcdr2BeSerializer},
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
    dynamic_type: &dyn DynamicType,
    is_key_field: bool,
    de: &mut T,
    serializer: &mut impl SerializeFinalStruct,
) -> Result<(), XTypesError>
where
    for<'b> &'b mut T: XTypesDeserializer<'a>,
{
    match dynamic_type.get_kind() {
        type_object::TK_BOOLEAN => {
            let v = de.deserialize_boolean()?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        type_object::TK_INT64 => {
            let v = de.deserialize_int64()?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        type_object::TK_UINT64 => {
            let v = de.deserialize_uint64()?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        type_object::TK_UINT16 => {
            let v = de.deserialize_uint16()?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        type_object::TK_INT32 => {
            let v = de.deserialize_int32()?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        type_object::TK_UINT32 => {
            let v = de.deserialize_uint32()?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        type_object::TK_UINT8 => {
            let v = de.deserialize_uint8()?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        type_object::TK_SEQUENCE => {
            let len = de.deserialize_sequence()?.len();
            for _ in 0..len {
                push_to_key(dynamic_type, serializer, de)?;
            }
        }
        type_object::TK_STRUCTURE => {
            push_to_key(dynamic_type, serializer, de)?;
        }
        type_object::TK_STRING8 => {
            let v = de.deserialize_string()?;
            if is_key_field {
                serializer.serialize_field(&v, "")?;
            }
        }
        _ => panic!(),
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
    for id in 0..dynamic_type.get_member_count() {
        let member = dynamic_type.get_member_by_index(id)?;
        deserialize_and_serialize_if_key_field(
            member.get_descriptor()?.type_,
            member.get_descriptor()?.is_key,
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
    for id in 0..dynamic_type.get_member_count() {
        let member = dynamic_type.get_member_by_index(id)?;
        if member.get_descriptor()?.is_key {
            deserialize_and_serialize_if_key_field(
                member.get_descriptor()?.type_,
                true,
                de,
                serializer,
            )?;
        }
    }
    Ok(())
}

fn push_to_key_built_in<'a>(
    dynamic_type: &dyn DynamicType,
    serializer: &mut impl SerializeFinalStruct,
    deserializer: &mut Xcdr1LeDeserializer<'a>,
) -> Result<(), XTypesError> {
    let mut de = deserializer.deserialize_mutable_struct()?;
    let pid = dynamic_type.get_member_by_index(0)?.get_descriptor()?.id as u16;
    let key: InstanceHandle = de.deserialize_field(pid, "")?;
    serializer.serialize_field(&key, "")?;
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
            PL_CDR_BE => {
                push_to_key_built_in(dynamic_type, &mut s, &mut Xcdr1LeDeserializer::new(data))?
            }
            PL_CDR_LE => {
                push_to_key_built_in(dynamic_type, &mut s, &mut Xcdr1LeDeserializer::new(data))?
            }
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
    collection.append(&mut vec![0, 1, 0, 0]);
    {
        let representation_identifier = [data[0], data[1]];
        data.consume(4);
        let mut serializer = Xcdr2BeSerializer::new(&mut collection);
        let mut s = serializer.serialize_final_struct()?;
        match representation_identifier {
            CDR_BE => push_to_key(dynamic_type, &mut s, &mut Xcdr1BeDeserializer::new(data))?,
            CDR_LE => push_to_key(dynamic_type, &mut s, &mut Xcdr1LeDeserializer::new(data))?,
            PL_CDR_BE => {
                push_to_key_built_in(dynamic_type, &mut s, &mut Xcdr1LeDeserializer::new(data))?
            }
            PL_CDR_LE => {
                push_to_key_built_in(dynamic_type, &mut s, &mut Xcdr1LeDeserializer::new(data))?
            }
            _ => panic!("representation_identifier not supported"),
        }
    }
    Ok(collection)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dust_dds_derive::TypeSupport;

    #[derive(XTypesSerialize, TypeSupport)]
    //@extensibility(FINAL) @nested
    struct Nested {
        #[dust_dds(key)]
        x: u8,
        #[dust_dds(key)]
        y: u8,
    }
    #[derive(XTypesSerialize, TypeSupport)]
    //@extensibility(FINAL)
    struct Complex {
        field1: i64,
        #[dust_dds(key)]
        key_field1: u16,
        field2: u32,
        #[dust_dds(key)]
        key_field2: Nested,
    }

    #[derive(XTypesSerialize, TypeSupport)]
    //@extensibility(FINAL)
    struct Simple {
        #[dust_dds(key)]
        key_field1: i64,
        #[dust_dds(key)]
        key_field2: i64,
    }

    #[test]
    fn simple_key() {
        let v = Simple {
            key_field1: 1,
            key_field2: 2,
        };
        let data = [
            0, 1, 0, 0, //rtps header
            1, 0, 0, 0, 0, 0, 0, 0, //key_field1 (i64)
            2, 0, 0, 0, 0, 0, 0, 0, //key_field1 (i64)
        ];
        let mut collection = data[..4].to_vec();
        v.serialize(&mut Xcdr1LeSerializer::new(&mut collection))
            .unwrap();
        assert_eq!(&data, collection.as_slice());
        let key_data = InstanceHandle::new([0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2]);
        assert_eq!(
            get_instance_handle_from_serialized_foo(&data, &Simple::get_type()).unwrap(),
            key_data
        );
    }

    #[test]
    fn from_serialized_foo_complex() {
        let v = Complex {
            field1: 2,
            key_field1: 3,
            field2: 4,
            key_field2: Nested { x: 5, y: 6 },
        };
        let data = [
            0, 1, 0, 0, //rtps header
            2, 0, 0, 0, 0, 0, 0, 0, //field1 (i64)
            3, 0, 0, 0, //key_field1 (u16) | padding (2B)
            4, 0, 0, 0, //field2 (u32)
            5, 6, //key_field2 (u8, u8)
        ];
        let mut collection = data[..4].to_vec();
        v.serialize(&mut Xcdr1LeSerializer::new(&mut collection))
            .unwrap();
        assert_eq!(&data, collection.as_slice());
        assert_eq!(
            get_instance_handle_from_serialized_foo(&data, &Complex::get_type()).unwrap(),
            InstanceHandle::new([0, 3, 5, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
        );
        assert_eq!(
            get_serialized_key_from_serialized_foo(&data, &Complex::get_type()).unwrap(),
            vec![0, 1, 0, 0, 0, 3, 5, 6]
        )
    }

    #[test]
    fn instance_handle_from_serialized_key() {
        let data = [
            0, 1, 0, 0, //rtps header
            3, 0, //key_field1 (u16) | padding (2B)
            5, 6, //key_field2 (u8, u8)
        ];
        assert_eq!(
            get_instance_handle_from_serialized_key(&data, &Complex::get_type()).unwrap(),
            InstanceHandle::new([0, 3, 5, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
        )
    }

    #[derive(XTypesSerialize, TypeSupport)]
    //@extensibility(FINAL)
    struct Large {
        #[dust_dds(key)]
        key_field1: i64,
        #[dust_dds(key)]
        key_field2: i64,
        #[dust_dds(key)]
        key_field3: i64,
    }

    #[test]
    fn large_key() {
        let v = Large {
            key_field1: 1,
            key_field2: 2,
            key_field3: 3,
        };
        let data = [
            0, 1, 0, 0, //rtps header
            1, 0, 0, 0, 0, 0, 0, 0, //key_field1 (i64)
            2, 0, 0, 0, 0, 0, 0, 0, //key_field1 (i64)
            3, 0, 0, 0, 0, 0, 0, 0, //key_field1 (i64)
        ];
        let mut collection = data[..4].to_vec();
        v.serialize(&mut Xcdr1LeSerializer::new(&mut collection))
            .unwrap();
        assert_eq!(&data, collection.as_slice());
        let key_data = [
            0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3,
        ];
        let instance_handle =
            get_instance_handle_from_serialized_foo(&data, &Large::get_type()).unwrap();
        assert_eq!(
            <[u8; 16]>::from(instance_handle),
            md5::compute(key_data).as_slice()
        );
    }
}
