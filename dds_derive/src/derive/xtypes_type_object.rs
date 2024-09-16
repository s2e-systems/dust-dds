use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, DeriveInput, Field, Result, Type};

use super::attributes::{get_input_extensibility, Extensibility};

fn is_field_optional(field: &Field) -> bool {
    match &field.ty {
        syn::Type::Path(field_type_path) if field_type_path.path.segments[0].ident == "Option" => {
            true
        }
        _ => false,
    }
}

fn get_type_identifier(_type: &Type) -> Result<TokenStream> {
    match _type {
        syn::Type::Array(field_type_array) => {
            let element_identifier = get_type_identifier(&field_type_array.elem)?;
            let len = &field_type_array.len;
            Ok(quote!{
                { if #len < 256 {
                    dust_dds::xtypes::type_object::TypeIdentifier::TiPlainArraySmall {
                        array_sdefn: Box::new(dust_dds::xtypes::type_object::PlainArraySElemDefn {
                            header: dust_dds::xtypes::type_object::PlainCollectionHeader {
                                equiv_kind: dust_dds::xtypes::type_object::EK_COMPLETE,
                                element_flags: dust_dds::xtypes::type_object::CollectionElementFlag {
                                    try_construct: dust_dds::xtypes::type_object::TryConstruct::Discard,
                                    is_external: false,
                                }
                            },
                            array_bound_seq: vec![#len],
                            element_identifier: #element_identifier,
                        })
                    }
                } else {
                    dust_dds::xtypes::type_object::TypeIdentifier::TiPlainArrayLarge {
                        array_sdefn: Box::new(dust_dds::xtypes::type_object::PlainArrayLElemDefn {
                            header: dust_dds::xtypes::type_object::PlainCollectionHeader {
                                equiv_kind: dust_dds::xtypes::type_object::EK_COMPLETE,
                                element_flags: dust_dds::xtypes::type_object::CollectionElementFlag {
                                    try_construct: dust_dds::xtypes::type_object::TryConstruct::Discard,
                                    is_external: false,
                                }
                            },
                            array_bound_seq: vec![#len],
                            element_identifier: #element_identifier,
                        })
                    }
                }}
        })
        },
        syn::Type::Path(field_type_path) => match field_type_path.path.get_ident() {
            Some(i) => match i.to_string().as_str() {
                "bool" => Ok(quote!(dust_dds::xtypes::type_object::TypeIdentifier::TkBoolean)),
                "i8" => Ok(quote!(dust_dds::xtypes::type_object::TypeIdentifier::TkInt8Type)),
                "i16" => Ok(quote!(dust_dds::xtypes::type_object::TypeIdentifier::TkInt16Type)),
                "i32" => Ok(quote!(dust_dds::xtypes::type_object::TypeIdentifier::TkInt32Type)),
                "i64" => Ok(quote!(dust_dds::xtypes::type_object::TypeIdentifier::TkInt64Type)),
                "u8" => Ok(quote!(dust_dds::xtypes::type_object::TypeIdentifier::TkUint8Type)),
                "u16" => Ok(quote!(dust_dds::xtypes::type_object::TypeIdentifier::TkUint16Type)),
                "u32" => Ok(quote!(dust_dds::xtypes::type_object::TypeIdentifier::TkUint32Type)),
                "u64" => Ok(quote!(dust_dds::xtypes::type_object::TypeIdentifier::TkUint64Type)),
                "f32" => Ok(quote!(dust_dds::xtypes::type_object::TypeIdentifier::TkFloat32Type)),
                "f64" => Ok(quote!(dust_dds::xtypes::type_object::TypeIdentifier::TkFloat64Type)),
                "f128" => Ok(quote!(dust_dds::xtypes::type_object::TypeIdentifier::TkFloat128Type)),
                "char" => Ok(quote!(dust_dds::xtypes::type_object::TypeIdentifier::TkChar8Type)),
                "String" | "str" => Ok(quote!(dust_dds::xtypes::type_object::TypeIdentifier::TiString8Small{
                    string_sdefn: dust_dds::xtypes::type_object::StringSTypeDefn {
                        bound: 0u8,
                    }
                })),
                _ => todo!(),
            }
            None => {
                if field_type_path.path.segments[0].ident == "Vec" {
                    let element_identifier = if let syn::PathArguments::AngleBracketed(a) = &field_type_path.path.segments[0].arguments {
                        if let syn::GenericArgument::Type(ty) = &a.args[0] {
                            get_type_identifier(ty)
                        } else {
                            Err(syn::Error::new(
                                _type.span(),
                                "Expected type argument inside angle brackets",))
                        }
                    } else {
                        todo!()
                    }?;
                    Ok(quote!{
                        dust_dds::xtypes::type_object::TypeIdentifier::TiPlainSequenceSmall {
                            seq_sdefn: Box::new(dust_dds::xtypes::type_object::StringSTypeDefn {
                                header: dust_dds::xtypes::type_object::PlainCollectionHeader {
                                    equiv_kind: dust_dds::xtypes::type_object::EK_COMPLETE,
                                    element_flags: dust_dds::xtypes::type_object::CollectionElementFlag {
                                        try_construct: dust_dds::xtypes::type_object::TryConstruct::Discard,
                                        is_external: false,
                                    }
                                },
                                bound: 0u8,
                                element_identifier: #element_identifier,
                            })
                        }
                    })
                } else if field_type_path.path.segments[0].ident == "Option" {
                    if let syn::PathArguments::AngleBracketed(a) = &field_type_path.path.segments[0].arguments {
                        if let syn::GenericArgument::Type(ty) = &a.args[0] {
                            get_type_identifier(ty)
                        } else {
                            Err(syn::Error::new(
                                _type.span(),
                                "Expected type argument inside angle brackets",))
                        }
                    } else {
                        todo!()
                    }

                } else {
                    todo!()
                }
            }
        },
        syn::Type::Slice(_) => todo!(),
        _ => Err(syn::Error::new(
            _type.span(),
            "Field type not supported for automatic XTypesTypeObject derive. Use a custom implementation instead",
        )),
    }
}

pub fn expand_xtypes_type_object(input: &DeriveInput) -> Result<TokenStream> {
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let ident = &input.ident;

    let complete_type_object_quote = match &input.data {
        syn::Data::Struct(data_struct) => {
            let type_name = ident.to_string();
            let (is_final, is_appendable, is_mutable) = match get_input_extensibility(input)? {
                Extensibility::Final => (true, false, false),
                Extensibility::Appendable => (false, true, false),
                Extensibility::Mutable => (false, false, true),
            };

            let is_nested = false;
            let is_autoid_hash = false;
            let struct_flags = quote! {
                dust_dds::xtypes::type_object::StructTypeFlag {
                    is_final: #is_final,
                    is_appendable: #is_appendable,
                    is_mutable: #is_mutable,
                    is_nested: #is_nested,
                    is_autoid_hash: #is_autoid_hash,
                }
            };
            let struct_header = quote! {
                dust_dds::xtypes::type_object::CompleteStructHeader {
                    base_type: dust_dds::xtypes::type_object::TypeIdentifier::TkNone,
                    detail: dust_dds::xtypes::type_object::CompleteTypeDetail {
                        ann_builtin: None,
                        ann_custom: None,
                        type_name: #type_name.to_string(),
                    },
                }
            };
            let mut member_seq = quote! {};
            for (field_index, field) in data_struct.fields.iter().enumerate() {
                let member_id = field_index as u32;
                let field_name = field
                    .ident
                    .as_ref()
                    .map(|i| i.to_string())
                    .unwrap_or(field_index.to_string());
                let is_optional = is_field_optional(field);
                let member_type_id = get_type_identifier(&field.ty)?;
                member_seq.extend(
                    quote! {dust_dds::xtypes::type_object::CompleteStructMember {
                        common: dust_dds::xtypes::type_object::CommonStructMember {
                            member_id: #member_id,
                            member_flags: dust_dds::xtypes::type_object::StructMemberFlag {
                                try_construct:
                                    dust_dds::xtypes::type_object::TryConstruct::Discard,
                                is_external: false,
                                is_optional: #is_optional,
                                is_must_undestand: true,
                                is_key: false,
                            },
                            member_type_id:
                                #member_type_id,
                        },
                        detail: dust_dds::xtypes::type_object::CompleteMemberDetail {
                            name: #field_name.to_string(),
                            ann_builtin: None,
                            ann_custom: None,
                        },
                    },},
                );
            }
            Ok(quote! {
                dust_dds::xtypes::type_object::CompleteTypeObject::TkStructure {
                    struct_type: dust_dds::xtypes::type_object::CompleteStructType {
                        struct_flags: #struct_flags,
                        header: #struct_header,
                        member_seq: vec![#member_seq],
                    },
                },
            })
        }
        syn::Data::Enum(_data_enum) => Ok(quote! {}),
        syn::Data::Union(data_union) => Err(syn::Error::new(
            data_union.union_token.span,
            "Union not supported",
        )),
    }?;

    Ok(quote! {
        impl #impl_generics dust_dds::xtypes::type_object::XTypesTypeObject for #ident #type_generics #where_clause {
            fn type_object() -> dust_dds::xtypes::type_object::TypeObject
            {
                dust_dds::xtypes::type_object::TypeObject::EkComplete {
                    complete: #complete_type_object_quote
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use quote::ToTokens;
    use syn::ItemImpl;

    use super::*;

    #[test]
    fn xtypes_serialize_final_struct_with_basic_types() {
        let input = syn::parse2::<DeriveInput>(
            "
            #[xtypes(extensibility = \"Mutable\")]
            struct MyData {
                id: u8,
                values: [u16;5],
                y: Option<i32>,
            }
        "
            .parse()
            .unwrap(),
        )
        .unwrap();

        let output_token_stream = expand_xtypes_type_object(&input).unwrap();
        let result = syn::parse2::<ItemImpl>(output_token_stream).unwrap();
        let expected = syn::parse2::<ItemImpl>(
            r#"
            impl dust_dds::xtypes::type_object::XTypesTypeObject for MyData {
                fn type_object() -> dust_dds::xtypes::type_object::TypeObject {
                    dust_dds::xtypes::type_object::TypeObject::EkComplete {
                        complete: dust_dds::xtypes::type_object::CompleteTypeObject::TkStructure {
                            struct_type: dust_dds::xtypes::type_object::CompleteStructType {
                                struct_flags: dust_dds::xtypes::type_object::StructTypeFlag {
                                    is_final: false,
                                    is_appendable: false,
                                    is_mutable: true,
                                    is_nested: false,
                                    is_autoid_hash: false,
                                },
                                header: dust_dds::xtypes::type_object::CompleteStructHeader {
                                    base_type: dust_dds::xtypes::type_object::TypeIdentifier::TkNone,
                                    detail: dust_dds::xtypes::type_object::CompleteTypeDetail {
                                        ann_builtin: None,
                                        ann_custom: None,
                                        type_name: "MyData".to_string(),
                                    },
                                },
                                member_seq: vec![
                                    dust_dds::xtypes::type_object::CompleteStructMember {
                                        common: dust_dds::xtypes::type_object::CommonStructMember {
                                            member_id: 0u32,
                                            member_flags: dust_dds::xtypes::type_object::StructMemberFlag {
                                                try_construct:
                                                    dust_dds::xtypes::type_object::TryConstruct::Discard,
                                                is_external: false,
                                                is_optional: false,
                                                is_must_undestand: true,
                                                is_key: false,
                                            },
                                            member_type_id:
                                                dust_dds::xtypes::type_object::TypeIdentifier::TkUint8Type,
                                        },
                                        detail: dust_dds::xtypes::type_object::CompleteMemberDetail {
                                            name: "id".to_string(),
                                            ann_builtin: None,
                                            ann_custom: None,
                                        },
                                    },
                                    dust_dds::xtypes::type_object::CompleteStructMember {
                                        common: dust_dds::xtypes::type_object::CommonStructMember {
                                            member_id: 1u32,
                                            member_flags: dust_dds::xtypes::type_object::StructMemberFlag {
                                                try_construct:
                                                    dust_dds::xtypes::type_object::TryConstruct::Discard,
                                                is_external: false,
                                                is_optional: false,
                                                is_must_undestand: true,
                                                is_key: false,
                                            },
                                            member_type_id: {
                                                if 5 < 256 {
                                                    dust_dds::xtypes::type_object::TypeIdentifier::TiPlainArraySmall {
                                                        array_sdefn: Box::new(dust_dds::xtypes::type_object::PlainArraySElemDefn {
                                                            header: dust_dds::xtypes::type_object::PlainCollectionHeader {
                                                                equiv_kind: dust_dds::xtypes::type_object::EK_COMPLETE,
                                                                element_flags: dust_dds::xtypes::type_object::CollectionElementFlag {
                                                                    try_construct: dust_dds::xtypes::type_object::TryConstruct::Discard,
                                                                    is_external: false,
                                                                }
                                                            },
                                                            array_bound_seq: vec![5],
                                                            element_identifier: dust_dds::xtypes::type_object::TypeIdentifier::TkUint16Type,
                                                        })
                                                    }
                                                } else {
                                                    dust_dds::xtypes::type_object::TypeIdentifier::TiPlainArrayLarge {
                                                        array_sdefn: Box::new(dust_dds::xtypes::type_object::PlainArrayLElemDefn {
                                                            header: dust_dds::xtypes::type_object::PlainCollectionHeader {
                                                                equiv_kind: dust_dds::xtypes::type_object::EK_COMPLETE,
                                                                element_flags: dust_dds::xtypes::type_object::CollectionElementFlag {
                                                                    try_construct: dust_dds::xtypes::type_object::TryConstruct::Discard,
                                                                    is_external: false,
                                                                }
                                                            },
                                                            array_bound_seq: vec![5],
                                                            element_identifier: dust_dds::xtypes::type_object::TypeIdentifier::TkUint16Type,
                                                        })
                                                    }
                                                }
                                            },
                                        },
                                        detail: dust_dds::xtypes::type_object::CompleteMemberDetail {
                                            name: "values".to_string(),
                                            ann_builtin: None,
                                            ann_custom: None,
                                        },
                                    },
                                    dust_dds::xtypes::type_object::CompleteStructMember {
                                        common: dust_dds::xtypes::type_object::CommonStructMember {
                                            member_id: 2u32,
                                            member_flags: dust_dds::xtypes::type_object::StructMemberFlag {
                                                try_construct:
                                                    dust_dds::xtypes::type_object::TryConstruct::Discard,
                                                is_external: false,
                                                is_optional: true,
                                                is_must_undestand: true,
                                                is_key: false,
                                            },
                                            member_type_id:
                                                dust_dds::xtypes::type_object::TypeIdentifier::TkInt32Type,
                                        },
                                        detail: dust_dds::xtypes::type_object::CompleteMemberDetail {
                                            name: "y".to_string(),
                                            ann_builtin: None,
                                            ann_custom: None,
                                        },
                                    },
                                ],
                            },
                        },
                    }
                }
            }
            "#
            .parse()
            .unwrap(),
        )
        .unwrap();

        assert_eq!(
            result,
            expected,
            "\n R: {:?} \n \n L: {:?} \n ",
            result.clone().into_token_stream().to_string(),
            expected.clone().into_token_stream().to_string()
        );
    }
}
