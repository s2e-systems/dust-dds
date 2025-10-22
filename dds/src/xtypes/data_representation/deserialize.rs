use crate::xtypes::{
    dynamic_type::{DynamicData, DynamicType},
    error::XTypesResult,
};

pub trait XTypesDeserialize {
    fn deserialize_final_struct(&mut self, dynamic_type: DynamicType) -> XTypesResult<DynamicData>;

    fn deserialize_appendable_struct(
        &mut self,
        dynamic_type: DynamicType,
    ) -> XTypesResult<DynamicData>;

    fn deserialize_mutable_struct(
        &mut self,
        dynamic_type: DynamicType,
    ) -> XTypesResult<DynamicData>;
}
