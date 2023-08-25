use dust_dds::topic_definition::type_support::DdsType;
use serde::{Deserialize, Serialize};

#[derive(DdsType)]
struct StructNoKey {
    a: i32,
    b: i32,
}

#[derive(DdsType)]
struct StructWithKey {
    a: i32,
    #[key]
    b: i32,
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

#[derive(DdsType)]
struct TupleNoKey(i32, i32);

#[derive(DdsType)]
struct TupleWithKeys(i32, #[key] i32, #[key] bool, char);

#[derive(DdsType, PartialEq, Eq, Debug)]
enum EnumNoKey {
    _One,
    _Two,
    _Three,
}

#[derive(Serialize, Deserialize, DdsType, PartialEq, Eq, Debug)]
#[key]
enum EnumKey {
    _One,
    _Two,
    _Three,
}
