use crate::xtypes::{
    dynamic_type::{
        DynamicData, ExtensibilityKind, TK_INT16, TK_INT32, TK_INT64, TK_STRUCTURE, TK_UINT16,
        TK_UINT32, TK_UINT8,
    },
    serialize::XTypesSerialize,
    serializer::{SerializeFinalStruct, SerializeMutableStruct},
};

impl XTypesSerialize for DynamicData {
    fn serialize(
        &self,
        serializer: impl super::serialize::XTypesSerializer,
    ) -> Result<(), super::error::XTypesError> {
        match self.type_ref().get_kind() {
            TK_STRUCTURE => match self.type_ref().get_descriptor().extensibility_kind {
                ExtensibilityKind::Final => {
                    let mut final_serializer = serializer.serialize_final_struct()?;
                    for field_index in 0..self.get_item_count() {
                        let member_id = self.get_member_id_at_index(field_index)?;
                        let member_descriptor = self.get_descriptor(member_id)?;
                        let member_type_kind = member_descriptor.r#type.get_kind();
                        let member_name = &member_descriptor.name;
                        match member_type_kind {
                            TK_UINT8 => {
                                let value = self.get_uint8_value(member_id)?;
                                final_serializer.serialize_field(value, member_name)?;
                            }
                            TK_UINT16 => {
                                let value = self.get_uint16_value(member_id)?;
                                final_serializer.serialize_field(value, member_name)?;
                            }
                            TK_UINT32 => {
                                let value = self.get_uint32_value(member_id)?;
                                final_serializer.serialize_field(value, member_name)?;
                            }
                            TK_INT16 => {
                                let value = self.get_int16_value(member_id)?;
                                final_serializer.serialize_field(value, member_name)?;
                            }
                            TK_INT32 => {
                                let value = self.get_int32_value(member_id)?;
                                final_serializer.serialize_field(value, member_name)?;
                            }
                            TK_INT64 => {
                                let value = self.get_int64_value(member_id)?;
                                final_serializer.serialize_field(value, member_name)?;
                            }
                            TK_STRUCTURE => {
                                let complex_value = self.get_complex_value(member_id)?;
                                final_serializer.serialize_field(complex_value, member_name)?;
                            }

                            _ => todo!("Not implemented for members of type {member_type_kind:x?}"),
                        }
                    }
                    Ok(())
                }
                ExtensibilityKind::Appendable => {
                    let mutappendable_serializer = serializer.serialize_appendable_struct()?;
                    todo!()
                }
                ExtensibilityKind::Mutable => {
                    let mut mutable_serializer = serializer.serialize_mutable_struct()?;
                    for field_index in 0..self.get_item_count() {
                        let member_id = self.get_member_id_at_index(field_index)?;
                        let member_descriptor = self.get_descriptor(member_id)?;
                        let member_type_kind = member_descriptor.r#type.get_kind();
                        let member_name = &member_descriptor.name;
                        let pid = member_id;
                        match member_type_kind {
                            TK_UINT8 => {
                                let value = self.get_uint8_value(member_id)?;
                                mutable_serializer.serialize_field(value, pid, member_name)?;
                            }
                            TK_UINT16 => {
                                let value = self.get_uint16_value(member_id)?;
                                mutable_serializer.serialize_field(value, pid, member_name)?;
                            }
                            TK_UINT32 => {
                                let value = self.get_uint32_value(member_id)?;
                                mutable_serializer.serialize_field(value, pid, member_name)?;
                            }
                            TK_INT16 => {
                                let value = self.get_int16_value(member_id)?;
                                mutable_serializer.serialize_field(value, pid, member_name)?;
                            }
                            TK_INT32 => {
                                let value = self.get_int32_value(member_id)?;
                                mutable_serializer.serialize_field(value, pid, member_name)?;
                            }
                            TK_INT64 => {
                                let value = self.get_int64_value(member_id)?;
                                mutable_serializer.serialize_field(value, pid, member_name)?;
                            }
                            TK_STRUCTURE => {
                                let value = self.get_complex_value(member_id)?;
                                mutable_serializer.serialize_field(value, pid, member_name)?;
                            }

                            _ => {
                                todo!("Not implemented for members of type 0x{member_type_kind:x?}")
                            }
                        }
                    }
                    mutable_serializer.end()?;
                    Ok(())
                }
            },
            _ => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        infrastructure::type_support::TypeSupport, xtypes::xcdr_serializer::Xcdr1LeSerializer,
    };

    #[test]
    fn serialize_final_struct_cdr1() {
        #[derive(TypeSupport)]
        struct FinalStruct {
            id: u8,
            a: i32,
            b: u32,
            c: i16,
            d: u16,
        }

        let data = FinalStruct {
            id: 1,
            a: 100,
            b: 200,
            c: 10,
            d: 20,
        };
        let dynamic_sample = data.create_dynamic_sample();
        let mut buffer = vec![];
        let mut serializer = Xcdr1LeSerializer::new(&mut buffer);
        dynamic_sample.serialize(&mut serializer).unwrap();

        assert_eq!(
            &buffer,
            &[
                1, 0, 0, 0, //id
                100, 0, 0, 0, //a
                200, 0, 0, 0, //b
                10, 0, //c
                20, 0, //d
            ]
        )
    }

    #[test]
    fn serialize_nested_final_struct_cdr1() {
        #[derive(TypeSupport)]
        struct Nested {
            n: u16,
            m: i16,
        }
        #[derive(TypeSupport)]
        struct FinalStruct {
            a: i32,
            nested: Nested,
        }
        let dynamic_sample = FinalStruct {
            a: 1,
            nested: Nested { n: 2, m: 3 },
        }
        .create_dynamic_sample();
        let mut buffer = vec![];
        dynamic_sample
            .serialize(&mut Xcdr1LeSerializer::new(&mut buffer))
            .unwrap();
        assert_eq!(
            &buffer,
            &[
                1, 0, 0, 0, // a
                2, 0, 3, 0, // nested
            ]
        )
    }

    #[test]
    fn serialize_mutable_struct_cdr1() {
        #[derive(TypeSupport)]
        #[dust_dds(extensibility = "mutable")]
        struct MutableStruct {
            #[dust_dds(id = 0x07)]
            byte: u8,
            #[dust_dds(id = 0x08)]
            long: i32,
        }
        let dynamic_sample = MutableStruct { byte: 1, long: 2 }.create_dynamic_sample();
        let mut buffer = vec![];
        dynamic_sample
            .serialize(&mut Xcdr1LeSerializer::new(&mut buffer))
            .unwrap();
        assert_eq!(
            &buffer,
            &[
                0x07, 0x00, 1, 0, // PID | length
                1, 0, 0, 0, // byte | padding
                0x08, 0x00, 4, 0, // PID | length
                2, 0, 0, 0, // long
                1, 0, 0, 0, // Sentinel
            ]
        )
    }

    #[test]
    fn serialize_nested_mutable_struct_cdr1() {
        #[derive(TypeSupport)]
        #[dust_dds(extensibility = "mutable", nested)]
        struct NestedStruct {
            #[dust_dds(id = 0x09)]
            nested_byte: u8,
            #[dust_dds(id = 0x0A)]
            nested_long: i32,
        }
        #[derive(TypeSupport)]
        #[dust_dds(extensibility = "mutable")]
        struct MutableStruct {
            #[dust_dds(id = 0x07)]
            byte: u8,
            #[dust_dds(id = 0xFF)]
            nested: NestedStruct,
            #[dust_dds(id = 0x08)]
            long: i32,
        }
        let dynamic_sample = MutableStruct {
            byte: 1,
            long: 2,
            nested: NestedStruct {
                nested_byte: 11,
                nested_long: 22,
            },
        }
        .create_dynamic_sample();
        let mut buffer = vec![];
        dynamic_sample
            .serialize(&mut Xcdr1LeSerializer::new(&mut buffer))
            .unwrap();
        assert_eq!(
            &buffer,
            &[
                0x07, 0x00, 1, 0, // PID | length
                1, 0, 0, 0, // byte | padding
                0x09, 0x00, 1, 0, // PID | length
                11, 0, 0, 0, // nested_byte | padding
                0x0A, 0x00, 1, 0, // PID | length
                22, 0, 0, 0, // nested_long | padding
                0x08, 0x00, 4, 0, // PID | length
                2, 0, 0, 0, // long
                1, 0, 0, 0, // Sentinel
            ]
        )
    }
}
