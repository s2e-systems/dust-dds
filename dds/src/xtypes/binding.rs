use core::u32;

use crate::xtypes::{
    dynamic_type::TryConstructKind,
    type_object::{
        CollectionElementFlag, EquivalenceKind, PlainArrayLElemDefn, PlainArraySElemDefn,
        PlainCollectionHeader, PlainSequenceLElemDefn, StringLTypeDefn, TypeIdentifier,
    },
};

pub trait RustXTypesBinding {
    fn type_identifier() -> TypeIdentifier;
}

impl RustXTypesBinding for i32 {
    fn type_identifier() -> TypeIdentifier {
        TypeIdentifier::TkInt32Type
    }
}

impl RustXTypesBinding for u32 {
    fn type_identifier() -> TypeIdentifier {
        TypeIdentifier::TkUint32Type
    }
}

impl RustXTypesBinding for i8 {
    fn type_identifier() -> TypeIdentifier {
        TypeIdentifier::TkInt8Type
    }
}

impl RustXTypesBinding for u8 {
    fn type_identifier() -> TypeIdentifier {
        TypeIdentifier::TkUint8Type
    }
}

impl RustXTypesBinding for i16 {
    fn type_identifier() -> TypeIdentifier {
        TypeIdentifier::TkInt16Type
    }
}

impl RustXTypesBinding for u16 {
    fn type_identifier() -> TypeIdentifier {
        TypeIdentifier::TkUint16Type
    }
}

impl RustXTypesBinding for i64 {
    fn type_identifier() -> TypeIdentifier {
        TypeIdentifier::TkInt64Type
    }
}

impl RustXTypesBinding for u64 {
    fn type_identifier() -> TypeIdentifier {
        TypeIdentifier::TkUint64Type
    }
}

impl RustXTypesBinding for f32 {
    fn type_identifier() -> TypeIdentifier {
        TypeIdentifier::TkFloat32Type
    }
}

impl RustXTypesBinding for f64 {
    fn type_identifier() -> TypeIdentifier {
        TypeIdentifier::TkFloat64Type
    }
}

impl RustXTypesBinding for char {
    fn type_identifier() -> TypeIdentifier {
        TypeIdentifier::TkChar8Type
    }
}

impl RustXTypesBinding for bool {
    fn type_identifier() -> TypeIdentifier {
        TypeIdentifier::TkBoolean
    }
}

impl RustXTypesBinding for String {
    fn type_identifier() -> TypeIdentifier {
        TypeIdentifier::TiString8Large {
            string_ldefn: StringLTypeDefn { bound: u32::MAX },
        }
    }
}

impl<T: RustXTypesBinding, const N: usize> RustXTypesBinding for [T; N] {
    fn type_identifier() -> TypeIdentifier {
        assert!(
            N <= u32::MAX as usize,
            "Maximum allowed size of array is {}",
            u32::MAX
        );

        let header = PlainCollectionHeader {
            equiv_kind: EquivalenceKind::Complete,
            element_flags: CollectionElementFlag {
                try_construct: TryConstructKind::UseDefault,
                is_external: false,
            },
        };
        if N <= u8::MAX as usize {
            TypeIdentifier::TiPlainArraySmall {
                array_sdefn: Box::new(PlainArraySElemDefn {
                    header,
                    array_bound_seq: vec![N as u8],
                    element_identifier: T::type_identifier(),
                }),
            }
        } else {
            TypeIdentifier::TiPlainArrayLarge {
                array_ldefn: Box::new(PlainArrayLElemDefn {
                    header,
                    array_bound_seq: vec![N as u32],
                    element_identifier: T::type_identifier(),
                }),
            }
        }
    }
}

impl<T: RustXTypesBinding> RustXTypesBinding for Vec<T> {
    fn type_identifier() -> TypeIdentifier {
        let header = PlainCollectionHeader {
            equiv_kind: EquivalenceKind::Complete,
            element_flags: CollectionElementFlag {
                try_construct: TryConstructKind::UseDefault,
                is_external: false,
            },
        };
        TypeIdentifier::TiPlainSequenceLarge {
            seq_ldefn: Box::new(PlainSequenceLElemDefn {
                header,
                bound: u32::MAX,
                element_identifier: T::type_identifier(),
            }),
        }
    }
}

impl<T: RustXTypesBinding> RustXTypesBinding for &[T] {
    fn type_identifier() -> TypeIdentifier {
        let header = PlainCollectionHeader {
            equiv_kind: EquivalenceKind::Complete,
            element_flags: CollectionElementFlag {
                try_construct: TryConstructKind::UseDefault,
                is_external: false,
            },
        };
        TypeIdentifier::TiPlainSequenceLarge {
            seq_ldefn: Box::new(PlainSequenceLElemDefn {
                header,
                bound: u32::MAX,
                element_identifier: T::type_identifier(),
            }),
        }
    }
}
