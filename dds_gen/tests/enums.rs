use syn::File;

#[test]
fn enum_generation() {
    let idl = r#"
        enum Suits { Spades, Hearts, Diamonds, Clubs };
        enum Direction { North, East, South, West };
    "#;

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

    let result =
        syn::parse2::<File>(dust_dds_gen::compile_idl(idl).unwrap().parse().unwrap()).unwrap();

    assert_eq!(result, expected);
}
