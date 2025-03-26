use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, DeriveInput, Field, Result, Type};

use super::attributes::{get_field_attributes, get_input_extensibility, Extensibility};

fn is_field_optional(field: &Field) -> bool {
    matches!(&field.ty, syn::Type::Path(field_type_path) if field_type_path.path.segments[0].ident == "Option")
}

fn get_type_identifier(type_: &Type) -> Result<TokenStream> {
    match type_ {
        syn::Type::Array(field_type_array) => {
            let element_identifier = get_type_identifier(&field_type_array.elem)?;
            let len = &field_type_array.len;
            Ok(quote! {
                    { if #len < 256 {
                        dust_dds::xtypes::type_object::TypeIdentifier::TiPlainArraySmall {
                            array_sdefn: Box::new(dust_dds::xtypes::type_object::PlainArraySElemDefn {
                                header: dust_dds::xtypes::type_object::PlainCollectionHeader {
                                    equiv_kind: dust_dds::xtypes::type_object::EK_COMPLETE,
                                    element_flags: dust_dds::xtypes::type_object::CollectionElementFlag {
                                        try_construct: dust_dds::xtypes::dynamic_type::TryConstructKind::Discard,
                                        is_external: false,
                                    }
                                },
                                array_bound_seq: vec![dust_dds::xtypes::type_object::SBound::try_from(#len).unwrap()],
                                element_identifier: #element_identifier,
                            })
                        }
                    } else {
                        dust_dds::xtypes::type_object::TypeIdentifier::TiPlainArrayLarge {
                            array_ldefn: Box::new(dust_dds::xtypes::type_object::PlainArrayLElemDefn {
                                header: dust_dds::xtypes::type_object::PlainCollectionHeader {
                                    equiv_kind: dust_dds::xtypes::type_object::EK_COMPLETE,
                                    element_flags: dust_dds::xtypes::type_object::CollectionElementFlag {
                                        try_construct: dust_dds::xtypes::dynamic_type::TryConstructKind::Discard,
                                        is_external: false,
                                    }
                                },
                                array_bound_seq: vec![dust_dds::xtypes::type_object::LBound::try_from(#len).unwrap()],
                                element_identifier: #element_identifier,
                            })
                        }
                    }}
            })
        }
        syn::Type::Path(field_type_path) => match field_type_path.path.get_ident() {
            Some(i) => match i.to_string().as_str() {
                "bool" => Ok(quote!(
                    dust_dds::xtypes::type_object::TypeIdentifier::TkBoolean
                )),
                "i8" => Ok(quote!(
                    dust_dds::xtypes::type_object::TypeIdentifier::TkInt8Type
                )),
                "i16" => Ok(quote!(
                    dust_dds::xtypes::type_object::TypeIdentifier::TkInt16Type
                )),
                "i32" => Ok(quote!(
                    dust_dds::xtypes::type_object::TypeIdentifier::TkInt32Type
                )),
                "i64" => Ok(quote!(
                    dust_dds::xtypes::type_object::TypeIdentifier::TkInt64Type
                )),
                "u8" => Ok(quote!(
                    dust_dds::xtypes::type_object::TypeIdentifier::TkUint8Type
                )),
                "u16" => Ok(quote!(
                    dust_dds::xtypes::type_object::TypeIdentifier::TkUint16Type
                )),
                "u32" => Ok(quote!(
                    dust_dds::xtypes::type_object::TypeIdentifier::TkUint32Type
                )),
                "u64" => Ok(quote!(
                    dust_dds::xtypes::type_object::TypeIdentifier::TkUint64Type
                )),
                "f32" => Ok(quote!(
                    dust_dds::xtypes::type_object::TypeIdentifier::TkFloat32Type
                )),
                "f64" => Ok(quote!(
                    dust_dds::xtypes::type_object::TypeIdentifier::TkFloat64Type
                )),
                "f128" => Ok(quote!(
                    dust_dds::xtypes::type_object::TypeIdentifier::TkFloat128Type
                )),
                "char" => Ok(quote!(
                    dust_dds::xtypes::type_object::TypeIdentifier::TkChar8Type
                )),
                "String" | "str" => Ok(quote!(
                    dust_dds::xtypes::type_object::TypeIdentifier::TiString8Small {
                        string_sdefn: dust_dds::xtypes::type_object::StringSTypeDefn { bound: 0u8 }
                    }
                )),
                _ => Ok(quote!(
                    dust_dds::xtypes::type_object::TypeIdentifier::EkComplete {
                        complete: Box::new(<#i as dust_dds::topic_definition::type_support::TypeSupport>::get_type())
                    }
                )),
            },
            None => {
                if field_type_path.path.segments[0].ident == "Vec" {
                    let element_identifier = if let syn::PathArguments::AngleBracketed(a) =
                        &field_type_path.path.segments[0].arguments
                    {
                        if let syn::GenericArgument::Type(ty) = &a.args[0] {
                            get_type_identifier(ty)
                        } else {
                            Err(syn::Error::new(
                                type_.span(),
                                "Expected type argument inside angle brackets",
                            ))
                        }
                    } else {
                        todo!()
                    }?;
                    Ok(quote! {
                        dust_dds::xtypes::type_object::TypeIdentifier::TiPlainSequenceSmall {
                            seq_sdefn: Box::new(dust_dds::xtypes::type_object::PlainSequenceSElemDefn {
                                header: dust_dds::xtypes::type_object::PlainCollectionHeader {
                                    equiv_kind: dust_dds::xtypes::type_object::EK_COMPLETE,
                                    element_flags: dust_dds::xtypes::type_object::CollectionElementFlag {
                                        try_construct: dust_dds::xtypes::dynamic_type::TryConstructKind::Discard,
                                        is_external: false,
                                    }
                                },
                                bound: 0u8,
                                element_identifier: #element_identifier,
                            })
                        }
                    })
                } else if field_type_path.path.segments[0].ident == "Option" {
                    if let syn::PathArguments::AngleBracketed(a) =
                        &field_type_path.path.segments[0].arguments
                    {
                        if let syn::GenericArgument::Type(ty) = &a.args[0] {
                            get_type_identifier(ty)
                        } else {
                            Err(syn::Error::new(
                                type_.span(),
                                "Expected type argument inside angle brackets",
                            ))
                        }
                    } else {
                        todo!()
                    }
                } else {
                    todo!()
                }
            }
        },
        syn::Type::Reference(reference_type) => get_type_identifier(&reference_type.elem),
        syn::Type::Slice(slice_type) => {
            let element_identifier = get_type_identifier(&slice_type.elem)?;
            Ok(quote! {
                dust_dds::xtypes::type_object::TypeIdentifier::TiPlainSequenceSmall {
                    seq_sdefn: Box::new(dust_dds::xtypes::type_object::PlainSequenceSElemDefn {
                        header: dust_dds::xtypes::type_object::PlainCollectionHeader {
                            equiv_kind: dust_dds::xtypes::type_object::EK_COMPLETE,
                            element_flags: dust_dds::xtypes::type_object::CollectionElementFlag {
                                try_construct: dust_dds::xtypes::dynamic_type::TryConstructKind::Discard,
                                is_external: false,
                            }
                        },
                        bound: 0u8,
                        element_identifier: #element_identifier,
                    })
                }
            })
        }
        _ => Err(syn::Error::new(
            type_.span(),
            "Field type not supported for automatic derive. Use a custom implementation instead",
        )),
    }
}

pub fn expand_type_support(input: &DeriveInput) -> Result<TokenStream> {
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let ident = &input.ident;
    let ident_str = ident.to_string();

    let complete_type_object_quote = match &input.data {
        syn::Data::Struct(data_struct) => {
            let type_name = ident.to_string();
            let extensibility = get_input_extensibility(input)?;
            let (is_final, is_appendable, is_mutable) = match extensibility {
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
                let field_attributes = get_field_attributes(field)?;

                let member_id = match extensibility {
                    Extensibility::Final | Extensibility::Appendable => {
                        syn::parse_str(&field_index.to_string())
                    }
                    Extensibility::Mutable => field_attributes.id.ok_or(syn::Error::new(
                        field.span(),
                        "Mutable struct must define id attribute for every field",
                    )),
                }?;
                let field_name = field
                    .ident
                    .as_ref()
                    .map(|i| i.to_string())
                    .unwrap_or(field_index.to_string());
                let is_optional = is_field_optional(field);
                let member_type_id = get_type_identifier(&field.ty)?;
                let is_key = field_attributes.key;
                member_seq.extend(
                    quote! {dust_dds::xtypes::type_object::CompleteStructMember {
                        common: dust_dds::xtypes::type_object::CommonStructMember {
                            member_id: #member_id,
                            member_flags: dust_dds::xtypes::type_object::StructMemberFlag {
                                try_construct:
                                    dust_dds::xtypes::dynamic_type::TryConstructKind::Discard,
                                is_external: false,
                                is_optional: #is_optional,
                                is_must_undestand: true,
                                is_key: #is_key,
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
                    }
            })
        }
        syn::Data::Enum(_data_enum) => Ok(quote! {
            dust_dds::xtypes::type_object::TypeIdentifier::TkNone
        }),
        syn::Data::Union(data_union) => Err(syn::Error::new(
            data_union.union_token.span,
            "Union not supported",
        )),
    }?;

    Ok(quote! {
        impl #impl_generics dust_dds::topic_definition::type_support::TypeSupport for #ident #type_generics #where_clause {
            fn get_type_name() -> &'static str {
                #ident_str
            }

            fn get_type() -> impl dust_dds::xtypes::dynamic_type::DynamicType
            {
                #complete_type_object_quote
            }
        }
    })
}
