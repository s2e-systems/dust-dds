use std::path::Path;

use syn::File;

#[test]
fn basic_types() {
    let idl_file = Path::new("tests/basic_types.idl");
    let expected = syn::parse2::<File>(
        r#"
            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
            pub struct BasicTypes {
                pub a: bool,
                pub b: char,
                pub c: char,
                pub d: u8,
                pub e: String,
                pub f: String,
                pub g: i16,
                pub h: u16,
                pub i: i32,
                pub j: u32,
                pub k: i64,
                pub l: u64,
                pub m: f32,
                pub n: f64,
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
fn template_types() {
    let idl_file = Path::new("tests/template_types.idl");

    let expected = syn::parse2::<File>(
        r#"
            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
            pub struct TemplateTypes {
                pub a: Vec<Vec<u8>>,
                pub b: String,
                pub c: Vec<i16>,
                pub d: String,
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
fn enums() {
    let idl_file = Path::new("tests/enums.idl");

    let expected = syn::parse2::<File>(
        r#"
    #[derive(Debug)]
    pub enum Suits {
        Spades,
        Hearts,
        Diamonds,
        Clubs,
    }
    #[derive(Debug)]
    pub enum Direction {
        North,
        East,
        South,
        West,
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
