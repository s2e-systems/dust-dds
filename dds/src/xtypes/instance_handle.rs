use std::io::{BufRead, Write};

use crate::xtypes::{serialize::XTypesSerializer, xcdr_serializer::Xcdr1LeSerializer};

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

struct TypeObejct {
    member_list: Vec<TypeKind>,
}

enum TypeKind {
    LongLong,
    Ushort,
    Ulong,
    Byte,
    NonBasic(TypeObejct),
}
impl Complex {
    fn is_key_field(id: usize) -> bool {
        match id {
            0 => false,
            1 => true,
            2 => false,
            3 => true,
            _ => panic!(),
        }
    }
    fn field_type(id: usize) -> TypeKind {
        match id {
            0 => TypeKind::LongLong,
            1 => TypeKind::Ushort,
            2 => TypeKind::Ulong,
            3 => TypeKind::NonBasic(TypeObejct {
                member_list: vec![TypeKind::Byte, TypeKind::Byte],
            }),
            _ => panic!(),
        }
    }
    fn field_ids() -> Vec<usize> {
        vec![0, 1, 2, 3]
    }
}

trait GetKeyFromSerializedKey {
    fn get_key_from_serialized_type(&self, data: &[u8]) -> [u8; 16];
}

struct Md5(md5::Context);
impl<'a> Extend<&'a u8> for Md5 {
    fn extend<T: IntoIterator<Item = &'a u8>>(&mut self, iter: T) {
        for i in iter {
            self.0.consume(&[*i]);
        }
    }
}

impl super::serializer::SerializeFinalStruct for VirtualCdr1Serializer {
    fn serialize_field<T: XTypesSerialize>(
        &mut self,
        value: &T,
        name: &str,
    ) -> Result<(), super::error::XTypesError> {
        todo!()
    }

    fn serialize_optional_field<T: XTypesSerialize>(
        &mut self,
        value: &Option<T>,
        name: &str,
    ) -> Result<(), super::error::XTypesError> {
        todo!()
    }
}
fn round_up_to_multiples(position: usize, alignment: usize) -> usize {
    (position + alignment - 1) / alignment * alignment
}
struct VirtualCdr1Serializer {
    //buffer: Md5,
    position: usize,
}
impl VirtualCdr1Serializer {
    fn serialize_final_struct(
        self,
    ) -> Result<impl super::serializer::SerializeFinalStruct, super::error::XTypesError> {
        Ok(self)
    }
    fn serialize_int64(self) -> Result<(), super::error::XTypesError> {
        todo!()
    }
    fn serialize_uint8(self, v: u8) -> Result<(), super::error::XTypesError> {
        todo!()
    }
    fn serialize_uint16(self, v: u16) -> Result<(), super::error::XTypesError> {
        todo!()
    }
    fn consume(&mut self, alignment: usize) -> usize {
        let padded = round_up_to_multiples(self.position, alignment) - self.position + alignment;
        self.position += padded;
        padded
    }
}

impl Complex {
    fn get_key_from_serialized_type(&self, mut data: &[u8]) -> [u8; 16] {
        let total = data.len();
        let mut buffer = md5::Context::new();
        let mut collection = Vec::new();
        // let mut serializer = Xcdr1LeSerializer::new(&mut collection);
        let mut serializer = VirtualCdr1Serializer {
            //buffer: collection,
            position: 0,
        };
        for id in Complex::field_ids() {
            let field_len = match Complex::field_type(id) {
                TypeKind::LongLong => 8,
                TypeKind::Ulong => 4,
                TypeKind::Ushort => 2,
                TypeKind::Byte => 1,
                TypeKind::NonBasic(type_object) => {
                    let mut field_len = 0;
                    for member in &type_object.member_list {
                        match member {
                            TypeKind::LongLong => field_len += 8,
                            TypeKind::Ulong => field_len += 4,
                            TypeKind::Ushort => field_len += 2,
                            TypeKind::Byte => field_len += 1,
                            TypeKind::NonBasic(_) => todo!(),
                        }
                    }
                    field_len
                }
            };
            let padded_field_length = serializer.consume(field_len);
            if Complex::is_key_field(id) {
                buffer.consume(&data[..padded_field_length]);
                collection.append(&mut data[..padded_field_length].to_vec());
            }
            data.consume(padded_field_length);
        }
        const ZEROS: [u8; 16] = [0; 16];
        buffer.consume(&ZEROS[collection.len()..]);
        // buffer.compute().into()        
        collection.append(&mut ZEROS[collection.len()..].to_vec());
        collection.as_slice().try_into().unwrap()
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
        2, 0, 0, 0, 0, 0, 0, 0, //field1
        3, 0, 0, 0, //key_field1 | padding (2B)
        4, 0, 0, 0, //field2
        5, 6, //key_field2
    ];
    let mut collection = Vec::new();
    v.serialize(&mut Xcdr1LeSerializer::new(&mut collection))
        .unwrap();
    assert_eq!(&data, collection.as_slice());
    let key_data = [3, 0, 5, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    // assert_eq!(
    //     v.get_key_from_serialized_type(&data),
    //     md5::compute(key_data).as_slice()
    // );
    assert_eq!(
        v.get_key_from_serialized_type(&data),
        key_data.as_slice()
    );
}
