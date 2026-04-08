use crate::xtypes::{binding::XTypesBinding, dynamic_type::DynamicTypeMember};

pub mod discovered_reader_data;
pub mod discovered_topic_data;
pub mod discovered_writer_data;
pub mod parameter_id_values;
pub mod spdp_discovered_participant_data;

struct ConvenienceTypeBuilder;
impl ConvenienceTypeBuilder {
    const fn type_descriptor(name: &'static str) -> dust_dds::xtypes::dynamic_type::TypeDescriptor {
        dust_dds::xtypes::dynamic_type::TypeDescriptor {
            kind: dust_dds::xtypes::dynamic_type::TypeKind::STRUCTURE,
            name,
            base_type: None,
            discriminator_type: None,
            bound: None,
            element_type: None,
            key_element_type: None,
            extensibility_kind: dust_dds::xtypes::dynamic_type::ExtensibilityKind::Mutable,
            is_nested: false,
        }
    }
    const fn key_member<T: XTypesBinding>(
        index: u32,
        name: &'static str,
        id: i16,
    ) -> DynamicTypeMember {
        DynamicTypeMember {
            descriptor: dust_dds::xtypes::dynamic_type::MemberDescriptor {
                name,
                id: id as u32,
                r#type: T::TYPE_INFORMATION,
                default_value: None,
                index,
                try_construct_kind: dust_dds::xtypes::dynamic_type::TryConstructKind::UseDefault,
                label: None,
                is_key: true,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            },
        }
    }

    const fn member<T: XTypesBinding>(
        index: u32,
        name: &'static str,
        id: i16,
    ) -> DynamicTypeMember {
        DynamicTypeMember {
            descriptor: dust_dds::xtypes::dynamic_type::MemberDescriptor {
                name,
                id: id as u32,
                r#type: T::TYPE_INFORMATION,
                default_value: None,
                index,
                try_construct_kind: dust_dds::xtypes::dynamic_type::TryConstructKind::UseDefault,
                label: None,
                is_key: false,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            },
        }
    }

    const fn member_with_default<T: XTypesBinding>(
        index: u32,
        name: &'static str,
        id: i16,
    ) -> DynamicTypeMember {
        DynamicTypeMember {
            descriptor: dust_dds::xtypes::dynamic_type::MemberDescriptor {
                name,
                id: id as u32,
                r#type: T::TYPE_INFORMATION,
                default_value: None,
                index,
                try_construct_kind: dust_dds::xtypes::dynamic_type::TryConstructKind::UseDefault,
                label: None,
                is_key: false,
                is_optional: true,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            },
        }
    }
}
