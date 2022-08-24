use cdr::{CdrBe, CdrLe};
use dust_dds::dds_type::{BigEndian, DdsType, LittleEndian};
use dust_dds_derive::DdsType;
use serde::{Deserialize, Serialize};

#[derive(DdsType)]
struct StructNoKey {
    a: i32,
    b: i32,
}

#[test]
fn test_struct_no_key_info() {
    assert_eq!(StructNoKey::type_name(), "StructNoKey");
    assert_eq!(StructNoKey::has_key(), false);
}

#[test]
fn test_struct_no_key_get() {
    let snk = StructNoKey { a: 1, b: 2 };
    assert!(snk.get_serialized_key::<BigEndian>().is_empty());
}

#[test]
fn test_struct_no_key_set() {
    let mut snk2 = StructNoKey { a: 3, b: 4 };
    snk2.set_key_fields_from_serialized_key(&[]).unwrap();
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
    assert_eq!(StructWithKey::has_key(), true);
}

#[test]
fn test_struct_with_key_get() {
    let swk = StructWithKey { a: 1, b: 2 };
    assert_eq!(
        swk.get_serialized_key::<BigEndian>(),
        cdr::serialize::<_, _, CdrBe>(&2i32, cdr::Infinite).unwrap()
    );
}

#[test]
fn test_struct_with_key_set() {
    let mut swk = StructWithKey { a: 0, b: 0 };
    let key = cdr::serialize::<_, _, CdrBe>(&42i32, cdr::Infinite).unwrap();
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
    assert_eq!(StructManyKeys::has_key(), true);
}

#[test]
fn test_struct_many_keys_get() {
    let smk = StructManyKeys {
        a: 1,
        b: 2,
        c: 'X',
        d: false,
    };
    assert_eq!(
        smk.get_serialized_key::<LittleEndian>(),
        cdr::serialize::<_, _, CdrLe>(&(1i32, 'X', false), cdr::Infinite).unwrap(),
    );
}

#[test]
fn test_struct_many_keys_set() {
    let mut smk = StructManyKeys {
        a: 0,
        b: 0,
        c: '\0',
        d: false,
    };
    let key = cdr::serialize::<_, _, CdrLe>(&(69i32, 'X', true), cdr::Infinite).unwrap();
    smk.set_key_fields_from_serialized_key(&key).unwrap();
    assert_eq!(smk.a, 69);
    assert_eq!(smk.b, 0);
    assert_eq!(smk.c, 'X');
    assert_eq!(smk.d, true);
}

#[derive(Serialize, Deserialize, DdsType)]
#[key]
struct StructAllKey {
    a: i32,
    b: i32,
    c: String,
}

#[test]
fn test_struct_all_key_info() {
    assert_eq!(StructAllKey::type_name(), "StructAllKey");
    assert_eq!(StructAllKey::has_key(), true);
}

#[test]
fn test_struct_all_key_get() {
    let sak = StructAllKey {
        a: 1,
        b: 2,
        c: "hello".to_string(),
    };
    assert_eq!(
        sak.get_serialized_key::<LittleEndian>(),
        cdr::serialize::<_, _, CdrLe>(&(1, 2, "hello".to_string()), cdr::Infinite).unwrap(),
    );
}

#[test]
fn test_struct_all_key_set() {
    let mut sak = StructAllKey {
        a: 0,
        b: 0,
        c: String::new(),
    };
    let key = cdr::serialize::<_, _, CdrLe>(&(1, 2, "hello".to_string()), cdr::Infinite).unwrap();
    sak.set_key_fields_from_serialized_key(&key).unwrap();

    assert_eq!(sak.a, 1);
    assert_eq!(sak.b, 2);
    assert_eq!(sak.c, "hello".to_string());
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
    assert_eq!(TypeWithGeneric::<i32>::has_key(), true);
}

#[test]
fn test_dds_type_derive_with_generic_get() {
    let twg = TypeWithGeneric {
        a: vec![0, 1, 0],
        b: 42,
    };
    assert_eq!(
        twg.get_serialized_key::<LittleEndian>(),
        cdr::serialize::<_, _, CdrLe>(&42i32, cdr::Infinite).unwrap()
    )
}

#[test]
fn test_dds_type_derive_with_generic_set() {
    let mut twg = TypeWithGeneric {
        a: vec![false],
        b: 0,
    };
    let key = cdr::serialize::<_, _, CdrLe>(&42i32, cdr::Infinite).unwrap();
    twg.set_key_fields_from_serialized_key(&key).unwrap();
    assert_eq!(twg.a, vec![false]);
    assert_eq!(twg.b, 42);
}

#[derive(DdsType)]
struct TupleNoKey(i32, i32);

#[test]
fn test_tuple_no_key_info() {
    assert_eq!(TupleNoKey::type_name(), "TupleNoKey");
    assert_eq!(TupleNoKey::has_key(), false);
}

#[test]
fn test_tuple_no_key_get() {
    let twk = TupleNoKey(10, 25);
    assert!(twk.get_serialized_key::<LittleEndian>().is_empty());
}

#[test]
fn test_tuple_no_key_set() {
    let mut twk = TupleNoKey(1, 2);
    twk.set_key_fields_from_serialized_key(&[]).unwrap();
    assert_eq!(twk.0, 1);
    assert_eq!(twk.1, 2);
}

#[derive(DdsType)]
struct TupleWithKeys(i32, #[key] i32, #[key] bool, char);

#[test]
fn test_tuple_with_keys_info() {
    assert_eq!(TupleWithKeys::type_name(), "TupleWithKeys");
    assert_eq!(TupleWithKeys::has_key(), true);
}

#[test]
fn test_tuple_with_keys_get() {
    let twk = TupleWithKeys(1, 2, true, '🦀');
    assert_eq!(
        twk.get_serialized_key::<LittleEndian>(),
        cdr::serialize::<_, _, CdrLe>(&(2i32, true), cdr::Infinite).unwrap(),
    )
}

#[test]
fn test_tuple_with_keys_set() {
    let mut twk = TupleWithKeys(0, 0, false, '\0');
    let key = cdr::serialize::<_, _, CdrLe>(&(2i32, true), cdr::Infinite).unwrap();
    twk.set_key_fields_from_serialized_key(&key).unwrap();
    assert_eq!(twk.0, 0);
    assert_eq!(twk.1, 2);
    assert_eq!(twk.2, true);
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
    assert_eq!(EnumNoKey::has_key(), false);
}

#[test]
fn test_enum_no_key_get() {
    let enk = EnumNoKey::_Two;
    assert!(enk.get_serialized_key::<LittleEndian>().is_empty());
}

#[test]
fn test_enum_no_key_set() {
    let mut enk = EnumNoKey::_Two;
    enk.set_key_fields_from_serialized_key(&[]).unwrap();
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
    assert_eq!(EnumKey::has_key(), true);
}

#[test]
fn test_enum_key_get() {
    let ek = EnumKey::_Two;
    assert_eq!(
        ek.get_serialized_key::<LittleEndian>(),
        cdr::serialize::<_, _, CdrLe>(&ek, cdr::Infinite).unwrap()
    );
}

#[test]
fn test_enum_key_set() {
    let mut ek = EnumKey::_One;
    let key = cdr::serialize::<_, _, CdrLe>(&EnumKey::_Two, cdr::Infinite).unwrap();
    ek.set_key_fields_from_serialized_key(&key).unwrap();
    assert_eq!(ek, EnumKey::_Two);
}
