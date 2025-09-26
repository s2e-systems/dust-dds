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

// Implementation for built-in types
impl TypeSupport for u8 {
    fn get_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TK_UINT8)
    }

    fn create_sample(mut src: DynamicData) -> DdsResult<Self>
    where
        Self: Sized,
    {
        Ok(src.remove_value(0)?)
    }

    fn create_dynamic_sample(self) -> DynamicData {
        DynamicDataFactory::create_data(Self::get_type()).insert_value(0, self)
    }
}

impl TypeSupport for u16 {
    fn get_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TK_UINT16)
    }

    fn create_sample(mut src: DynamicData) -> DdsResult<Self>
    where
        Self: Sized,
    {
        Ok(src.remove_value(0)?)
    }

    fn create_dynamic_sample(self) -> DynamicData {
        DynamicDataFactory::create_data(Self::get_type()).insert_value(0, self)
    }
}

impl TypeSupport for u32 {
    fn get_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TK_UINT32)
    }

    fn create_sample(mut src: DynamicData) -> DdsResult<Self>
    where
        Self: Sized,
    {
        Ok(src.remove_value(0)?)
    }

    fn create_dynamic_sample(self) -> DynamicData {
        DynamicDataFactory::create_data(Self::get_type()).insert_value(0, self)
    }
}

impl TypeSupport for u64 {
    fn get_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TK_UINT64)
    }

    fn create_sample(mut src: DynamicData) -> DdsResult<Self>
    where
        Self: Sized,
    {
        Ok(src.remove_value(0)?)
    }

    fn create_dynamic_sample(self) -> DynamicData {
        DynamicDataFactory::create_data(Self::get_type()).insert_value(0, self)
    }
}

impl TypeSupport for i8 {
    fn get_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TK_INT8)
    }

    fn create_sample(mut src: DynamicData) -> DdsResult<Self>
    where
        Self: Sized,
    {
        Ok(src.remove_value(0)?)
    }

    fn create_dynamic_sample(self) -> DynamicData {
        DynamicDataFactory::create_data(Self::get_type()).insert_value(0, self)
    }
}

impl TypeSupport for i16 {
    fn get_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TK_INT16)
    }

    fn create_sample(mut src: DynamicData) -> DdsResult<Self>
    where
        Self: Sized,
    {
        Ok(src.remove_value(0)?)
    }

    fn create_dynamic_sample(self) -> DynamicData {
        DynamicDataFactory::create_data(Self::get_type()).insert_value(0, self)
    }
}

impl TypeSupport for i32 {
    fn get_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TK_INT32)
    }

    fn create_sample(mut src: DynamicData) -> DdsResult<Self>
    where
        Self: Sized,
    {
        Ok(src.remove_value(0)?)
    }

    fn create_dynamic_sample(self) -> DynamicData {
        DynamicDataFactory::create_data(Self::get_type()).insert_value(0, self)
    }
}

impl TypeSupport for i64 {
    fn get_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TK_INT64)
    }

    fn create_sample(mut src: DynamicData) -> DdsResult<Self>
    where
        Self: Sized,
    {
        Ok(src.remove_value(0)?)
    }

    fn create_dynamic_sample(self) -> DynamicData {
        DynamicDataFactory::create_data(Self::get_type()).insert_value(0, self)
    }
}

impl TypeSupport for bool {
    fn get_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TK_BOOLEAN)
    }

    fn create_sample(mut src: DynamicData) -> DdsResult<Self>
    where
        Self: Sized,
    {
        Ok(src.remove_value(0)?)
    }

    fn create_dynamic_sample(self) -> DynamicData {
        DynamicDataFactory::create_data(Self::get_type()).insert_value(0, self)
    }
}

impl TypeSupport for char {
    fn get_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TK_CHAR8)
    }

    fn create_sample(mut src: DynamicData) -> DdsResult<Self>
    where
        Self: Sized,
    {
        Ok(src.remove_value(0)?)
    }

    fn create_dynamic_sample(self) -> DynamicData {
        DynamicDataFactory::create_data(Self::get_type()).insert_value(0, self)
    }
}

impl TypeSupport for f32 {
    fn get_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TK_FLOAT32)
    }

    fn create_sample(mut src: DynamicData) -> DdsResult<Self>
    where
        Self: Sized,
    {
        Ok(src.remove_value(0)?)
    }

    fn create_dynamic_sample(self) -> DynamicData {
        DynamicDataFactory::create_data(Self::get_type()).insert_value(0, self)
    }
}

impl TypeSupport for f64 {
    fn get_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TK_FLOAT64)
    }

    fn create_sample(mut src: DynamicData) -> DdsResult<Self>
    where
        Self: Sized,
    {
        Ok(src.remove_value(0)?)
    }

    fn create_dynamic_sample(self) -> DynamicData {
        DynamicDataFactory::create_data(Self::get_type()).insert_value(0, self)
    }
}

impl TypeSupport for String {
    fn get_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TK_STRING8)
    }

    fn create_sample(mut src: DynamicData) -> DdsResult<Self>
    where
        Self: Sized,
    {
        Ok(src.remove_value(0)?)
    }

    fn create_dynamic_sample(self) -> DynamicData {
        DynamicDataFactory::create_data(Self::get_type()).insert_value(0, self)
    }
}

impl<T: TypeSupport + Clone + Send + Sync + 'static, const N: usize> TypeSupport for [T; N] {
    fn get_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_array_type(T::get_type(), vec![N as u32]).build()
    }

    fn create_sample(mut src: DynamicData) -> DdsResult<Self>
    where
        Self: Sized,
    {
        Ok(src.remove_value(0)?)
    }

    fn create_dynamic_sample(self) -> DynamicData {
        DynamicDataFactory::create_data(Self::get_type()).insert_value(0, self.to_vec())
    }
}

impl<T: TypeSupport + Send + Sync + 'static> TypeSupport for Vec<T> {
    fn get_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(T::get_type(), u32::MAX).build()
    }

    fn create_sample(mut src: DynamicData) -> DdsResult<Self>
    where
        Self: Sized,
    {
        Ok(src.remove_value(0)?)
    }

    fn create_dynamic_sample(self) -> DynamicData {
        DynamicDataFactory::create_data(Self::get_type()).insert_value(0, self)
    }
}

impl TypeSupport for &'static [u8] {
    fn get_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(u8::get_type(), u32::MAX).build()
    }

    fn create_sample(mut src: DynamicData) -> DdsResult<Self>
    where
        Self: Sized,
    {
        Ok(src.remove_value(0)?)
    }

    fn create_dynamic_sample(self) -> DynamicData {
        DynamicDataFactory::create_data(Self::get_type()).insert_value(0, self.to_vec())
    }
}

impl<T: Send + Sync + 'static> TypeSupport for Option<T> {
    fn get_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(u8::get_type(), u32::MAX).build()
    }

    fn create_sample(mut src: DynamicData) -> DdsResult<Self>
    where
        Self: Sized,
    {
        Ok(src.remove_value(0)?)
    }

    fn create_dynamic_sample(self) -> DynamicData {
        DynamicDataFactory::create_data(Self::get_type()).insert_value(0, self)
    }
}
