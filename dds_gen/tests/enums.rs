#[test]
fn enum_generation() {
    let idl = r#"
        enum Suits { Spades, Hearts, Diamonds, Clubs };
        enum Direction { North, East, South, West };
    "#;

    let expected = r#"
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
    "#;

    let result = dust_dds_gen::compile_idl(idl).unwrap();

    assert_eq!(result, expected);
}
