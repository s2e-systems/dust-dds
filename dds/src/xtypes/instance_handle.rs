use super::{
    deserializer::{DeserializeArray, DeserializeSequence},
    serialize::XTypesSerializer,
    serializer::SerializeFinalStruct,
    type_object::TypeIdentifier,
    xcdr_deserializer::Xcdr1BeDeserializer,
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
    dynamic_type: &TypeIdentifier,
    is_key_field: bool,
    de: &mut T,
    serializer: &mut impl SerializeFinalStruct,
) -> Result<(), XTypesError>
where
    for<'b> &'b mut T: XTypesDeserializer<'a>,
{
    match dynamic_type {
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
        TypeIdentifier::TiPlainSequenceSmall { .. } => {
            let len = de.deserialize_sequence()?.len();
            for _ in 0..len {
                push_to_key(dynamic_type, serializer, de)?;
            }
        }
        TypeIdentifier::TiPlainSequenceLarge { .. } => todo!(),
        TypeIdentifier::TiPlainArraySmall { .. } => {
            let mut array_des = de.deserialize_array()?;
            for _ in 0..16 {
                //descriptor.type_.get_member_count() {
                let v: u8 = array_des.deserialize_element()?;
                serializer.serialize_field(&v, "")?;
            }
        }
        TypeIdentifier::TiPlainArrayLarge { .. } => todo!(),
        TypeIdentifier::TiPlainMapSmall { .. } => todo!(),
        TypeIdentifier::TiPlainMapLarge { .. } => todo!(),
        TypeIdentifier::TiStronglyConnectedComponent { .. } => todo!(),
        TypeIdentifier::EkComplete { .. } => {
            push_to_key(dynamic_type, serializer, de)?;
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

fn push_to_key_parameter_list<'a>(
    dynamic_type: &dyn DynamicType,
    serializer: &mut impl SerializeFinalStruct,
    data: &[u8],
) -> Result<(), XTypesError> {
    for id in 0..dynamic_type.get_member_count() {
        let member = dynamic_type.get_member_by_index(id)?;
        let descriptor = member.get_descriptor()?;
        if descriptor.is_key {
            let buffer = go_to_pid_le(&data, descriptor.id)?;
            let mut de = Xcdr1LeDeserializer::new(buffer);
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
            PL_CDR_BE => push_to_key_parameter_list(dynamic_type, &mut s, data)?,
            PL_CDR_LE => push_to_key_parameter_list(dynamic_type, &mut s, data)?,
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
        let representation_options = [data[2], data[3]];
        collection.extend_from_slice(&representation_identifier);
        collection.extend_from_slice(&representation_options);
        data.consume(4);
        let mut serializer = Xcdr2BeSerializer::new(&mut collection);
        let mut s = serializer.serialize_final_struct()?;
        match representation_identifier {
            CDR_BE => push_to_key(dynamic_type, &mut s, &mut Xcdr1BeDeserializer::new(data))?,
            CDR_LE => push_to_key(dynamic_type, &mut s, &mut Xcdr1LeDeserializer::new(data))?,
            PL_CDR_BE => push_to_key_parameter_list(dynamic_type, &mut s, data)?,
            PL_CDR_LE => push_to_key_parameter_list(dynamic_type, &mut s, data)?,
            _ => panic!("representation_identifier not supported"),
        }
    }
    Ok(collection)
}

#[cfg(test)]
mod tests {
    use crate::{
        topic_definition::type_support::TypeSupport,
        xtypes::{serialize::XTypesSerialize, xcdr_serializer::Xcdr1LeSerializer},
    };

    use super::*;
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
    fn key_from_mutable_struct() {
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
        assert_eq!(
            get_serialized_key_from_serialized_foo(&data, &MutableStruct::get_type()).unwrap(),
            vec![
                0, 3, 0, 0, // RTPS header
                1, 0, 0, 2 // key_field1 (u8) | padding (1byte) | key_field2 (u16)
            ]
        );
    }

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
