use cdr::{CdrBe, CdrLe};
use dds_derive::DdsType;
use dds_implementation::dds_type::{BigEndian, DdsType, LittleEndian};
use serde::Serialize;

#[derive(DdsType)]
struct StructNoKey {
    _a: i32,
    _b: i32,
}

#[test]
fn test_struct_no_key() {
    assert_eq!(StructNoKey::type_name(), "StructNoKey");
    assert_eq!(StructNoKey::has_key(), false);

    let snk = StructNoKey { _a: 1, _b: 2 };
    assert_eq!(snk.serialized_key::<BigEndian>(), vec![]);
}

#[derive(DdsType)]
struct StructWithKey {
    _a: i32,
    #[key]
    _b: i32,
}

#[test]
fn test_struct_with_key() {
    assert_eq!(StructWithKey::type_name(), "StructWithKey");
    assert_eq!(StructWithKey::has_key(), true);

    let swk = StructWithKey { _a: 1, _b: 2 };
    assert_eq!(
        swk.serialized_key::<BigEndian>(),
        cdr::serialize::<_, _, CdrBe>(&2i32, cdr::Infinite).unwrap()
    );
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

/*
 * cdr::serialize only seems to work for ascii characters (I tried a few
 * Unicode and Latin-1 chars)
 */

#[test]
fn test_struct_many_keys() {
    assert_eq!(StructManyKeys::type_name(), "StructManyKeys");
    assert_eq!(StructManyKeys::has_key(), true);

    let smk = StructManyKeys {
        _a: 1,
        _b: 2,
        _c: 'X',
        _d: false,
    };
    assert_eq!(
        smk.serialized_key::<LittleEndian>(),
        [
            cdr::serialize::<_, _, CdrLe>(&1i32, cdr::Infinite).unwrap(),
            cdr::serialize::<_, _, CdrLe>(&'X', cdr::Infinite).unwrap(),
            cdr::serialize::<_, _, CdrLe>(&false, cdr::Infinite).unwrap(),
        ]
        .concat()
    );
}

#[derive(Serialize, DdsType)]
#[key]
struct StructAllKey {
    _a: i32,
    _b: i32,
    _c: String,
}

/*
 * Because of the way cdr::serialize works, marking the whole struct with
 * #[key] is not equivalent with marking each individual field with #[key]
 */

#[test]
fn test_struct_all_key() {
    assert_eq!(StructAllKey::type_name(), "StructAllKey");
    assert_eq!(StructAllKey::has_key(), true);

    let sak = StructAllKey {
        _a: 1,
        _b: 2,
        _c: "hello".to_string(),
    };
    assert_eq!(
        sak.serialized_key::<LittleEndian>(),
        cdr::serialize::<_, _, CdrLe>(&sak, cdr::Infinite).unwrap(),
    );
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
fn test_dds_type_derive_with_generic() {
    assert_eq!(TypeWithGeneric::<i32>::type_name(), "TypeWithGeneric");
    assert_eq!(TypeWithGeneric::<i32>::has_key(), true);

    let twg = TypeWithGeneric {
        _a: vec![0, 1, 0],
        _b: 42,
    };
    assert_eq!(
        twg.serialized_key::<LittleEndian>(),
        cdr::serialize::<_, _, CdrLe>(&42i32, cdr::Infinite).unwrap()
    )
}

#[derive(DdsType)]
struct TupleNoKey(i32, i32);

#[test]
fn test_tuple_no_key() {
    assert_eq!(TupleNoKey::type_name(), "TupleNoKey");
    assert_eq!(TupleNoKey::has_key(), false);

    let twk = TupleNoKey(10, 25);
    assert_eq!(twk.serialized_key::<LittleEndian>(), vec![]);
}

#[derive(DdsType)]
struct TupleWithKeys(i32, #[key] i32, #[key] bool, char);

#[test]
fn test_tuple_with_keys() {
    assert_eq!(TupleWithKeys::type_name(), "TupleWithKeys");
    assert_eq!(TupleWithKeys::has_key(), true);

    let twk = TupleWithKeys(1, 2, true, '🦀');
    assert_eq!(
        twk.serialized_key::<LittleEndian>(),
        [
            cdr::serialize::<_, _, CdrLe>(&2i32, cdr::Infinite).unwrap(),
            cdr::serialize::<_, _, CdrLe>(&true, cdr::Infinite).unwrap(),
        ]
        .concat()
    )
}

#[derive(DdsType)]
enum EnumNoKey {
    _One,
    _Two,
    _Three,
}

#[test]
fn test_enum_no_key() {
    assert_eq!(EnumNoKey::type_name(), "EnumNoKey");
    assert_eq!(EnumNoKey::has_key(), false);

    let enk = EnumNoKey::_Two;
    assert_eq!(enk.serialized_key::<LittleEndian>(), vec![]);
}

#[derive(Serialize, DdsType)]
#[key]
enum EnumKey {
    _One,
    _Two,
    _Three,
}

#[test]
fn test_enum_key() {
    assert_eq!(EnumKey::type_name(), "EnumKey");
    assert_eq!(EnumKey::has_key(), true);

    let ek = EnumKey::_Two;
    assert_eq!(
        ek.serialized_key::<LittleEndian>(),
        cdr::serialize::<_, _, CdrLe>(&ek, cdr::Infinite).unwrap()
    );
}

#[derive(DdsType)]
union UnionNoKey {
    _a: i32,
    _b: char,
}


#[test]
fn test_union_no_key() {
    assert_eq!(UnionNoKey::type_name(), "UnionNoKey");
    assert_eq!(UnionNoKey::has_key(), false);

    let unk = UnionNoKey { _b: '🦀' };
    assert_eq!(unk.serialized_key::<LittleEndian>(), vec![]);
}

/*
 * This doesn't work because Serde doesn't derive unions
 */

// #[derive(Serialize, DdsType)]
// #[key]
// union UnionKey {
//     _a: i32,
//     _b: [u8; 4],
// }

// #[test]
// fn test_union_key() {
//     assert_eq!(UnionKey::type_name(), "Union");
//     assert_eq!(UnionKey::has_key(), true);

//     let uk = UnionKey { _a: 413 };
//     assert_eq!(
//         uk.serialized_key::<LittleEndian>(),
//         cdr::serialize::<_, _, CdrLe>(&uk, cdr::Infinite).unwrap()
//     );
// }