use dust_dds::xtypes::{
    dynamic_type::{
        DynamicTypeBuilderFactory, ExtensibilityKind, MemberDescriptor, TryConstructKind,
        TypeDescriptor,
    },
    type_object::{StringSTypeDefn, TypeIdentifier, TypeKind},
};

#[test]
fn build_valid_primitive_types() {
    let kind = TypeKind::INT64;
    let dynamic_type = DynamicTypeBuilderFactory::get_primitive_type(kind);
    assert_eq!(dynamic_type.get_kind(), kind);
    assert_eq!(dynamic_type.get_name(), "");
    assert_eq!(dynamic_type.get_member_count(), 0);
}

#[test]
fn build_invalid_primitive_types() {
    let dynamic_type = DynamicTypeBuilderFactory::get_primitive_type(TypeKind::STRUCTURE);
    assert_eq!(dynamic_type.get_kind(), TypeKind::NONE);
    assert_eq!(dynamic_type.get_name(), "");
    assert_eq!(dynamic_type.get_member_count(), 0);
}

#[test]
fn create_type_with_builder() {
    let mut builder = DynamicTypeBuilderFactory::create_type(TypeDescriptor {
        kind: TypeKind::STRUCTURE,
        name: String::from("Shape"),
        extensibility_kind: ExtensibilityKind::Appendable,
        is_nested: false,
    });

    builder
        .add_member(MemberDescriptor {
            name: String::from("Color"),
            id: 0,
            r#type: TypeIdentifier::TiString8Small {
                string_sdefn: StringSTypeDefn { bound: 128 },
            },
            default_value: "",
            index: 0,
            try_construct_kind: TryConstructKind::UseDefault,
            is_key: true,
            is_optional: false,
            is_must_understand: true,
            is_shared: false,
            is_default_label: false,
        })
        .unwrap();

    let dynamic_shape_type = builder.build();
}
