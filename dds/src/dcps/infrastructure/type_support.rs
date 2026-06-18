use crate::xtypes::dynamic_type::{DynamicData, DynamicType};
pub use dust_dds_derive::TypeSupport;

pub trait Type {
    const TYPE_TYPE: DynamicType = dust_dds::xtypes::dynamic_type::DynamicType {
        descriptor: &dust_dds::xtypes::dynamic_type::TypeDescriptor {
            kind: dust_dds::xtypes::dynamic_type::TypeKind::NONE,
            name: "Inner",
            base_type: None,
            discriminator_type: None,
            bound: None,
            element_type: None,
            key_element_type: None,
            extensibility_kind: dust_dds::xtypes::dynamic_type::ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
}
/// The TypeSupport trait represents a type that can be transmitted by DDS.
pub trait TypeSupport {
    const TYPE: DynamicType = dust_dds::xtypes::dynamic_type::DynamicType {
        descriptor: &dust_dds::xtypes::dynamic_type::TypeDescriptor {
            kind: dust_dds::xtypes::dynamic_type::TypeKind::NONE,
            name: "Inner",
            base_type: None,
            discriminator_type: None,
            bound: None,
            element_type: None,
            key_element_type: None,
            extensibility_kind: dust_dds::xtypes::dynamic_type::ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };

    /// This constant represent the ['DynamicType'] object corresponding to the TypeSupport’s data type
    fn get_type() -> DynamicType
    where
        Self: Type,
    {
        Self::TYPE_TYPE
    }

    /// Create a sample of the TypeSupport’s data type with the contents of an input DynamicData object.
    fn create_sample(src: &mut DynamicData) -> Self;

    /// Create a 'DynamicData' object with the contents of an input sample of the TypeSupport’s data type.
    fn create_dynamic_sample(self, data: &mut DynamicData);
}

/// This is a convenience derive to allow the user to easily derive all the different traits needed for a type to be used for
/// communication with Dust DDS. If the individual traits are manually derived then this derive should not be used.
pub use dust_dds_derive::DdsType;
