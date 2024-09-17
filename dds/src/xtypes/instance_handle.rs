use super::serialize::{Write, XTypesSerialize};
use crate::xtypes::{
    deserializer::XTypesDeserializer,
    xcdr_deserializer::Xcdr1LeDeserializer,
    xcdr_serializer::{Xcdr1LeSerializer, Xcdr2BeSerializer},
};

//@extensibility(FINAL)
struct Simple {
    //@key
    key_field: u16,
    field: u64,
}

#[derive(XTypesSerialize)]
//@extensibility(FINAL) @nested
struct Nested {
    x: u8,
    y: u8,
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

trait TypeSupport {
    fn get_type() -> impl DynamicType;
}

impl TypeSupport for Complex {
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
        if total_new_length < self.key.len() {
            self.key[self.length..total_new_length].copy_from_slice(buf);
        }
        self.context.consume(buf);
        self.length += buf.len();
    }
}

fn push_to_key(
    dynamic_type: &dyn DynamicType,
    serializer: &mut Xcdr2BeSerializer<'_, Md5>,
    de: &mut Xcdr1LeDeserializer<'_>,
) {
    for id in 0..dynamic_type.get_member_count() {
        let is_key_field = dynamic_type.get_member_by_index(id).is_key_field();
        match dynamic_type.get_member_by_index(id).field_type().get_kind() {
            TypeKind::LongLong => {
                let v = de.deserialize_int64().unwrap();
                if is_key_field {
                    v.serialize(&mut *serializer).unwrap()
                }
            }
            TypeKind::Ushort => {
                let v = de.deserialize_uint16().unwrap();
                if is_key_field {
                    v.serialize(&mut *serializer).unwrap()
                }
            }
            TypeKind::Ulong => {
                let v = de.deserialize_uint32().unwrap();
                if is_key_field {
                    v.serialize(&mut *serializer).unwrap()
                }
            }
            TypeKind::Byte => {
                let v = de.deserialize_uint8().unwrap();
                if is_key_field {
                    v.serialize(&mut *serializer).unwrap()
                }
            }
            TypeKind::NonBasic(type_object) => {
                push_to_key(&type_object, serializer, de);
            }
        }
    }
}



fn get_key_from_serialized_type<T: TypeSupport>(data: &[u8]) -> [u8; 16] {
    let dynamic_type = T::get_type();

    let mut md5_collection = Md5 {
        key: [0; 16],
        context: md5::Context::new(),
        length: 0,
    };
    let mut serializer = Xcdr2BeSerializer::new(&mut md5_collection);
    let mut de: Xcdr1LeDeserializer<'_> = Xcdr1LeDeserializer::new(data);

    push_to_key(&dynamic_type, &mut serializer, &mut de);
    md5_collection.into_key()
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
    assert_eq!(get_key_from_serialized_type::<Complex>(&data), key_data.as_slice());
}

#[derive(XTypesSerialize)]
//@extensibility(FINAL)
struct Large {
    key_field1: i64,
    key_field2: i64,
    key_field3: i64,
}

impl TypeSupport for Large {
    fn get_type() -> impl DynamicType {
        TypeObejct {
            member_list: vec![
                MemberInfo {
                    type_kind: TypeKind::LongLong,
                    is_key: true,
                },
                MemberInfo {
                    type_kind: TypeKind::LongLong,
                    is_key: true,
                },
                MemberInfo {
                    type_kind: TypeKind::LongLong,
                    is_key: true,
                },
            ],
        }
    }
}

#[test]
fn large_key() {
    let v = Large {
        key_field1: 1,
        key_field2: 2,
        key_field3: 3,
    };
    let data = [
        1, 0, 0, 0, 0, 0, 0, 0, //key_field1 (i64)
        2, 0, 0, 0, 0, 0, 0, 0, //key_field1 (i64)
        3, 0, 0, 0, 0, 0, 0, 0, //key_field1 (i64)
    ];
    let mut collection = Vec::new();
    v.serialize(&mut Xcdr1LeSerializer::new(&mut collection))
        .unwrap();
    assert_eq!(&data, collection.as_slice());
    let key_data = [
        0, 0, 0, 0, 0, 0, 0, 1, 
        0, 0, 0, 0, 0, 0, 0, 2, 
        0, 0, 0, 0, 0, 0, 0, 3, 
    ];
    assert_eq!(
        get_key_from_serialized_type::<Large>(&data),
        md5::compute(key_data).as_slice()
    );
}
