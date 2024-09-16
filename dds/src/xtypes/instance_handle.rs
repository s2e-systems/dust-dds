use std::io::{BufRead, Write};

use crate::{
    topic_definition::type_support,
    xtypes::{
        deserializer::XTypesDeserializer,
        dynamic_type,
        serialize::XTypesSerializer,
        xcdr_deserializer::Xcdr1LeDeserializer,
        xcdr_serializer::{Xcdr1LeSerializer, Xcdr2BeSerializer},
    },
};

use super::{serialize::XTypesSerialize, xcdr_serializer::Xcdr2LeSerializer};

//@extensibility(FINAL)
struct Simple {
    //@key
    key_field: u16,
    field: u64,
}

impl Simple {
    fn get_key_from_serialized_type(&self, data: &[u8]) -> [u8; 16] {
        let a = Xcdr1LeSerializer::bytes_len(&self.field).unwrap();
        [0; 16]
    }
}

#[derive(XTypesSerialize)]
//@extensibility(FINAL) @nested
struct Nested {
    x: u8,
    y: u8,
}

impl Nested {
    fn field_type(id: usize) -> TypeKind {
        match id {
            0 => TypeKind::Byte,
            1 => TypeKind::Byte,
            _ => panic!(),
        }
    }
    fn field_ids() -> Vec<usize> {
        vec![0, 1]
    }
}

#[derive(XTypesSerialize)]
//@extensibility(FINAL)
struct Complex {
    field1: i64,
    //@key
    key_field1: u16,
    field2: u32,
    //@key
    key_field2: Nested,
}

trait BytesLen {
    fn bytes_len_cdr1(&self) -> usize;
    fn bytes_len_cdr2(&self) -> usize;
}
struct ByteCounter(usize);

impl<'a> Extend<&'a u8> for ByteCounter {
    fn extend<T: IntoIterator<Item = &'a u8>>(&mut self, iter: T) {
        self.0 += iter.into_iter().count();
    }
}

impl<T: XTypesSerialize> BytesLen for T {
    fn bytes_len_cdr1(&self) -> usize {
        let mut byte_counter = ByteCounter(0);
        let mut serializer = Xcdr1LeSerializer::new(&mut byte_counter);
        XTypesSerialize::serialize(self, &mut serializer).unwrap();
        byte_counter.0
    }
    fn bytes_len_cdr2(&self) -> usize {
        let mut byte_counter = ByteCounter(0);
        let mut serializer = Xcdr2LeSerializer::new(&mut byte_counter);
        XTypesSerialize::serialize(self, &mut serializer).unwrap();
        byte_counter.0
    }
}

#[derive(Clone)]
struct TypeObejct {
    member_list: Vec<MemberInfo>,
}

#[derive(Clone)]
struct MemberInfo {
    type_kind: TypeKind,
    is_key: bool,
}
#[derive(Clone)]
enum TypeKind {
    LongLong,
    Ushort,
    Ulong,
    Byte,
    NonBasic(TypeObejct),
}

impl DynamicType for TypeKind {
    fn get_member_by_index(&self, id: usize) -> &dyn DynamicTypeMember {
        match self {
            TypeKind::NonBasic(type_object) => type_object.get_member_by_index(id),
            _ => panic!(),
        }
    }

    fn get_kind(&self) -> TypeKind {
        self.clone()
    }

    fn get_member_count(&self) -> usize {
        match self {
            TypeKind::NonBasic(type_object) => type_object.get_member_count(),
            _ => 0,
        }
    }
}

impl DynamicTypeMember for MemberInfo {
    fn is_key_field(&self) -> bool {
        self.is_key
    }

    fn field_type(&self) -> &dyn DynamicType {
        &self.type_kind
    }
}

trait DynamicTypeMember {
    fn is_key_field(&self) -> bool;
    fn field_type(&self) -> &dyn DynamicType;
}
trait DynamicType {
    fn get_member_by_index(&self, id: usize) -> &dyn DynamicTypeMember;
    fn get_kind(&self) -> TypeKind;
    fn get_member_count(&self) -> usize;
}

impl DynamicType for TypeObejct {
    fn get_member_by_index(&self, id: usize) -> &dyn DynamicTypeMember {
        &self.member_list[id]
    }

    fn get_kind(&self) -> TypeKind {
        TypeKind::NonBasic(self.clone())
    }

    fn get_member_count(&self) -> usize {
        self.member_list.len()
    }
}

