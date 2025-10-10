use crate::{
    infrastructure::type_support::TypeSupport,
    xtypes::{
        dynamic_type::{DynamicData, ExtensibilityKind, MemberDescriptor, TypeKind},
        serializer::{
            SerializeAppendableStruct, SerializeFinalStruct, SerializeMutableStruct,
            XTypesSerializer,
        },
    },
};
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq)]
pub enum DataKind {
    UInt8(u8),
    Int8(i8),
    UInt16(u16),
    Int16(i16),
    Int32(i32),
    UInt32(u32),
    Int64(i64),
    UInt64(u64),
    Float32(f32),
    Float64(f64),
    Char8(char),
    Boolean(bool),
    String(String),
    ComplexValue(DynamicData),
    UInt8List(Vec<u8>),
    Int8List(Vec<i8>),
    UInt16List(Vec<u16>),
    Int16List(Vec<i16>),
    Int32List(Vec<i32>),
    UInt32List(Vec<u32>),
    Int64List(Vec<i64>),
    UInt64List(Vec<u64>),
    Float32List(Vec<f32>),
    Float64List(Vec<f64>),
    Char8List(Vec<char>),
    BooleanList(Vec<bool>),
    StringList(Vec<String>),
    ComplexValueList(Vec<DynamicData>),
    UInt8Array(Vec<u8>),
    Int8Array(Vec<i8>),
    UInt16Array(Vec<u16>),
    Int16Array(Vec<i16>),
    Int32Array(Vec<i32>),
    UInt32Array(Vec<u32>),
    Int64Array(Vec<i64>),
    UInt64Array(Vec<u64>),
    Float32Array(Vec<f32>),
    Float64Array(Vec<f64>),
    Char8Array(Vec<char>),
    BooleanArray(Vec<bool>),
    StringArray(Vec<String>),
    ComplexValueArray(Vec<DynamicData>),
}

impl DataKind {
    pub fn serialize<T>(&self, serializer: &mut T) -> Result<(), super::error::XTypesError>
    where
        for<'a> &'a mut T: XTypesSerializer,
    {
        match self {
            DataKind::UInt8(v) => serializer.serialize_uint8(*v),
            DataKind::Int8(v) => serializer.serialize_int8(*v),
            DataKind::UInt16(v) => serializer.serialize_uint16(*v),
            DataKind::Int16(v) => serializer.serialize_int16(*v),
            DataKind::Int32(v) => serializer.serialize_int32(*v),
            DataKind::UInt32(v) => serializer.serialize_uint32(*v),
            DataKind::Int64(v) => serializer.serialize_int64(*v),
            DataKind::UInt64(v) => serializer.serialize_uint64(*v),
            DataKind::Float32(v) => serializer.serialize_float32(*v),
            DataKind::Float64(v) => serializer.serialize_float64(*v),
            DataKind::Char8(v) => serializer.serialize_char8(*v),
            DataKind::Boolean(v) => serializer.serialize_boolean(*v),
            DataKind::String(v) => serializer.serialize_string(v),
            DataKind::ComplexValue(dynamic_data) => dynamic_data.serialize_nested(serializer),
            DataKind::UInt8List(items) => serializer.serialize_uint8_list(items),
            DataKind::Int8List(items) => serializer.serialize_int8_list(items),
            DataKind::UInt16List(items) => serializer.serialize_uint16_list(items),
            DataKind::Int16List(items) => serializer.serialize_int16_list(items),
            DataKind::Int32List(items) => serializer.serialize_int32_list(items),
            DataKind::UInt32List(items) => serializer.serialize_uint32_list(items),
            DataKind::Int64List(items) => serializer.serialize_int64_list(items),
            DataKind::UInt64List(items) => serializer.serialize_uint64_list(items),
            DataKind::Float32List(items) => serializer.serialize_float32_list(items),
            DataKind::Float64List(items) => serializer.serialize_float64_list(items),
            DataKind::Char8List(items) => serializer.serialize_char8_list(items),
            DataKind::BooleanList(items) => serializer.serialize_boolean_list(items),
            DataKind::StringList(items) => serializer.serialize_string_list(items),
            DataKind::ComplexValueList(items) => serializer.serialize_sequence(items),
            DataKind::UInt8Array(items) => serializer.serialize_uint8_array(items),
            DataKind::Int8Array(items) => serializer.serialize_int8_array(items),
            DataKind::UInt16Array(items) => serializer.serialize_uint16_array(items),
            DataKind::Int16Array(items) => serializer.serialize_int16_array(items),
            DataKind::Int32Array(items) => serializer.serialize_int32_array(items),
            DataKind::UInt32Array(items) => serializer.serialize_uint32_array(items),
            DataKind::Int64Array(items) => serializer.serialize_int64_array(items),
            DataKind::UInt64Array(items) => serializer.serialize_uint64_array(items),
            DataKind::Float32Array(items) => serializer.serialize_float32_array(items),
            DataKind::Float64Array(items) => serializer.serialize_float64_array(items),
            DataKind::Char8Array(items) => serializer.serialize_char8_array(items),
            DataKind::BooleanArray(items) => serializer.serialize_boolean_array(items),
            DataKind::StringArray(items) => serializer.serialize_string_array(items),
            DataKind::ComplexValueArray(items) => serializer.serialize_array(items),
        }
    }
}

