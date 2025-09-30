use crate::xtypes::{
    dynamic_type::{
        DynamicData, ExtensibilityKind, TK_INT16, TK_INT32, TK_INT64, TK_STRUCTURE, TK_UINT16,
        TK_UINT32, TK_UINT8,
    },
    serialize::XTypesSerialize,
    serializer::SerializeFinalStruct,
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
                            _ => todo!("Not implemented for members of type {member_type_kind:x?}"),
                        }
                    }
                    Ok(())
                }
                ExtensibilityKind::Appendable => {
                    let appendable_serializer = serializer.serialize_appendable_struct()?;
                    todo!()
                }
                ExtensibilityKind::Mutable => {
                    let mutable_serializer = serializer.serialize_mutable_struct()?;
                    todo!()
                }
            },
            _ => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        infrastructure::type_support::TypeSupport, xtypes::xcdr_serializer::Xcdr1LeSerializer,
    };

    use super::*;

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
}
