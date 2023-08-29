use dust_dds::topic_definition::type_support::DdsHasKey;

#[derive(DdsHasKey)]
struct StructNoKey {
    _a: i32,
    _b: i32,
}

#[test]
fn struct_no_key() {
    assert_eq!(StructNoKey::HAS_KEY, false);
}

#[derive(DdsHasKey)]
struct StructWithKey {
    _a: i32,
    #[key]
    _b: i32,
}

#[test]
fn struct_with_key() {
    assert_eq!(StructWithKey::HAS_KEY, true);
}

#[derive(DdsHasKey)]
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
    assert_eq!(StructManyKeys::HAS_KEY, true);
}

/*
 * Derive macros must take care of types with generics
 * See: https://naftuli.wtf/2019/01/02/rust-derive-macros/
 */

#[derive(DdsHasKey)]
struct TypeWithGeneric<T> {
    _a: T,
    #[key]
    _b: i32,
}

#[test]
fn type_with_generic() {
    assert_eq!(TypeWithGeneric::<u8>::HAS_KEY, true);
}

#[derive(DdsHasKey)]
struct TupleNoKey(i32, i32);

#[test]
fn tuple_no_key() {
    assert_eq!(TupleNoKey::HAS_KEY, false);
}

#[derive(DdsHasKey)]
struct TupleWithKeys(i32, #[key] i32, #[key] bool, char);

#[test]
fn tuple_with_keys() {
    assert_eq!(TupleWithKeys::HAS_KEY, true);
}
