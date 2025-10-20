use crate::xtypes::dynamic_type::{DynamicData, DynamicType};
pub use dust_dds_derive::TypeSupport;

/// The TypeSupport trait represents a type that can be transmitted by DDS.
pub trait TypeSupport {
    /// This operation returns the default name for the data-type represented by the TypeSupport.
    fn get_type_name() -> &'static str {
        core::any::type_name::<Self>()
    }

    /// This operation returns a ['DynamicType'] object corresponding to the TypeSupport’s data type
    fn get_type() -> DynamicType;

    /// Create a sample of the TypeSupport’s data type with the contents of an input DynamicData object.
    fn create_sample(src: DynamicData) -> Self;

    /// Create a 'DynamicData' object with the contents of an input sample of the TypeSupport’s data type.
    fn create_dynamic_sample(self) -> DynamicData;
}

/// This is a convenience derive to allow the user to easily derive all the different traits needed for a type to be used for
/// communication with Dust DDS. If the individual traits are manually derived then this derive should not be used.
pub use dust_dds_derive::DdsType;