impl From<u8> for DataKind {
    fn from(value: u8) -> Self {
        Self::UInt8(value)
    }
}

impl From<i8> for DataKind {
    fn from(value: i8) -> Self {
        Self::Int8(value)
    }
}

impl From<u16> for DataKind {
    fn from(value: u16) -> Self {
        Self::UInt16(value)
    }
}

impl From<i16> for DataKind {
    fn from(value: i16) -> Self {
        Self::Int16(value)
    }
}

impl From<u32> for DataKind {
    fn from(value: u32) -> Self {
        Self::UInt32(value)
    }
}

impl From<i32> for DataKind {
    fn from(value: i32) -> Self {
        Self::Int32(value)
    }
}

impl From<u64> for DataKind {
    fn from(value: u64) -> Self {
        Self::UInt64(value)
    }
}

impl From<i64> for DataKind {
    fn from(value: i64) -> Self {
        Self::Int64(value)
    }
}

impl From<f32> for DataKind {
    fn from(value: f32) -> Self {
        Self::Float32(value)
    }
}

impl From<f64> for DataKind {
    fn from(value: f64) -> Self {
        Self::Float64(value)
    }
}

impl From<char> for DataKind {
    fn from(value: char) -> Self {
        Self::Char8(value)
    }
}

impl From<bool> for DataKind {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<String> for DataKind {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

// impl<T: TypeSupport> From<T> for DataKind {
//     fn from(value: T) -> Self {
//         Self::ComplexValue(value.create_dynamic_sample())
//     }
// }

impl<const N: usize> From<[u8; N]> for DataKind {
    fn from(value: [u8; N]) -> Self {
        Self::UInt8Array(value.to_vec())
    }
}

impl<const N: usize> From<[i8; N]> for DataKind {
    fn from(value: [i8; N]) -> Self {
        Self::Int8List(value.to_vec())
    }
}

impl<const N: usize> From<[u16; N]> for DataKind {
    fn from(value: [u16; N]) -> Self {
        Self::UInt16List(value.to_vec())
    }
}

impl<const N: usize> From<[i16; N]> for DataKind {
    fn from(value: [i16; N]) -> Self {
        Self::Int16List(value.to_vec())
    }
}

impl<const N: usize> From<[u32; N]> for DataKind {
    fn from(value: [u32; N]) -> Self {
        Self::UInt32List(value.to_vec())
    }
}

impl<const N: usize> From<[i32; N]> for DataKind {
    fn from(value: [i32; N]) -> Self {
        Self::Int32List(value.to_vec())
    }
}

impl<const N: usize> From<[u64; N]> for DataKind {
    fn from(value: [u64; N]) -> Self {
        Self::UInt64List(value.to_vec())
    }
}

impl<const N: usize> From<[i64; N]> for DataKind {
    fn from(value: [i64; N]) -> Self {
        Self::Int64List(value.to_vec())
    }
}

impl<const N: usize> From<[f32; N]> for DataKind {
    fn from(value: [f32; N]) -> Self {
        Self::Float32List(value.to_vec())
    }
}

impl<const N: usize> From<[f64; N]> for DataKind {
    fn from(value: [f64; N]) -> Self {
        Self::Float64List(value.to_vec())
    }
}

impl<const N: usize> From<[char; N]> for DataKind {
    fn from(value: [char; N]) -> Self {
        Self::Char8List(value.to_vec())
    }
}

impl<const N: usize> From<[bool; N]> for DataKind {
    fn from(value: [bool; N]) -> Self {
        Self::BooleanList(value.to_vec())
    }
}

impl<const N: usize> From<[String; N]> for DataKind {
    fn from(value: [String; N]) -> Self {
        Self::StringList(value.to_vec())
    }
}

impl<const N: usize, T: TypeSupport> From<[T; N]> for DataKind {
    fn from(value: [T; N]) -> Self {
        Self::ComplexValueList(
            value
                .into_iter()
                .map(TypeSupport::create_dynamic_sample)
                .collect(),
        )
    }
}

impl From<Vec<u8>> for DataKind {
    fn from(value: Vec<u8>) -> Self {
        Self::UInt8Array(value)
    }
}

impl From<Vec<i8>> for DataKind {
    fn from(value: Vec<i8>) -> Self {
        Self::Int8List(value)
    }
}

impl From<Vec<u16>> for DataKind {
    fn from(value: Vec<u16>) -> Self {
        Self::UInt16List(value)
    }
}

impl From<Vec<i16>> for DataKind {
    fn from(value: Vec<i16>) -> Self {
        Self::Int16List(value)
    }
}

impl From<Vec<u32>> for DataKind {
    fn from(value: Vec<u32>) -> Self {
        Self::UInt32List(value)
    }
}

impl From<Vec<i32>> for DataKind {
    fn from(value: Vec<i32>) -> Self {
        Self::Int32List(value)
    }
}

impl From<Vec<u64>> for DataKind {
    fn from(value: Vec<u64>) -> Self {
        Self::UInt64List(value)
    }
}

impl From<Vec<i64>> for DataKind {
    fn from(value: Vec<i64>) -> Self {
        Self::Int64List(value)
    }
}

impl From<Vec<f32>> for DataKind {
    fn from(value: Vec<f32>) -> Self {
        Self::Float32List(value)
    }
}

impl From<Vec<f64>> for DataKind {
    fn from(value: Vec<f64>) -> Self {
        Self::Float64List(value)
    }
}

impl From<Vec<char>> for DataKind {
    fn from(value: Vec<char>) -> Self {
        Self::Char8List(value)
    }
}

impl From<Vec<bool>> for DataKind {
    fn from(value: Vec<bool>) -> Self {
        Self::BooleanList(value)
    }
}

impl From<Vec<String>> for DataKind {
    fn from(value: Vec<String>) -> Self {
        Self::StringList(value)
    }
}

impl From<&[u8]> for DataKind {
    fn from(value: &[u8]) -> Self {
        Self::UInt8Array(value.to_vec())
    }
}

impl<T: TypeSupport> From<Vec<T>> for DataKind {
    fn from(value: Vec<T>) -> Self {
        Self::ComplexValueList(
            value
                .into_iter()
                .map(TypeSupport::create_dynamic_sample)
                .collect(),
        )
    }
}

impl<T: TypeSupport> From<T> for DataKind {
    fn from(value: T) -> Self {
        DataKind::ComplexValue(value.create_dynamic_sample())
    }
}

impl MemberDescriptor {
    fn get_element_type(&self) -> TypeKind {
        self.r#type
            .get_descriptor()
            .element_type
            .as_ref()
            .expect("array type must have element type")
            .get_kind()
    }
}

impl DynamicData {
    pub fn serialize<T>(&self, serializer: &mut T) -> Result<(), super::error::XTypesError>
    where
        for<'a> &'a mut T: XTypesSerializer,
    {
        self.serialize_nested(serializer)
    }

