use dust_dds::topic_definition::type_support::DdsType;
use serde::{Deserialize, Serialize};

#[derive(DdsType)]
struct StructNoKey {
    a: i32,
    b: i32,
}

#[test]
fn test_struct_no_key_set() {
    let mut snk2 = StructNoKey { a: 3, b: 4 };
    snk2.set_key_fields_from_serialized_key(&[][..].into())
        .unwrap();
    assert_eq!(snk2.a, 3);
    assert_eq!(snk2.b, 4);
}

#[derive(DdsType)]
struct StructWithKey {
    a: i32,
    #[key]
    b: i32,
}

#[test]
fn test_struct_with_key_set() {
    let mut swk = StructWithKey { a: 0, b: 0 };
    let key = [42, 0, 0, 0][..].into();
    swk.set_key_fields_from_serialized_key(&key).unwrap();
    assert_eq!(swk.a, 0);
    assert_eq!(swk.b, 42)
}

#[derive(DdsType)]
struct StructManyKeys {
    #[key]
    a: i32,
    b: i32,
    #[key]
    c: char,
    #[key]
    d: bool,
}

/*
 * cdr::serialize only seems to work for ascii characters (I tried a few
 * Unicode and Latin-1 chars)
 */

#[test]
fn test_struct_many_keys_set() {
    let mut smk = StructManyKeys {
        a: 0,
        b: 0,
        c: '\0',
        d: false,
    };
    let key = [69, 0, 0, 0, b'X', 1][..].into();
    smk.set_key_fields_from_serialized_key(&key).unwrap();
    assert_eq!(smk.a, 69);
    assert_eq!(smk.b, 0);
    assert_eq!(smk.c, 'X');
    assert!(smk.d);
}

/*
 * Derive macros must take care of types with generics
 * See: https://naftuli.wtf/2019/01/02/rust-derive-macros/
 */

#[derive(DdsType)]
struct TypeWithGeneric<T> {
    a: T,
    #[key]
    b: i32,
}

#[test]
fn test_dds_type_derive_with_generic_set() {
    let mut twg = TypeWithGeneric {
        a: vec![false],
        b: 0,
    };
    let key = [42, 0, 0, 0][..].into();
    twg.set_key_fields_from_serialized_key(&key).unwrap();
    assert_eq!(twg.a, vec![false]);
    assert_eq!(twg.b, 42);
}

#[derive(DdsType)]
struct TupleNoKey(i32, i32);

#[test]
fn test_tuple_no_key_set() {
    let mut twk = TupleNoKey(1, 2);
    twk.set_key_fields_from_serialized_key(&[][..].into())
        .unwrap();
    assert_eq!(twk.0, 1);
    assert_eq!(twk.1, 2);
}

#[derive(DdsType)]
struct TupleWithKeys(i32, #[key] i32, #[key] bool, char);

#[test]
fn test_tuple_with_keys_set() {
    let mut twk = TupleWithKeys(0, 0, false, '\0');
    let key = [2, 0, 0, 0, 1][..].into();
    twk.set_key_fields_from_serialized_key(&key).unwrap();
    assert_eq!(twk.0, 0);
    assert_eq!(twk.1, 2);
    assert!(twk.2);
    assert_eq!(twk.3, '\0');
}

#[derive(DdsType, PartialEq, Eq, Debug)]
enum EnumNoKey {
    _One,
    _Two,
    _Three,
}

#[test]
fn test_enum_no_key_set() {
    let mut enk = EnumNoKey::_Two;
    enk.set_key_fields_from_serialized_key(&[][..].into())
        .unwrap();
    assert_eq!(enk, EnumNoKey::_Two);
}

#[derive(Serialize, Deserialize, DdsType, PartialEq, Eq, Debug)]
#[key]
enum EnumKey {
    _One,
    _Two,
    _Three,
}

#[test]
fn test_enum_key_set() {
    let mut ek = EnumKey::_One;
    let key = [1, 0, 0, 0][..].into();
    ek.set_key_fields_from_serialized_key(&key).unwrap();
    assert_eq!(ek, EnumKey::_Two);
}
