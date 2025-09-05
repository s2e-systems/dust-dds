use std::path::Path;

use syn::File;

use dust_dds_gen::parse_idl;
use dust_dds_gen::generate_rust_def;

#[test]
fn structs_generation() {
    let idl_file = Path::new("tests/structs_generation.idl");

    let expected = syn::parse2::<File>(
        r#"
            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
            pub struct Point {
                pub x: f64,
                pub y: f64,
            }
            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
            pub struct ChessSquare {
                #[dust_dds(key)] pub column: char,
                #[dust_dds(key)] pub line: u16,
            }
            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
            pub struct HelloWorld {
                pub message: String,
                pub id: u32,
            }
            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
            pub struct Sentence {
                pub words: Vec<String>,
                pub dependencies: Vec<Vec<u32>>,
            }
            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
            pub struct User {
                pub name: String,
                pub active: bool,
            }
    "#
        .parse()
        .unwrap(),
    )
    .unwrap();

    let result = syn::parse2::<File>(
        dust_dds_gen::compile_idl(idl_file)
            .unwrap()
            .parse()
            .unwrap(),
    )
    .unwrap();

    assert_eq!(result, expected);
}

#[test]
fn module_generation() {
    let idl_file = Path::new("tests/module_generation.idl");

    let expected = syn::parse2::<File>(
        r#"
        pub mod Game {
            pub mod Chess {
                #[derive(Debug)]
                pub enum ChessPiece {
                    Pawn,
                    Rook,
                    Knight,
                    Bishop,
                    Queen,
                    King,
                }
                #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
                pub struct ChessSquare {
                    pub column: char,
                    pub line: u16,
                }
            }
            pub mod Cards {
                #[derive(Debug)]
                pub enum Suit {
                    Spades,
                    Hearts,
                    Diamonds,
                    Clubs,
                }
            }
        }
        #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
        pub struct Point {
            pub x: f64,
            pub y: f64,
        }
    "#
        .parse()
        .unwrap(),
    )
    .unwrap();

    let result = syn::parse2::<File>(
        dust_dds_gen::compile_idl(idl_file)
            .unwrap()
            .parse()
            .unwrap(),
    )
    .unwrap();

    assert_eq!(result, expected);
}

#[test]
fn nested_types() {
    let idl_file = Path::new("tests/nested_types.idl");

    let expected = syn::parse2::<File>(
        r#"
            #[derive(Debug)]
            pub enum Presence {
                Present,
                NotPresent,
            }
            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
            pub struct Color {
                pub red: u8,
                pub green: u8,
                pub blue: u8,
            }

            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
            pub struct ColorSensor {
                pub state: Presence,
                pub value: Color,
            }"#
        .parse()
        .unwrap(),
    )
    .unwrap();

    let result = syn::parse2::<File>(
        dust_dds_gen::compile_idl(idl_file)
            .unwrap()
            .parse()
            .unwrap(),
    )
    .unwrap();

    assert_eq!(result, expected);
}

#[test]
fn parse_struct_with_annotations() {
    let idl = r#"
        @derive("Debug, Clone")
        @appendable
        struct Foo {
            @key long id;
            float value;
        };
    "#;

    let structs = parse_idl(idl).unwrap();

    println!("Parsed structs: {:#?}", structs);

    assert!(!structs.is_empty(), "Expected at least one struct, got none");

    let foo = &structs[0];
    assert_eq!(foo.name, "Foo");

    assert_eq!(foo.annotations.len(), 2);
    assert_eq!(foo.annotations[0].name, "derive");
    assert_eq!(foo.annotations[1].name, "appendable");

    let id_member = &foo.members[0];
    assert_eq!(id_member.name, "id");
    assert_eq!(id_member.annotations[0].name, "key");
}


#[test]
fn test_generate_rust_code_with_annotations() {
    let idl = r#"
        @derive("Debug, Clone")
        struct MyStruct {
            @rust_type("Vec<u8>")
            sequence<octet> data;
            @serde_skip
            long ignored_field;
        };
    "#;

    let structs = parse_idl(idl).expect("Failed to parse IDL");
    let rust_code = generate_rust_def(&structs[0]);

    let expected = r#"
        #[derive(Debug, Clone)]
        pub struct MyStruct {
            pub data: Vec<u8>,
            #[serde(skip)]
            pub ignored_field: i32,
        }
    "#;

    println!("Preprocessed IDL:\n{}", rust_code);
    println!("Parsed structs: {:#?}", structs);


    assert_eq!(rust_code.trim(), expected.trim(), "Generated code did not match expected");
}


#[test]
fn parse_struct_with_annotation_debug() {
    let idl = r#"
        @derive("Debug, Clone")
        struct MyStruct {
            @key long id;
        };
    "#;

    let structs = parse_idl(idl).unwrap();

    println!("Parsed structs: {:#?}", structs);

    dbg!(&structs);
    assert!(!structs.is_empty());

    assert_eq!(structs[0].annotations[0].name, "derive");
    assert_eq!(structs[0].annotations[0].parameters[0], r#"Debug, Clone"#);
    assert_eq!(structs[0].members[0].annotations[0].name, "key");
}

#[test]
fn test_generate_rust_code_with_psm_annotations() {
    let idl = r#"
        @derive("Debug, Clone")
        @appendable
        struct TestStruct {
            @key long field1;
            @appendable string field2;
            @mutable bool field3;
            @final octet field4;
            @hashid unsigned long long field5;
            @must_understand float field6;
            @optional string field7;
        };
    "#;

    // Parse the IDL to get the struct definitions
    let structs = parse_idl(idl).expect("Failed to parse IDL");
    assert_eq!(structs.len(), 1);
    let s = &structs[0];

    // Check top-level annotations on the struct
    assert!(s.annotations.iter().any(|a| a.name == "derive"));
    assert!(s.annotations.iter().any(|a| a.name == "appendable"));

    // Check member annotations presence
    let members = &s.members;
    assert!(members.iter().any(|m| m.annotations.iter().any(|a| a.name == "key")));
    assert!(members.iter().any(|m| m.annotations.iter().any(|a| a.name == "appendable")));
    assert!(members.iter().any(|m| m.annotations.iter().any(|a| a.name == "mutable")));
    assert!(members.iter().any(|m| m.annotations.iter().any(|a| a.name == "final")));
    assert!(members.iter().any(|m| m.annotations.iter().any(|a| a.name == "hashid")));
    assert!(members.iter().any(|m| m.annotations.iter().any(|a| a.name == "must_understand")));
    assert!(members.iter().any(|m| m.annotations.iter().any(|a| a.name == "optional")));

    // Generate Rust code from parsed struct
    let rust_code = generate_rust_def(s);

    // Verify that all dust_dds annotations appear in the generated code
    let expected_annotations = [
        "#[dust_dds(key)]",
        "#[dust_dds(appendable)]",
        "#[dust_dds(mutable)]",
        "#[dust_dds(final)]",
        "#[dust_dds(hashid)]",
        "#[dust_dds(must_understand)]",
        "#[dust_dds(optional)]",
    ];

    for ann in &expected_annotations {
        assert!(rust_code.contains(ann), "Expected annotation {} missing", ann);
    }

    // Also check struct-level derive and appendable appear as attributes (if you implement that)
    // For example:
    assert!(rust_code.contains("#[derive(Debug, Clone)]"));
    // (If you add support for struct-level dust_dds(appendable), test that here.)

    println!("Generated Rust code:\n{}", rust_code);
}