    pub fn serialize_nested<T>(&self, serializer: &mut T) -> Result<(), super::error::XTypesError>
    where
        for<'a> &'a mut T: XTypesSerializer,
    {
        match self.type_ref().get_kind() {
            TypeKind::ENUM => {
                self.get_value(0)?.serialize(&mut *serializer)?;
            }
            TypeKind::STRUCTURE => match self.type_ref().get_descriptor().extensibility_kind {
                ExtensibilityKind::Final => {
                    let mut final_serializer = serializer.serialize_final_struct()?;
                    for field_index in 0..self.get_item_count() {
                        let member_id = self.get_member_id_at_index(field_index)?;
                        let member_descriptor = self.get_descriptor(member_id)?;
                        final_serializer
                            .serialize_field(self.get_value(member_id)?, &member_descriptor.name)?;
                    }
                }
                ExtensibilityKind::Appendable => {
                    let mut appendable_serializer = serializer.serialize_appendable_struct()?;
                    for field_index in 0..self.get_item_count() {
                        let member_id = self.get_member_id_at_index(field_index)?;
                        let member_descriptor = self.get_descriptor(member_id)?;
                        appendable_serializer
                            .serialize_field(self.get_value(member_id)?, &member_descriptor.name)?;
                    }
                }
                ExtensibilityKind::Mutable => {
                    let mut mutable_serializer = serializer.serialize_mutable_struct()?;
                    for field_index in 0..self.get_item_count() {
                        let member_id = self.get_member_id_at_index(field_index)?;
                        let member_descriptor = self.get_descriptor(member_id)?;
                        let member_type_kind = member_descriptor.r#type.get_kind();
                        let member_name = &member_descriptor.name;
                        let pid = member_id;

                        if member_descriptor.is_optional {
                            if let Some(default_value) = &member_descriptor.default_value {
                                if self.get_value(member_id)? == default_value {
                                    continue;
                                }
                            }
                        }

                        if member_type_kind == TypeKind::SEQUENCE {
                            let values = self.get_complex_values(member_id)?;
                            mutable_serializer.serialize_collection(values, pid, member_name)?;
                        } else {
                            let value = self.get_value(member_id)?;
                            mutable_serializer.serialize_field(value, pid, member_name)?;
                        }
                    }
                    mutable_serializer.end()?;
                }
            },
            kind => todo!("Noy yet implemented for {kind:?}"),
        }
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::{
//         infrastructure::type_support::TypeSupport, xtypes::xcdr_serializer::Xcdr1LeSerializer,
//     };

//     #[test]
//     fn serialize_final_struct_cdr1() {
//         #[derive(TypeSupport)]
//         struct FinalStruct {
//             id: u8,
//             a: i32,
//             b: u32,
//             c: i16,
//             d: u16,
//         }

//         let data = FinalStruct {
//             id: 1,
//             a: 100,
//             b: 200,
//             c: 10,
//             d: 20,
//         };
//         let dynamic_sample = data.create_dynamic_sample();
//         let mut buffer = vec![];
//         let mut serializer = Xcdr1LeSerializer::new(&mut buffer);
//         dynamic_sample.serialize(&mut serializer).unwrap();

//         assert_eq!(
//             &buffer,
//             &[
//                 1, 0, 0, 0, //id
//                 100, 0, 0, 0, //a
//                 200, 0, 0, 0, //b
//                 10, 0, //c
//                 20, 0, //d
//             ]
//         )
//     }

//     #[test]
//     fn serialize_final_struct_with_array_cdr1() {
//         #[derive(TypeSupport)]
//         struct FinalStruct {
//             array_i16: [i16; 2],
//         }
//         let dynamic_sample = FinalStruct { array_i16: [1, 2] }.create_dynamic_sample();
//         let mut buffer = vec![];
//         dynamic_sample
//             .serialize(&mut Xcdr1LeSerializer::new(&mut buffer))
//             .unwrap();
//         assert_eq!(
//             &buffer,
//             &[
//                 1, 0, 2, 0, //array_i16
//             ]
//         )
//     }

//     #[test]
//     fn serialize_final_struct_with_sequence_cdr1() {
//         #[derive(TypeSupport)]
//         struct FinalStruct {
//             sequence_i16: Vec<i16>,
//         }
//         let dynamic_sample = FinalStruct {
//             sequence_i16: vec![1, 2],
//         }
//         .create_dynamic_sample();
//         let mut buffer = vec![];
//         dynamic_sample
//             .serialize(&mut Xcdr1LeSerializer::new(&mut buffer))
//             .unwrap();
//         assert_eq!(
//             &buffer,
//             &[
//                 2, 0, 0, 0, // length sequence_i16
//                 1, 0, 2, 0, //sequence_i16
//             ]
//         )
//     }

//     #[test]
//     fn serialize_nested_final_struct_cdr1() {
//         #[derive(TypeSupport)]
//         struct Nested {
//             n: u16,
//             m: i16,
//         }
//         // #[derive(TypeSupport)]
//         struct FinalStruct {
//             a: i32,
//             nested: Nested,
//         }

//         impl dust_dds::infrastructure::type_support::TypeSupport for FinalStruct {
//             fn get_type() -> dust_dds::xtypes::dynamic_type::DynamicType {
//                 extern crate alloc;
//                 let mut builder =
//                     dust_dds::xtypes::dynamic_type::DynamicTypeBuilderFactory::create_type(
//                         dust_dds::xtypes::dynamic_type::TypeDescriptor {
//                             kind: dust_dds::xtypes::dynamic_type::TypeKind::STRUCTURE,
//                             name: alloc::string::String::from("FinalStruct"),
//                             base_type: None,
//                             discriminator_type: None,
//                             bound: alloc::vec::Vec::new(),
//                             element_type: None,
//                             key_element_type: None,
//                             extensibility_kind:
//                                 dust_dds::xtypes::dynamic_type::ExtensibilityKind::Final,
//                             is_nested: false,
//                         },
//                     );
//                 builder
//                     .add_member(dust_dds::xtypes::dynamic_type::MemberDescriptor {
//                         name: alloc::string::String::from("a"),
//                         id: 0,
//                         r#type: <i32 as TypeSupport>::get_type(),
//                         default_value: None,
//                         index: 0u32,
//                         try_construct_kind:
//                             dust_dds::xtypes::dynamic_type::TryConstructKind::UseDefault,
//                         label: alloc::vec::Vec::new(),
//                         is_key: false,
//                         is_optional: false,
//                         is_must_understand: true,
//                         is_shared: false,
//                         is_default_label: false,
//                     })
//                     .unwrap();
//                 builder
//                     .add_member(dust_dds::xtypes::dynamic_type::MemberDescriptor {
//                         name: alloc::string::String::from("nested"),
//                         id: 1,
//                         r#type: <Nested as TypeSupport>::get_type(),
//                         default_value: None,
//                         index: 1u32,
//                         try_construct_kind:
//                             dust_dds::xtypes::dynamic_type::TryConstructKind::UseDefault,
//                         label: alloc::vec::Vec::new(),
//                         is_key: false,
//                         is_optional: false,
//                         is_must_understand: true,
//                         is_shared: false,
//                         is_default_label: false,
//                     })
//                     .unwrap();
//                 builder.build()
//             }
//             fn create_dynamic_sample(self) -> dust_dds::xtypes::dynamic_type::DynamicData {
//                 let mut data = dust_dds::xtypes::dynamic_type::DynamicDataFactory::create_data(
//                     Self::get_type(),
//                 );
//                 data.set_value(0, self.a.into()).unwrap();
//                 data.set_value(1, self.nested.into()).unwrap();
//                 data
//             }
//         }
//         let dynamic_sample = FinalStruct {
//             a: 1,
//             nested: Nested { n: 2, m: 3 },
//         }
//         .create_dynamic_sample();
//         let mut buffer = vec![];
//         dynamic_sample
//             .serialize(&mut Xcdr1LeSerializer::new(&mut buffer))
//             .unwrap();
//         assert_eq!(
//             &buffer,
//             &[
//                 1, 0, 0, 0, // a
//                 2, 0, 3, 0, // nested
//             ]
//         )
//     }

//     #[test]
//     fn serialize_nested_appendable_struct_cdr1() {
//         #[derive(TypeSupport)]
//         #[dust_dds(extensibility = "appendable")]
//         struct Nested {
//             n: u16,
//             m: i16,
//         }
//         #[derive(TypeSupport)]
//         #[dust_dds(extensibility = "appendable")]
//         struct AppendableStruct {
//             a: i32,
//             nested: Nested,
//         }
//         let dynamic_sample = AppendableStruct {
//             a: 1,
//             nested: Nested { n: 2, m: 3 },
//         }
//         .create_dynamic_sample();
//         let mut buffer = vec![];
//         dynamic_sample
//             .serialize(&mut Xcdr1LeSerializer::new(&mut buffer))
//             .unwrap();
//         assert_eq!(
//             &buffer,
//             &[
//                 1, 0, 0, 0, // a
//                 2, 0, 3, 0, // nested
//             ]
//         )
//     }

//     #[test]
//     fn serialize_mutable_struct_cdr1() {
//         #[derive(TypeSupport)]
//         #[dust_dds(extensibility = "mutable")]
//         struct MutableStruct {
//             #[dust_dds(id = 0x07)]
//             byte: u8,
//             #[dust_dds(id = 0x08)]
//             long: i32,
//         }
//         let dynamic_sample = MutableStruct { byte: 1, long: 2 }.create_dynamic_sample();
//         let mut buffer = vec![];
//         dynamic_sample
//             .serialize(&mut Xcdr1LeSerializer::new(&mut buffer))
//             .unwrap();
//         assert_eq!(
//             &buffer,
//             &[
//                 0x07, 0x00, 1, 0, // PID | length
//                 1, 0, 0, 0, // byte | padding
//                 0x08, 0x00, 4, 0, // PID | length
//                 2, 0, 0, 0, // long
//                 1, 0, 0, 0, // Sentinel
//             ]
//         )
//     }

//     #[test]
//     fn serialize_nested_mutable_struct_cdr1() {
//         #[derive(TypeSupport)]
//         #[dust_dds(extensibility = "mutable")]
//         struct NestedStruct {
//             #[dust_dds(id = 0x09)]
//             nested_byte: u8,
//             #[dust_dds(id = 0x0A)]
//             nested_long: i32,
//         }
//         #[derive(TypeSupport)]
//         #[dust_dds(extensibility = "mutable")]
//         struct MutableStruct {
//             #[dust_dds(id = 0x06)]
//             byte: u8,
//             #[dust_dds(id = 0x07)]
//             nested: NestedStruct,
//             #[dust_dds(id = 0x08)]
//             long: i32,
//         }

//         let dynamic_sample = MutableStruct {
//             byte: 1,
//             nested: NestedStruct {
//                 nested_byte: 11,
//                 nested_long: 22,
//             },
//             long: 2,
//         }
//         .create_dynamic_sample();
//         let mut buffer = vec![];
//         dynamic_sample
//             .serialize(&mut Xcdr1LeSerializer::new(&mut buffer))
//             .unwrap();
//         assert_eq!(
//             &buffer,
//             &[
//                 0x06, 0x00, 1, 0, // PID | length
//                 1, 0, 0, 0, // byte | padding
//                 0x07, 0x00, 20, 0, // Nested: PID | length
//                 0x09, 0x00, 1, 0, // Nested: PID | length
//                 11, 0, 0, 0, // Nested: nested_byte | padding
//                 0x0A, 0x00, 4, 0, // Nested: PID | length
//                 22, 0, 0, 0, // Nested: nested_long | padding
//                 1, 0, 0, 0, // Nested: Sentinel
//                 0x08, 0x00, 4, 0, // PID | length
//                 2, 0, 0, 0, // long
//                 1, 0, 0, 0, // Sentinel
//             ]
//         )
//     }
// }
