use tracing::debug;

use crate::xtypes::{
    data_representation::cdr_reader::CdrPrimitiveTypeDeserialize,
    dynamic_type::{
        DynamicData, DynamicDataFactory, DynamicType, DynamicTypeMember, ExtensibilityKind,
        TypeKind,
    },
    error::{XTypesError, XTypesResult},
};

pub trait XTypesDeserialize {
    fn deserialize_complex_value(
        &mut self,
        dynamic_type: &DynamicType,
    ) -> XTypesResult<DynamicData> {
        let mut dynamic_data = DynamicDataFactory::create_data(dynamic_type.clone());
        self.deserialize_structure(dynamic_type, &mut dynamic_data)?;
        Ok(dynamic_data)
    }

    fn deserialize_structure(
        &mut self,
        dynamic_type: &DynamicType,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        // Deserialize the top-level which must be either a struct, an enum or a union.
        // No other types are supported at this stage because this is what we can get from DDS
        match dynamic_type.get_descriptor().kind {
            TypeKind::STRUCTURE => {
                // Extensibility kind must match the representation
                match dynamic_type.get_descriptor().extensibility_kind {
                    ExtensibilityKind::Final => {
                        self.deserialize_final_struct(dynamic_type, dynamic_data)
                    }
                    ExtensibilityKind::Appendable => {
                        self.deserialize_appendable_struct(dynamic_type, dynamic_data)
                    }
                    ExtensibilityKind::Mutable => {
                        self.deserialize_mutable_struct(dynamic_type, dynamic_data)
                    }
                }
            }
            //     TypeKind::ENUM => todo!(),
            //     TypeKind::UNION => todo!(),
            kind => {
                debug!("Expected structure, enum or union. Got kind {kind:?} ");
                return Err(XTypesError::InvalidType);
            }
        }
    }

    fn deserialize_final_struct(
        &mut self,
        dynamic_type: &DynamicType,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        for member_index in 0..dynamic_type.get_member_count() {
            let member = dynamic_type.get_member_by_index(member_index)?;
            self.deserialize_final_member(member, dynamic_data)?;
        }
        Ok(())
    }

