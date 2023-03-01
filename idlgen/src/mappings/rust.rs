use crate::idl;

pub fn base_type(t: idl::BaseType) -> String {
    match t {
        idl::BaseType::Float => "f32".to_string(),
        idl::BaseType::Double => "f64".to_string(),
        idl::BaseType::Short => "i16".to_string(),
        idl::BaseType::Long => "i32".to_string(),
        idl::BaseType::LongLong => "i64".to_string(),
        idl::BaseType::UnsignedShort => "u16".to_string(),
        idl::BaseType::UnsignedLong => "u32".to_string(),
        idl::BaseType::UnsignedLongLong => "u64".to_string(),
        idl::BaseType::Char => "char".to_string(),
        idl::BaseType::WChar => "char".to_string(),
        idl::BaseType::Boolean => "bool".to_string(),
        idl::BaseType::Octet => "u8".to_string(),
    }
}

pub fn template_type(t: idl::TemplateType) -> String {
    match t {
        idl::TemplateType::Sequence(t, Some(size)) => {
            format!("[{}; {}]", type_spec(*t), size)
        }
        idl::TemplateType::Sequence(t, None) => format!("Vec<{}>", type_spec(*t)),

        idl::TemplateType::String(Some(_size)) => format!("String"),
        idl::TemplateType::String(None) => "String".to_string(),

        idl::TemplateType::WideString(Some(_size)) => format!("String"),
        idl::TemplateType::WideString(None) => "String".to_string(),
    }
}

pub fn type_spec(t: idl::Type) -> String {
    match t {
        idl::Type::BaseType(t) => base_type(t),
        idl::Type::TemplateType(t) => template_type(t),
    }
}

pub fn struct_member(member: idl::StructMember) -> String {
    let key_tag = if member.is_key {
        "#[key] "
    } else {
        ""
    };
    format!("{}pub {}: {}", key_tag, member.name, type_spec(member.datatype))
}

pub fn struct_def(def: idl::Struct) -> impl Iterator<Item = String> {
    [
        "#[derive(Debug, serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsSerde, dust_dds::topic_definition::type_support::DdsType)]".to_string(),
        format!("pub struct {} {{", def.name),
    ]
        .into_iter()
        .chain(
            def.members
                .into_iter()
                .map(|member| format!("    {},", struct_member(member))),
        )
        .chain(["}".to_string()].into_iter())
}

pub fn enum_def(def: idl::Enum) -> impl Iterator<Item = String> {
    [
        "#[derive(Debug, serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsSerde, dust_dds::topic_definition::type_support::DdsType)]".to_string(),
        format!("pub enum {} {{", def.name),
    ]
        .into_iter()
        .chain(
            def.variants
                .into_iter()
                .map(|variant| format!("    {},", variant)),
        )
        .chain(["}".to_string()].into_iter())
}

pub fn module_def(def: idl::Module) -> impl Iterator<Item = String> {
    [format!("mod {} {{", def.name)]
        .into_iter()
        .chain(
            def.definitions
                .into_iter()
                .flat_map(definition)
                .into_iter()
                .map(|line| "    ".to_string() + &line),
        )
        .chain(["}".to_string()].into_iter())
}

pub fn definition(def: idl::Definition) -> Box<dyn Iterator<Item = String>> {
    match def {
        idl::Definition::Struct(s) => Box::new(struct_def(s)),
        idl::Definition::Enum(e) => Box::new(enum_def(e)),
        idl::Definition::Module(m) => Box::new(module_def(m)),
    }
}

#[cfg(test)]
mod tests {
    use crate::idl::{BaseType, Definition, Enum, Module, Struct, StructMember, Type};

    use super::*;

    #[test]
    fn test_rust_struct_mapping() {
        assert_eq!(
            struct_def(Struct {
                name: "Toto".to_string(),
                members: vec![
                    StructMember {
                        is_key: false,
                        datatype: Type::BaseType(BaseType::LongLong),
                        name: "a".to_string()
                    },
                    StructMember {
                        is_key: false,
                        datatype: Type::BaseType(BaseType::Char),
                        name: "b".to_string()
                    },
                    StructMember {
                        is_key: false,
                        datatype: Type::BaseType(BaseType::Double),
                        name: "c".to_string()
                    },
                ]
            })
            .collect::<Vec<String>>(),
            vec![
                "#[derive(Debug, serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsSerde, dust_dds::topic_definition::type_support::DdsType)]",
                "pub struct Toto {",
                "    pub a: i64,",
                "    pub b: char,",
                "    pub c: f64,",
                "}",
            ]
        )
    }

    #[test]
    fn test_rust_enum_mapping() {
        assert_eq!(
            enum_def(Enum {
                name: "Suit".to_string(),
                variants: vec![
                    "Spades".to_string(),
                    "Hearts".to_string(),
                    "Diamonds".to_string(),
                    "Clubs".to_string()
                ]
            })
            .collect::<Vec<String>>(),
            vec![
                "#[derive(Debug, serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsSerde, dust_dds::topic_definition::type_support::DdsType)]",
                "pub enum Suit {",
                "    Spades,",
                "    Hearts,",
                "    Diamonds,",
                "    Clubs,",
                "}",
            ]
        )
    }

    #[test]
    fn test_rust_module_mapping() {
        assert_eq!(
            module_def(Module {
                name: "M".to_string(),
                definitions: vec![
                    Definition::Struct(Struct {
                        name: "A".to_string(),
                        members: vec![StructMember {
                            is_key: false,
                            datatype: Type::BaseType(BaseType::Short),
                            name: "a".to_string(),
                        }]
                    }),
                    Definition::Module(Module {
                        name: "N".to_string(),
                        definitions: vec![Definition::Enum(Enum {
                            name: "B".to_string(),
                            variants: vec!["C".to_string(), "D".to_string()]
                        })]
                    })
                ]
            })
            .collect::<Vec<String>>(),
            vec![
                "mod M {",
                "    #[derive(Debug, serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsSerde, dust_dds::topic_definition::type_support::DdsType)]",
                "    pub struct A {",
                "        pub a: i16,",
                "    }",
                "    mod N {",
                "        #[derive(Debug, serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsSerde, dust_dds::topic_definition::type_support::DdsType)]",
                "        pub enum B {",
                "            C,",
                "            D,",
                "        }",
                "    }",
                "}",
            ]
        )
    }
}
