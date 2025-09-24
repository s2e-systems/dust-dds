use dust_dds::{
    infrastructure::type_support::TypeSupport,
    xtypes::dynamic_type::{
        DynamicData, DynamicDataFactory, DynamicTypeBuilderFactory, ExtensibilityKind,
        MemberDescriptor, TryConstructKind, TypeDescriptor, TK_INT64, TK_STRING8, TK_STRUCTURE,
    },
};

#[test]
fn build_valid_primitive_types() {
    let kind = TK_INT64;
    let dynamic_type = DynamicTypeBuilderFactory::get_primitive_type(kind).unwrap();
    assert_eq!(dynamic_type.get_kind(), kind);
    assert_eq!(dynamic_type.get_name(), "");
    assert_eq!(dynamic_type.get_member_count(), 0);
}

#[test]
fn build_invalid_primitive_types() {
    assert!(DynamicTypeBuilderFactory::get_primitive_type(TK_STRUCTURE).is_err());
}

#[test]
fn create_type_with_builder() {
    let mut builder = DynamicTypeBuilderFactory::create_type(TypeDescriptor {
        kind: TK_STRUCTURE,
        name: String::from("Shape"),
        base_type: None,
        discriminator_type: None,
        bound: Vec::new(),
        element_type: None,
        key_element_type: None,
        extensibility_kind: ExtensibilityKind::Appendable,
        is_nested: false,
    });

    builder
        .add_member(MemberDescriptor {
            name: String::from("Color"),
            id: 0,
            r#type: <String as TypeSupport>::get_type(),
            default_value: String::new(),
            index: 0,
            try_construct_kind: TryConstructKind::UseDefault,
            label: vec![],
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
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Shape {
        color: String,
    }

    impl TypeSupport for Shape {
        fn get_type_name() -> &'static str {
            "Shape"
        }

        fn get_type() -> dust_dds::xtypes::dynamic_type::DynamicType {
            let mut builder = DynamicTypeBuilderFactory::create_type(TypeDescriptor {
                kind: TK_STRUCTURE,
                name: String::from("Shape"),
                base_type: None,
                discriminator_type: None,
                bound: Vec::new(),
                element_type: None,
                key_element_type: None,
                extensibility_kind: ExtensibilityKind::Final,
                is_nested: false,
            });
            builder
                .add_member(MemberDescriptor {
                    name: String::from("color"),
                    id: 0,
                    r#type: <String as TypeSupport>::get_type(),
                    default_value: String::new(),
                    index: 0,
                    label: Vec::new(),
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
            mut src: dust_dds::xtypes::dynamic_type::DynamicData,
        ) -> dust_dds::infrastructure::error::DdsResult<Self>
        where
            Self: Sized,
        {
            Ok(Self {
                color: src.remove_value(0)?,
            })
        }

        fn create_dynamic_sample(self) -> dust_dds::xtypes::dynamic_type::DynamicData {
            DynamicDataFactory::create_data(Self::get_type()).insert_value(0, self.color)
        }
    }

    let shape = Shape {
        color: String::from("BLUE"),
    };

    let dynamic_shape_data = shape.clone().create_dynamic_sample();

    assert_eq!(Shape::create_sample(dynamic_shape_data).unwrap(), shape);
}

#[test]
fn create_nested_type_with_data() {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct BaseStruct {
        x: i32,
    }

    impl TypeSupport for BaseStruct {
        fn get_type() -> dust_dds::xtypes::dynamic_type::DynamicType {
            let mut builder = DynamicTypeBuilderFactory::create_type(TypeDescriptor {
                kind: TK_STRUCTURE,
                name: String::from("Shape"),
                base_type: None,
                discriminator_type: None,
                bound: Vec::new(),
                element_type: None,
                key_element_type: None,
                extensibility_kind: ExtensibilityKind::Final,
                is_nested: false,
            });
            builder
                .add_member(MemberDescriptor {
                    name: String::from("x"),
                    id: 0,
                    r#type: <i32 as TypeSupport>::get_type(),
                    default_value: String::new(),
                    index: 0,
                    label: Vec::new(),
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
            mut src: dust_dds::xtypes::dynamic_type::DynamicData,
        ) -> dust_dds::infrastructure::error::DdsResult<Self>
        where
            Self: Sized,
        {
            Ok(Self {
                x: src.remove_value(0)?,
            })
        }

        fn create_dynamic_sample(self) -> dust_dds::xtypes::dynamic_type::DynamicData {
            DynamicDataFactory::create_data(Self::get_type()).insert_value(0, self.x)
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct NestedType {
        base: BaseStruct,
        color: String,
    }

    impl TypeSupport for NestedType {
        fn get_type() -> dust_dds::xtypes::dynamic_type::DynamicType {
            let mut builder = DynamicTypeBuilderFactory::create_type(TypeDescriptor {
                kind: TK_STRUCTURE,
                name: String::from("Shape"),
                base_type: None,
                discriminator_type: None,
                bound: Vec::new(),
                element_type: None,
                key_element_type: None,
                extensibility_kind: ExtensibilityKind::Final,
                is_nested: false,
            });
            builder
                .add_member(MemberDescriptor {
                    name: String::from("base"),
                    id: 0,
                    r#type: <BaseStruct as TypeSupport>::get_type(),
                    default_value: String::new(),
                    index: 0,
                    label: Vec::new(),
                    try_construct_kind: TryConstructKind::UseDefault,
                    is_key: false,
                    is_optional: false,
                    is_must_understand: true,
                    is_shared: false,
                    is_default_label: false,
                })
                .unwrap();
            builder
                .add_member(MemberDescriptor {
                    name: String::from("color"),
                    id: 1,
                    r#type: <String as TypeSupport>::get_type(),
                    default_value: String::new(),
                    index: 0,
                    label: Vec::new(),
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
            mut src: dust_dds::xtypes::dynamic_type::DynamicData,
        ) -> dust_dds::infrastructure::error::DdsResult<Self>
        where
            Self: Sized,
        {
            Ok(Self {
                base: src.remove_value(0)?,
                color: src.remove_value(1)?,
            })
        }

        fn create_dynamic_sample(self) -> dust_dds::xtypes::dynamic_type::DynamicData {
            DynamicDataFactory::create_data(Self::get_type())
                .insert_value(0, self.base)
                .insert_value(1, self.color)
        }
    }

    let data = NestedType {
        base: BaseStruct { x: 10 },
        color: String::from("BLUE"),
    };

    let dynamic_data = data.clone().create_dynamic_sample();

    assert_eq!(NestedType::create_sample(dynamic_data).unwrap(), data);
}
