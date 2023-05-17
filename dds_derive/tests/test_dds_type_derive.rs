use dust_dds::topic_definition::type_support::DdsType;
use serde::{Deserialize, Serialize};

#[derive(DdsType)]
struct StructNoKey {
    a: i32,
    b: i32,
}

#[test]
fn test_struct_no_key_info() {
    assert_eq!(StructNoKey::type_name(), "StructNoKey");
    assert!(!StructNoKey::has_key());
}

#[test]
fn test_struct_no_key_get() {
    let snk = StructNoKey { a: 1, b: 2 };
    assert!(snk.get_serialized_key().as_ref().is_empty());
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
fn test_struct_with_key_info() {
    assert_eq!(StructWithKey::type_name(), "StructWithKey");
    assert!(StructWithKey::has_key());
}

#[test]
fn test_struct_with_key_get() {
    let swk = StructWithKey { a: 1, b: 2 };
    assert_eq!(swk.get_serialized_key(), [2, 0, 0, 0][..].into());
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
fn test_struct_many_keys_info() {
    assert_eq!(StructManyKeys::type_name(), "StructManyKeys");
    assert!(StructManyKeys::has_key());
}

#[test]
fn test_struct_many_keys_get() {
    let smk = StructManyKeys {
        a: 1,
        b: 2,
        c: 'X',
        d: false,
    };
    assert_eq!(smk.get_serialized_key(), [1, 0, 0, 0, b'X', 0][..].into());
}

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
fn test_dds_type_derive_with_generic_info() {
    assert_eq!(TypeWithGeneric::<i32>::type_name(), "TypeWithGeneric");
    assert!(TypeWithGeneric::<i32>::has_key());
}

#[test]
fn test_dds_type_derive_with_generic_get() {
    let twg = TypeWithGeneric {
        a: vec![0, 1, 0],
        b: 42,
    };
    assert_eq!(twg.get_serialized_key(), [42, 0, 0, 0][..].into())
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
fn test_tuple_no_key_info() {
    assert_eq!(TupleNoKey::type_name(), "TupleNoKey");
    assert!(!TupleNoKey::has_key());
}

#[test]
fn test_tuple_no_key_get() {
    let twk = TupleNoKey(10, 25);
    assert!(twk.get_serialized_key().as_ref().is_empty());
}

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
fn test_tuple_with_keys_info() {
    assert_eq!(TupleWithKeys::type_name(), "TupleWithKeys");
    assert!(TupleWithKeys::has_key());
}

#[test]
fn test_tuple_with_keys_get() {
    let twk = TupleWithKeys(1, 2, true, '🦀');
    assert_eq!(twk.get_serialized_key(), [2, 0, 0, 0, 1][..].into())
}

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
fn test_enum_no_key_info() {
    assert_eq!(EnumNoKey::type_name(), "EnumNoKey");
    assert!(!EnumNoKey::has_key());
}

#[test]
fn test_enum_no_key_get() {
    let enk = EnumNoKey::_Two;
    assert!(enk.get_serialized_key().as_ref().is_empty());
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
fn test_enum_key_info() {
    assert_eq!(EnumKey::type_name(), "EnumKey");
    assert!(EnumKey::has_key());
}

#[test]
fn test_enum_key_get() {
    let ek = EnumKey::_Two;
    assert_eq!(ek.get_serialized_key(), [1, 0, 0, 0][..].into());
}

#[test]
fn test_enum_key_set() {
    let mut ek = EnumKey::_One;
    let key = [1, 0, 0, 0][..].into();
    ek.set_key_fields_from_serialized_key(&key).unwrap();
    assert_eq!(ek, EnumKey::_Two);
}