impl Complex {
    fn get_type() -> impl DynamicType {
        TypeObejct {
            member_list: vec![
                MemberInfo {
                    type_kind: TypeKind::LongLong,
                    is_key: false,
                },
                MemberInfo {
                    type_kind: TypeKind::Ushort,
                    is_key: true,
                },
                MemberInfo {
                    type_kind: TypeKind::Ulong,
                    is_key: false,
                },
                MemberInfo {
                    type_kind: TypeKind::NonBasic(TypeObejct {
                        member_list: vec![
                            MemberInfo {
                                type_kind: TypeKind::Byte,
                                is_key: true,
                            },
                            MemberInfo {
                                type_kind: TypeKind::Byte,
                                is_key: true,
                            },
                        ],
                    }),
                    is_key: true,
                },
            ],
        }
    }
}

struct Md5 {
    key: [u8; 16],
    context: md5::Context,
    length: usize,
}
impl<'a> Extend<&'a u8> for Md5 {
    fn extend<T: IntoIterator<Item = &'a u8>>(&mut self, iter: T) {
        for i in iter {
            if self.length < self.key.len() {
                self.key[self.length] = *i;
            }
            self.context.consume(&[*i]);
            self.length += 1;
        }
    }
}
fn push_to_key(
    type_object: &TypeObejct,
    serializer: &mut Xcdr2BeSerializer<'_, Md5>,
    de: &mut Xcdr1LeDeserializer<'_>,
) {
    for id in 0..type_object.get_member_count() {
        match type_object.get_member_by_index(id).field_type().get_kind() {
            TypeKind::LongLong => de
                .deserialize_int64()
                .unwrap()
                .serialize(&mut *serializer)
                .unwrap(),
            TypeKind::Ushort => de
                .deserialize_uint16()
                .unwrap()
                .serialize(&mut *serializer)
                .unwrap(),
            TypeKind::Ulong => de
                .deserialize_uint32()
                .unwrap()
                .serialize(&mut *serializer)
                .unwrap(),
            TypeKind::Byte => de
                .deserialize_uint8()
                .unwrap()
                .serialize(&mut *serializer)
                .unwrap(),
            TypeKind::NonBasic(type_object) => {
                push_to_key(&type_object, serializer, de);
            }
        }
    }
}

impl Complex {
    fn get_key_from_serialized_type(&self, mut data: &[u8]) -> [u8; 16] {
        let dynamic_type = Complex::get_type();

        let mut md5_collection = Md5 {
            key: [0; 16],
            context: md5::Context::new(),
            length: 0,
        };
        let mut serializer = Xcdr2BeSerializer::new(&mut md5_collection);
        let mut de: Xcdr1LeDeserializer<'_> = Xcdr1LeDeserializer::new(data);

        for id in 0..dynamic_type.get_member_count() {
            let is_key_field = dynamic_type.get_member_by_index(id).is_key_field();
            match dynamic_type.get_member_by_index(id).field_type().get_kind() {
                TypeKind::LongLong => {
                    let v = de.deserialize_int64().unwrap();
                    if is_key_field {
                        v.serialize(&mut serializer).unwrap()
                    }
                }
                TypeKind::Ushort => {
                    let v = de.deserialize_uint16().unwrap();
                    if is_key_field {
                        v.serialize(&mut serializer).unwrap()
                    }
                },
                TypeKind::Ulong => {
                    let v = de.deserialize_uint32().unwrap();
                    if is_key_field {
                        v.serialize(&mut serializer).unwrap()
                    }
                },
                TypeKind::Byte => {
                    let v = de.deserialize_uint8().unwrap();
                    if is_key_field {
                        v.serialize(&mut serializer).unwrap()
                    }
                },
                TypeKind::NonBasic(type_object) => {
                    push_to_key(&type_object, &mut serializer, &mut de);
                }
            }
        }

        const ZEROS: [u8; 16] = [0; 16];
        if md5_collection.length < ZEROS.len() {
            md5_collection
                .context
                .consume(&ZEROS[md5_collection.length..]);
        }
        if md5_collection.length <= 16 {
            md5_collection.key
        } else {
            md5_collection.context.compute().into()
        }
    }
}

#[test]
fn key() {
    let v = Complex {
        field1: 2,
        key_field1: 3,
        field2: 4,
        key_field2: Nested { x: 5, y: 6 },
    };
    let data = [
        2, 0, 0, 0, 0, 0, 0, 0, //field1 (i64)
        3, 0, 0, 0, //key_field1 (u16) | padding (2B)
        4, 0, 0, 0, //field2 (u32)
        5, 6, //key_field2 (u8, u8)
    ];
    let mut collection = Vec::new();
    v.serialize(&mut Xcdr1LeSerializer::new(&mut collection))
        .unwrap();
    assert_eq!(&data, collection.as_slice());
    let key_data = [0, 3, 5, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    // assert_eq!(
    //     v.get_key_from_serialized_type(&data),
    //     md5::compute(key_data).as_slice()
    // );
    assert_eq!(v.get_key_from_serialized_type(&data), key_data.as_slice());
}