    fn deserialize_appendable_struct(
        &mut self,
        dynamic_type: &DynamicType,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()>;

    fn deserialize_mutable_struct(
        &mut self,
        dynamic_type: &DynamicType,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()>;

    fn deserialize_string(&mut self) -> Result<String, XTypesError> {
        let length = self.deserialize_primitive_type::<u32>()?;
        let mut values = Vec::with_capacity(length as usize);
        for _ in 0..length - 1 {
            values.push(self.deserialize_primitive_type::<u8>()?);
        }
        self.deserialize_primitive_type::<u8>()?;
        String::from_utf8(values).map_err(|_| XTypesError::InvalidData)
    }

    fn deserialize_primitive_type<T: CdrPrimitiveTypeDeserialize>(&mut self) -> XTypesResult<T>;

    fn deserialize_final_member(
        &mut self,
        member: &DynamicTypeMember,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        let member_descriptor = member.get_descriptor()?;
        match member_descriptor.r#type.get_kind() {
            TypeKind::NONE => todo!(),
            TypeKind::BOOLEAN => todo!(),
            TypeKind::BYTE => todo!(),
            TypeKind::INT16 => {
                dynamic_data.set_int16_value(member.get_id(), self.deserialize_primitive_type()?)
            }
            TypeKind::INT32 => {
                dynamic_data.set_int32_value(member.get_id(), self.deserialize_primitive_type()?)
            }
            TypeKind::INT64 => {
                dynamic_data.set_int64_value(member.get_id(), self.deserialize_primitive_type()?)
            }
            TypeKind::UINT16 => {
                dynamic_data.set_uint16_value(member.get_id(), self.deserialize_primitive_type()?)
            }
            TypeKind::UINT32 => {
                dynamic_data.set_uint32_value(member.get_id(), self.deserialize_primitive_type()?)
            }
            TypeKind::UINT64 => {
                dynamic_data.set_uint64_value(member.get_id(), self.deserialize_primitive_type()?)
            }
            TypeKind::FLOAT32 => todo!(),
            TypeKind::FLOAT64 => todo!(),
            TypeKind::FLOAT128 => todo!(),
            TypeKind::INT8 => {
                dynamic_data.set_int8_value(member.get_id(), self.deserialize_primitive_type()?)
            }
            TypeKind::UINT8 => {
                dynamic_data.set_uint8_value(member.get_id(), self.deserialize_primitive_type()?)
            }
            TypeKind::CHAR8 => todo!(),
            TypeKind::CHAR16 => todo!(),
            TypeKind::STRING8 => {
                dynamic_data.set_string_value(member.get_id(), self.deserialize_string()?)
            }
            TypeKind::STRING16 => todo!(),
            TypeKind::ALIAS => todo!(),
            TypeKind::ENUM => todo!(),
            TypeKind::BITMASK => todo!(),
            TypeKind::ANNOTATION => todo!(),
            TypeKind::STRUCTURE => dynamic_data.set_complex_value(
                member.get_id(),
                self.deserialize_complex_value(&member_descriptor.r#type)?,
            ),
            TypeKind::UNION => todo!(),
            TypeKind::BITSET => todo!(),
            TypeKind::SEQUENCE => self.deserialize_sequence(&member, dynamic_data),
            TypeKind::ARRAY => self.deserialize_array(&member, dynamic_data),
            TypeKind::MAP => todo!(),
        }
        // self.deserialize_primitive_type()
    }

    fn deserialize_array(
        &mut self,
        member: &DynamicTypeMember,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        let sequence_type = member
            .get_descriptor()?
            .r#type
            .get_descriptor()
            .element_type
            .as_ref()
            .expect("Sequence must have element type");
        let bound = member.get_descriptor()?.r#type.get_descriptor().bound[0];
        match sequence_type.get_kind() {
            TypeKind::NONE => todo!(),
            TypeKind::BOOLEAN => todo!(),
            TypeKind::BYTE => todo!(),
            TypeKind::INT16 => todo!(),
            TypeKind::INT32 => todo!(),
            TypeKind::INT64 => todo!(),
            TypeKind::UINT16 => todo!(),
            TypeKind::UINT32 => dynamic_data.set_uint32_values(
                member.get_id(),
                self.deserialize_primitive_type_array::<u32>(bound)?,
            ),
            TypeKind::UINT64 => todo!(),
            TypeKind::FLOAT32 => todo!(),
            TypeKind::FLOAT64 => todo!(),
            TypeKind::FLOAT128 => todo!(),
            TypeKind::INT8 => todo!(),
            TypeKind::UINT8 => dynamic_data.set_uint8_values(
                member.get_id(),
                self.deserialize_primitive_type_array::<u8>(bound)?,
            ),
            TypeKind::CHAR8 => todo!(),
            TypeKind::CHAR16 => todo!(),
            TypeKind::STRING8 => todo!(),
            TypeKind::STRING16 => todo!(),
            TypeKind::ALIAS => todo!(),
            TypeKind::ENUM => todo!(),
            TypeKind::BITMASK => todo!(),
            TypeKind::ANNOTATION => todo!(),
            TypeKind::STRUCTURE => todo!(),
            TypeKind::UNION => todo!(),
            TypeKind::BITSET => todo!(),
            TypeKind::SEQUENCE => todo!(),
            TypeKind::ARRAY => todo!(),
            TypeKind::MAP => todo!(),
        }
    }

    fn deserialize_primitive_type_sequence<T: CdrPrimitiveTypeDeserialize>(
        &mut self,
    ) -> XTypesResult<Vec<T>> {
        let length = self.deserialize_primitive_type::<u32>()?;
        self.deserialize_primitive_type_array(length)
    }

    fn deserialize_primitive_type_array<T: CdrPrimitiveTypeDeserialize>(
        &mut self,
        length: u32,
    ) -> XTypesResult<Vec<T>> {
        let mut values = Vec::with_capacity(length as usize);
        for _ in 0..length {
            values.push(self.deserialize_primitive_type::<T>()?);
        }
        Ok(values)
    }

    fn deserialize_sequence(
        &mut self,
        member: &DynamicTypeMember,
        dynamic_data: &mut DynamicData,
    ) -> XTypesResult<()> {
        let sequence_type = member
            .get_descriptor()?
            .r#type
            .get_descriptor()
            .element_type
            .as_ref()
            .expect("Sequence must have element type");
        match sequence_type.get_kind() {
            TypeKind::NONE => todo!(),
            TypeKind::BOOLEAN => todo!(),
            TypeKind::BYTE => todo!(),
            TypeKind::INT16 => todo!(),
            TypeKind::INT32 => todo!(),
            TypeKind::INT64 => todo!(),
            TypeKind::UINT16 => todo!(),
            TypeKind::UINT32 => dynamic_data.set_uint32_values(
                member.get_id(),
                self.deserialize_primitive_type_sequence::<u32>()?,
            ),
            TypeKind::UINT64 => todo!(),
            TypeKind::FLOAT32 => todo!(),
            TypeKind::FLOAT64 => todo!(),
            TypeKind::FLOAT128 => todo!(),
            TypeKind::INT8 => todo!(),
            TypeKind::UINT8 => dynamic_data.set_uint8_values(
                member.get_id(),
                self.deserialize_primitive_type_sequence::<u8>()?,
            ),
            TypeKind::CHAR8 => todo!(),
            TypeKind::CHAR16 => todo!(),
            TypeKind::STRING8 => todo!(),
            TypeKind::STRING16 => todo!(),
            TypeKind::ALIAS => todo!(),
            TypeKind::ENUM => todo!(),
            TypeKind::BITMASK => todo!(),
            TypeKind::ANNOTATION => todo!(),
            TypeKind::STRUCTURE => todo!(),
            TypeKind::UNION => todo!(),
            TypeKind::BITSET => todo!(),
            TypeKind::SEQUENCE => todo!(),
            TypeKind::ARRAY => todo!(),
            TypeKind::MAP => todo!(),
        }
    }
}
