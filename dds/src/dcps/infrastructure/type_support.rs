use core::u32;

use crate::{
    infrastructure::error::DdsResult,
    xtypes::dynamic_type::{
        DynamicData, DynamicDataFactory, DynamicType, DynamicTypeBuilderFactory, TK_BOOLEAN,
        TK_CHAR8, TK_FLOAT32, TK_FLOAT64, TK_INT16, TK_INT32, TK_INT64, TK_INT8, TK_STRING8,
        TK_UINT16, TK_UINT32, TK_UINT64, TK_UINT8,
    },
};
use alloc::vec::Vec;
pub use dust_dds_derive::{DdsType, TypeSupport};

/// The TypeSupport trait represents a type that can be transmitted by DDS.
pub trait TypeSupport {
    /// This operation returns the default name for the data-type represented by the TypeSupport.
    fn get_type_name() -> &'static str {
        std::any::type_name::<Self>()
    }

    /// This operation returns a ['DynamicType'] object corresponding to the TypeSupport’s data type.
    fn get_type() -> DynamicType;

    /// Create a sample of the TypeSupport’s data type with the contents of an input DynamicData object.
    fn create_sample(src: DynamicData) -> DdsResult<Self>
    where
        Self: Sized;

    /// Create a 'DynamicData' object with the contents of an input sample of the TypeSupport’s data type.
    fn create_dynamic_sample(self) -> DynamicData;
}
