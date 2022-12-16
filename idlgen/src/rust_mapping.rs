use crate::idl_syntax;

pub fn base_type(t: idl_syntax::BaseType) -> String {
    match t {
        idl_syntax::BaseType::Float => "f32".to_string(),
        idl_syntax::BaseType::Double => "f64".to_string(),
        idl_syntax::BaseType::Short => "i16".to_string(),
        idl_syntax::BaseType::Long => "i32".to_string(),
        idl_syntax::BaseType::LongLong => "i64".to_string(),
        idl_syntax::BaseType::UnsignedShort => "u16".to_string(),
        idl_syntax::BaseType::UnsignedLong => "u32".to_string(),
        idl_syntax::BaseType::UnsignedLongLong => "u64".to_string(),
        idl_syntax::BaseType::Char => "char".to_string(),
        idl_syntax::BaseType::WChar => "char".to_string(),
        idl_syntax::BaseType::Boolean => "bool".to_string(),
        idl_syntax::BaseType::Octet => "u8".to_string(),
    }
}

pub fn struct_member(member: idl_syntax::StructMember) -> String {
    match member.datatype {
        idl_syntax::Type::BaseType(t) => format!("{}: {}", member.name, base_type(t)),
    }
}

pub fn struct_def(def: idl_syntax::Struct) -> impl Iterator<Item = String> {
    [
        "#[derive(serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsSerde, dust_dds::topic_definition::type_support::DdsType)]".to_string(),
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

pub fn enum_def(def: idl_syntax::Enum) -> impl Iterator<Item = String> {
    [
        "#[derive(serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsSerde, dust_dds::topic_definition::type_support::DdsType)]".to_string(),
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

pub fn module_def(def: idl_syntax::Module) -> impl Iterator<Item = String> {
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

pub fn definition(def: idl_syntax::Definition) -> Box<dyn Iterator<Item = String>> {
    match def {
        idl_syntax::Definition::Struct(s) => Box::new(struct_def(s)),
        idl_syntax::Definition::Enum(e) => Box::new(enum_def(e)),
        idl_syntax::Definition::Module(m) => Box::new(module_def(m)),
    }
}

#[cfg(test)]
mod tests {
    use crate::idl_syntax::{BaseType, Definition, Enum, Module, Struct, StructMember, Type};

    use super::*;

    #[test]
    fn test_rust_struct_mapping() {
        assert_eq!(
            struct_def(Struct {
                name: "Toto".to_string(),
                members: vec![
                    StructMember {
                        datatype: Type::BaseType(BaseType::LongLong),
                        name: "a".to_string()
                    },
                    StructMember {
                        datatype: Type::BaseType(BaseType::Char),
                        name: "b".to_string()
                    },
                    StructMember {
                        datatype: Type::BaseType(BaseType::Double),
                        name: "c".to_string()
                    },
                ]
            })
            .collect::<Vec<String>>(),
            vec![
                "#[derive(serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsSerde, dust_dds::topic_definition::type_support::DdsType)]",
                "pub struct Toto {",
                "    a: i64,",
                "    b: char,",
                "    c: f64,",
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
                "#[derive(serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsSerde, dust_dds::topic_definition::type_support::DdsType)]",
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
                            datatype: Type::BaseType(BaseType::Short),
                            name: "a".to_string()
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
                "    #[derive(serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsSerde, dust_dds::topic_definition::type_support::DdsType)]",
                "    struct A {",
                "        a: i16,",
                "    }",
                "    mod N {",
                "        #[derive(serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsSerde, dust_dds::topic_definition::type_support::DdsType)]",
                "        enum B {",
                "            C,",
                "            D,",
                "        }",
                "    }",
                "}",
            ]
        )
    }
}
