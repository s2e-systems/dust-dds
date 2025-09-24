use dust_dds::{
    infrastructure::type_support::TypeSupport,
    xtypes::{
        dynamic_type::{
            DynamicDataFactory, DynamicTypeBuilderFactory, ExtensibilityKind, MemberDescriptor,
            TryConstructKind, TypeDescriptor,
        },
        type_object::{StringSTypeDefn, TypeIdentifier, TypeKind},
    },
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

#[test]
fn create_type_with_data() {
    let mut builder = DynamicTypeBuilderFactory::create_type(TypeDescriptor {
        kind: TypeKind::STRUCTURE,
        name: String::from("Shape"),
        extensibility_kind: ExtensibilityKind::Appendable,
        is_nested: false,
    });

    builder
        .add_member(MemberDescriptor {
            name: String::from("color"),
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

    let mut dynamic_shape_data = DynamicDataFactory::create_data(dynamic_shape_type);
    dynamic_shape_data
        .set_string_value(0, String::from("BLUE"))
        .unwrap();
    assert_eq!(dynamic_shape_data.get_string_value(0).unwrap(), "BLUE");
}

pub struct Shape {
    color: String,
}

impl TypeSupport for Shape {
    fn get_type_name() -> &'static str {
        "Shape"
    }

    fn get_type() -> dust_dds::xtypes::dynamic_type::DynamicType {
        let mut builder = DynamicTypeBuilderFactory::create_type(TypeDescriptor {
            kind: TypeKind::STRUCTURE,
            name: String::from("Shape"),
            extensibility_kind: ExtensibilityKind::Appendable,
            is_nested: false,
        });
        builder
            .add_member(MemberDescriptor {
                name: String::from("color"),
                id: 0,
                r#type: TypeIdentifier::TiString8Small {
                    string_sdefn: StringSTypeDefn { bound: 128 },
                },
                default_value: "",
                index: 0,
                try_construct_kind: TryConstructKind::UseDefault,
                is_key: false,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder.build()
    }

    fn create_sample(
        src: dust_dds::xtypes::dynamic_type::DynamicData,
    ) -> dust_dds::infrastructure::error::DdsResult<Self>
    where
        Self: Sized,
    {
        todo!()
    }

    fn create_dynamic_sample(&self) -> dust_dds::xtypes::dynamic_type::DynamicData {
        todo!()
    }
}
