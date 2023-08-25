use dust_dds::topic_definition::type_support::DdsType;

#[derive(DdsType)]
struct StructNoKey {
    _a: i32,
    _b: i32,
}

#[test]
fn struct_no_key() {
    assert_eq!(StructNoKey::has_key(), false);
}

#[derive(DdsType)]
struct StructWithKey {
    _a: i32,
    #[key]
    _b: i32,
}

#[test]
fn struct_with_key() {
    assert_eq!(StructWithKey::has_key(), true);
}

#[derive(DdsType)]
struct StructManyKeys {
    #[key]
    _a: i32,
    _b: i32,
    #[key]
    _c: char,
    #[key]
    _d: bool,
}

#[test]
fn struct_many_key() {
    assert_eq!(StructManyKeys::has_key(), true);
}

/*
 * Derive macros must take care of types with generics
 * See: https://naftuli.wtf/2019/01/02/rust-derive-macros/
 */

#[derive(DdsType)]
struct TypeWithGeneric<T> {
    _a: T,
    #[key]
    _b: i32,
}

#[test]
fn type_with_generic() {
    assert_eq!(TypeWithGeneric::<u8>::has_key(), true);
}

#[derive(DdsType)]
struct TupleNoKey(i32, i32);

#[test]
fn tuple_no_key() {
    assert_eq!(TupleNoKey::has_key(), false);
}

#[derive(DdsType)]
struct TupleWithKeys(i32, #[key] i32, #[key] bool, char);

#[test]
fn tuple_with_keys() {
    assert_eq!(TupleWithKeys::has_key(), true);
}
