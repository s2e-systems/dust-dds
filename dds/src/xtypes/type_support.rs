pub use dust_dds_derive::TypeSupport;


use crate::xtypes::dynamic_type::{DynamicType, DynamicData};

/// The Type trait represents static type information of Rust types
pub trait Type {
    const TYPE: DynamicType;
}

/// The TypeSupport trait represents a type that can be transmitted by DDS.
pub trait TypeSupport: Type {
    /// This constant represent the ['DynamicType'] object corresponding to the TypeSupport’s data type
    fn get_type() -> DynamicType {
        Self::TYPE
    }

    /// Create a sample of the TypeSupport’s data type with the contents of an input DynamicData object.
    fn create_sample(src: &mut DynamicData) -> Self;

    /// Create a 'DynamicData' object with the contents of an input sample of the TypeSupport’s data type.
    fn create_dynamic_sample(self, data: &mut DynamicData);
}
