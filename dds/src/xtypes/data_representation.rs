use crate::xtypes::{
    dynamic_type::{DynamicData, ExtensibilityKind, MemberDescriptor, TypeKind},
    serialize::{SerializeCollection, XTypesSerialize, XTypesSerializer},
    serializer::{SerializeAppendableStruct, SerializeFinalStruct, SerializeMutableStruct},
};
use alloc::vec::Vec;

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

struct Array<'a, T>(&'a Vec<T>);
impl<'a, T: XTypesSerialize> XTypesSerialize for Array<'a, T> {
    fn serialize(
        &self,
        serializer: impl crate::xtypes::serialize::XTypesSerializer,
    ) -> Result<(), crate::xtypes::error::XTypesError> {
        let mut collection_serializer = serializer.serialize_array()?;
        for e in self.0 {
            todo!()
            // collection_serializer.serialize_element(e)?;
        }

        Ok(())
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
