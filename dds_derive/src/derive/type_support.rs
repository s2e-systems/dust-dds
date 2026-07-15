use crate::derive::{
    attributes::{
        BitBound, Extensibility, get_enumerated_type_attributes, get_struct_attributes,
        get_structure_member_attributes, get_union_type_attributes, get_union_variant_attributes,
    },
    enum_support::read_enum_variant_discriminant_mapping,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DataEnum, DeriveInput, Fields, Index, Result, parse_quote, spanned::Spanned};

pub fn expand_type_support(input: &DeriveInput) -> Result<TokenStream> {
    let ident = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let (get_type_quote, create_dynamic_sample_quote, create_sample_quote) = match &input.data {
        syn::Data::Struct(xtypes_struct) => {
            // Get the type declaration attributes as defined in Table 21 – IDL Built-in Annotations Usage of the XTypes standard
            let r#struct = get_struct_attributes(input)?;
            let type_name = r#struct.name.as_str();
            let extensibility_kind = match r#struct.extensibility {
                Extensibility::Final => {
                    quote! {dust_dds::xtypes::dynamic_type::ExtensibilityKind::Final}
                }
                Extensibility::Appendable => {
                    quote! {dust_dds::xtypes::dynamic_type::ExtensibilityKind::Appendable}
                }
                Extensibility::Mutable => {
                    quote! {dust_dds::xtypes::dynamic_type::ExtensibilityKind::Mutable}
                }
            };
            let is_nested = r#struct.is_nested;
            let base_type = match r#struct.base_type {
                Some(t) => quote! {Some(<#t as dust_dds::xtypes::type_support::Type>::TYPE)},
                None => quote! {None},
            };

            let struct_descriptor = quote! {
                &dust_dds::xtypes::dynamic_type::TypeDescriptor {
                    kind: dust_dds::xtypes::dynamic_type::TypeKind::STRUCTURE,
                    name: #type_name,
                    base_type: #base_type,
                    discriminator_type: None,
                    bound: None,
                    element_type: None,
                    key_element_type: None,
                    extensibility_kind: #extensibility_kind,
                    is_nested: #is_nested,
                }
            };

            let mut member_list = Vec::new();
            let mut member_sample_seq = Vec::new();
            let mut member_dynamic_sample_seq = Vec::new();

            let mut next_auto_id = 0;
            for (member_index, member) in xtypes_struct.fields.iter().enumerate() {
                let index = member_index as u32;
                let struct_member_attributes = get_structure_member_attributes(member)?;

                let member_name = member
                    .ident
                    .as_ref()
                    .map(|i| i.to_string())
                    .unwrap_or(member_index.to_string());

                let member_id = if struct_member_attributes.hashid {
                    let member_hash = <[u8; 16]>::from(md5::compute(member_name.as_bytes()));
                    let member_hash_int = u32::from_le_bytes([
                        member_hash[0],
                        member_hash[1],
                        member_hash[2],
                        member_hash[3],
                    ]);
                    syn::parse_str(&member_hash_int.to_string())?
                } else {
                    match r#struct.extensibility {
                        Extensibility::Final | Extensibility::Appendable => {
                            syn::parse_str(&member_index.to_string())
                        }
                        Extensibility::Mutable => {
                            if let Some(provided_id) = struct_member_attributes.id {
                                Ok(provided_id)
                            } else {
                                syn::parse_str(&next_auto_id.to_string())
                            }
                        }
                    }?
                };

                if !struct_member_attributes.hashid {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Int(lit_int),
                        ..
                    }) = &member_id
                    {
                        next_auto_id = lit_int.base10_parse::<u32>()? + 1;
                    }
                }

                let member_type = &member.ty;
                let is_key = struct_member_attributes.key;
                let is_optional = struct_member_attributes.optional;
                let is_external = struct_member_attributes.external;
                let default_value = struct_member_attributes.default_value.map(|x| quote! {#x});

                let member_dynamic_type = if is_external {
                    todo!("Check that the type of this member is Box<T>, otherwise return error message saying that Box<T> is expected for external");
                    todo!("Retrieve the type T from the inside of the Box");
                    quote! {
                        dust_dds::xtypes::dynamic_type::DynamicType {
                            descriptor: todo!("Use the descriptor of type T to fill up this information"),
                            member_list: &[]
                        }
                    }
                } else {
                    quote! { <#member_type as dust_dds::xtypes::type_support::Type>::TYPE}
                };

                member_list.push(
                    quote! {
                         dust_dds::xtypes::dynamic_type::DynamicTypeMember {
                            descriptor: dust_dds::xtypes::dynamic_type::MemberDescriptor {
                                name: #member_name,
                                id: #member_id,
                                r#type: #member_dynamic_type,
                                default_value: None,
                                index: #index as u32,
                                try_construct_kind: dust_dds::xtypes::dynamic_type::TryConstructKind::Discard,
                                label: &[],
                                is_key: #is_key,
                                is_optional: #is_optional,
                                is_must_understand: #is_key,
                                is_shared: false,
                                is_default_label: false,
                                is_external: #is_external,
                            }
                        }
                    }
                );

                let member_type = &member.ty;
                let member_default_value = default_value
                    .unwrap_or(quote! { <#member_type as ::core::default::Default>::default()});
                if struct_member_attributes.non_serialized {
                    match &member.ident {
                        Some(member_ident) => {
                            member_sample_seq.push(quote! {
                                #member_ident: #member_default_value,
                            });
                        }
                        None => {
                            member_sample_seq.push(quote! {
                                #member_default_value,
                            });
                        }
                    }
                } else {
                    match &member.ident {
                        Some(member_ident) => {
                            if is_optional {
                                member_sample_seq.push(quote! {
                                    #member_ident: src.remove_value(#member_id).map_or(#member_default_value, |x| {
                                        dust_dds::xtypes::data_storage::DataStorageMapping::try_from_storage(x).expect("Must match")
                                    }),
                                });

                                member_dynamic_sample_seq
                                    .push(quote! {
                                        if self.#member_ident != #member_default_value {
                                            data.set_value(#member_id, dust_dds::xtypes::data_storage::DataStorageMapping::into_storage(self.#member_ident));
                                        }
                                    });
                            } else {
                                member_sample_seq.push(quote! {
                                    #member_ident: dust_dds::xtypes::data_storage::DataStorageMapping::try_from_storage(src.remove_value(#member_id).expect("Must exist")).expect("Must match"),
                                });
                                member_dynamic_sample_seq
                                    .push(quote! {data.set_value(#member_id, dust_dds::xtypes::data_storage::DataStorageMapping::into_storage(self.#member_ident));});
                            }
                        }
                        None => {
                            let index = Index::from(member_index);
                            // In Mutable structs every member is optional even when not explicitly marked as such
                            if r#struct.extensibility == Extensibility::Mutable || is_optional {
                                member_sample_seq.push(quote! {
                                    src.remove_value(#member_id).map_or(#member_default_value, |x| {
                                        DataStorageMapping::try_from_storage(x).expect("Must match")
                                    }),
                                });
                                member_dynamic_sample_seq.push(quote! {
                                    if self.#index != #member_default_value {
                                        data.set_value(#member_id, dust_dds::xtypes::data_storage::DataStorageMapping::into_storage(self.#index));
                                    }
                                })
                            } else {
                                member_sample_seq.push(quote! { dust_dds::xtypes::data_storage::DataStorageMapping::try_from_storage(src.remove_value(#member_id).expect("Must exist")).expect("Must match"),});
                                member_dynamic_sample_seq.push(quote! {
                                    data.set_value(#member_id, dust_dds::xtypes::data_storage::DataStorageMapping::into_storage(self.#index));
                                })
                            }
                        }
                    }
                }
            }
            let is_tuple = match xtypes_struct.fields.iter().next() {
                Some(s) => s.ident.is_none(),
                None => false,
            };

            let get_type_quote = quote! {
                const TYPE: dust_dds::xtypes::dynamic_type::DynamicType<'static> =
                    dust_dds::xtypes::dynamic_type::DynamicType {
                        descriptor: #struct_descriptor,
                        member_list: &[#(#member_list,)*]
                    };
            };

            let create_dynamic_sample_quote = quote! {
                #(#member_dynamic_sample_seq)*
            };
            let create_sample_quote = if is_tuple {
                quote! {Self(#(#member_sample_seq)*)}
            } else {
                quote! {Self{#(#member_sample_seq)*}}
            };
            Ok((
                get_type_quote,
                create_dynamic_sample_quote,
                create_sample_quote,
            ))
        }
        // Separate between Unions and Enumeration which are both
        // mapped as Rust enum types
        syn::Data::Enum(xtypes_union) if is_enum_xtypes_union(xtypes_union) => {
            let union_attributes = get_union_type_attributes(input)?;
            let discriminator_type = union_attributes.discriminator_type;
            let type_name = union_attributes.name.as_str();
            let extensibility_kind = match union_attributes.extensibility {
                Extensibility::Final => {
                    quote! {dust_dds::xtypes::dynamic_type::ExtensibilityKind::Final}
                }
                Extensibility::Appendable => {
                    quote! {dust_dds::xtypes::dynamic_type::ExtensibilityKind::Appendable}
                }
                Extensibility::Mutable => {
                    quote! {dust_dds::xtypes::dynamic_type::ExtensibilityKind::Mutable}
                }
            };
            let is_nested = union_attributes.is_nested;
            if xtypes_union.variants.len() > (u32::MAX as usize + 1) {
                return Err(syn::Error::new(
                    input.span(),
                    "Union can hold at most `u32::MAX + 1` variants",
                ));
            }

            let union_descriptor = quote! {
                &dust_dds::xtypes::dynamic_type::TypeDescriptor {
                    kind: dust_dds::xtypes::dynamic_type::TypeKind::UNION,
                    name: #type_name,
                    base_type: None,
                    discriminator_type: ::core::option::Option::Some(<#discriminator_type as ::dust_dds::xtypes::type_support::Type>::TYPE),
                    bound: None,
                    element_type: None,
                    key_element_type: None,
                    extensibility_kind: #extensibility_kind,
                    is_nested: #is_nested,
                }
            };

            let is_key = union_attributes.is_discriminator_key;
            let mut variant_list: Vec<TokenStream> = vec![quote! {
                 dust_dds::xtypes::dynamic_type::DynamicTypeMember {
                    descriptor: dust_dds::xtypes::dynamic_type::MemberDescriptor {
                        name: "disc",
                        id: 0u32,
                        r#type: <#discriminator_type as dust_dds::xtypes::type_support::Type>::TYPE,
                        default_value: None,
                        index: 0u32,
                        try_construct_kind: dust_dds::xtypes::dynamic_type::TryConstructKind::Discard,
                        label: &[],
                        is_key: #is_key,
                        is_optional: false,
                        is_must_understand: true,
                        is_shared: false,
                        is_default_label: false,
                        is_external: false,
                    }
                }
            }];
            let mut has_default = false;
            let mut variant_sample_seq = Vec::new();
            let mut variant_dynamic_sample_seq = Vec::new();

            for (variant_index, variant) in xtypes_union.variants.iter().enumerate() {
                let variant_attributes = get_union_variant_attributes(variant)?;
                let variant_index_unsuffixed = syn::Index::from(variant_index + 1);
                let is_default_label = variant_attributes.is_default;

                if !has_default && variant_attributes.is_default {
                    has_default = true;
                }
                let case_list = if variant_attributes.case.is_empty() {
                    vec![parse_quote!(#variant_index_unsuffixed)]
                } else {
                    variant_attributes.case
                };
                let first_discriminator = case_list[0].clone();
                let variant_ident = &variant.ident;
                let variant_name = variant_ident.to_string();

                match &variant.fields {
                    // If there is a single field we handle this as the single type wrapper which is the most common case
                    Fields::Named(fields_named) if fields_named.named.len() == 1 => {
                        let variant_field_name =
                            fields_named.named[0].ident.as_ref().ok_or(syn::Error::new(
                                fields_named.span(),
                                "Field of named variant must have defined name",
                            ))?;
                        let variant_ty = &fields_named.named[0].ty;

                        variant_list.push(quote!{ dust_dds::xtypes::dynamic_type::DynamicTypeMember {
                            descriptor: dust_dds::xtypes::dynamic_type::MemberDescriptor {
                                name: #variant_name,
                                id: #variant_index_unsuffixed as u32,
                                r#type: <#variant_ty as dust_dds::xtypes::type_support::Type>::TYPE,
                                default_value: None,
                                index: #variant_index_unsuffixed as u32,
                                try_construct_kind: dust_dds::xtypes::dynamic_type::TryConstructKind::Discard,
                                label: &[#(#case_list as i32,)*],
                                is_key: false,
                                is_optional: false,
                                is_must_understand: false,
                                is_shared: false,
                                is_default_label: #is_default_label,
                                is_external: false,
                            }
                        }
                        });
                        let variant_sample = quote! {
                            Self::#variant_ident {#variant_field_name: <#variant_ty as ::dust_dds::xtypes::data_storage::DataStorageMapping>::try_from_storage(
                              src.remove_value(#variant_index_unsuffixed as u32).expect("Must exist"),
                            ).expect("Must match")},
                        };

                        variant_sample_seq.push(if variant_attributes.is_default {
                            quote! {_ => #variant_sample}
                        } else {
                            quote! {#first_discriminator => #variant_sample}
                        });
                        variant_dynamic_sample_seq
                            .push(quote! {Self::#variant_ident {#variant_field_name} => {
                                data.set_value(0, <#discriminator_type as ::dust_dds::xtypes::data_storage::DataStorageMapping>::into_storage(#first_discriminator));
                                data.set_value(#variant_index_unsuffixed as u32, ::dust_dds::xtypes::data_storage::DataStorageMapping::into_storage(#variant_field_name));
                            }});
                    }
                    Fields::Unnamed(fields_unnamed) if fields_unnamed.unnamed.len() == 1 => {
                        let variant_ty = &fields_unnamed.unnamed[0].ty;

                        variant_list.push(quote!{ dust_dds::xtypes::dynamic_type::DynamicTypeMember {
                            descriptor: dust_dds::xtypes::dynamic_type::MemberDescriptor {
                                name: #variant_name,
                                id: #variant_index_unsuffixed as u32,
                                r#type: <#variant_ty as dust_dds::xtypes::type_support::Type>::TYPE,
                                default_value: None,
                                index: #variant_index_unsuffixed as u32,
                                try_construct_kind: dust_dds::xtypes::dynamic_type::TryConstructKind::Discard,
                                label: &[#(#case_list as i32,)*],
                                is_key: false,
                                is_optional: false,
                                is_must_understand: false,
                                is_shared: false,
                                is_default_label: #is_default_label,
                                is_external: false,
                            }
                        }
                        });
                        let variant_sample = quote! {
                            Self::#variant_ident(<#variant_ty as ::dust_dds::xtypes::data_storage::DataStorageMapping>::try_from_storage(
                              src.remove_value(#variant_index_unsuffixed as u32).expect("Must exist"),
                            ).expect("Must match")),
                        };

                        variant_sample_seq.push(if variant_attributes.is_default {
                            quote! {_ => #variant_sample}
                        } else {
                            quote! {#first_discriminator => #variant_sample}
                        });
                        variant_dynamic_sample_seq
                            .push(quote! {Self::#variant_ident (a) => {
                                data.set_value(0, <#discriminator_type as ::dust_dds::xtypes::data_storage::DataStorageMapping>::into_storage(#first_discriminator));
                                data.set_value(#variant_index_unsuffixed as u32, ::dust_dds::xtypes::data_storage::DataStorageMapping::into_storage(a));
                            }});
                    }
                    Fields::Unit => {
                        variant_list.push(quote!{ dust_dds::xtypes::dynamic_type::DynamicTypeMember {
                            descriptor: dust_dds::xtypes::dynamic_type::MemberDescriptor {
                                name: #variant_name,
                                id: #variant_index_unsuffixed as u32,
                                r#type: dust_dds::xtypes::dynamic_type::DynamicType {
                                    descriptor: &dust_dds::xtypes::dynamic_type::TypeDescriptor {
                                        kind: dust_dds::xtypes::dynamic_type::TypeKind::NONE,
                                        name: "",
                                        base_type: None,
                                        discriminator_type: None,
                                        bound: None,
                                        element_type: None,
                                        key_element_type: None,
                                        extensibility_kind: dust_dds::xtypes::dynamic_type::ExtensibilityKind::Final,
                                        is_nested: false,
                                    },
                                    member_list: &[],
                                },
                                default_value: None,
                                index: #variant_index_unsuffixed as u32,
                                try_construct_kind: dust_dds::xtypes::dynamic_type::TryConstructKind::Discard,
                                label: &[#(#case_list as i32,)*],
                                is_key: false,
                                is_optional: false,
                                is_must_understand: false,
                                is_shared: false,
                                is_default_label: #is_default_label,
                                is_external: false,
                            }
                        }
                        });
                        let variant_sample = quote! {
                            Self::#variant_ident,
                        };

                        variant_sample_seq.push(if variant_attributes.is_default {
                            quote! {_ => #variant_sample}
                        } else {
                            quote! {#first_discriminator => #variant_sample}
                        });
                        variant_dynamic_sample_seq.push(quote! {Self::#variant_ident => {
                            data.set_value(0, <#discriminator_type as ::dust_dds::xtypes::data_storage::DataStorageMapping>::into_storage(#first_discriminator));
                        },});
                    }
                    Fields::Named(_) | Fields::Unnamed(_) => {
                        return Err(syn::Error::new(
                            variant.span(),
                            "Only variants with a single field are supported",
                        ));
                    }
                }
            }

            if !has_default {
                variant_sample_seq.push(quote! {_ => panic!("Invalid discriminator"),});
            }

            let get_type_quote = quote! {
                const TYPE: dust_dds::xtypes::dynamic_type::DynamicType<'static> =
                    dust_dds::xtypes::dynamic_type::DynamicType {
                        descriptor: #union_descriptor,
                        member_list: &[#(#variant_list,)*]
                    };
            };

            let create_dynamic_sample_quote = quote! {
                match self {
                    #(#variant_dynamic_sample_seq)*
                }
            };

            let create_sample_quote = quote! {
                let disc =
                    <#discriminator_type as ::dust_dds::xtypes::data_storage::DataStorageMapping>::try_from_storage(
                        src.remove_value(0).expect("Must exist"),
                    )
                    .expect("Must match");
                match disc {
                    #(#variant_sample_seq)*
                }
            };
            Ok((
                get_type_quote,
                create_dynamic_sample_quote,
                create_sample_quote,
            ))
        }
        syn::Data::Enum(xtypes_enum) => {
            let enum_type_attributes = get_enumerated_type_attributes(input)?;
            let type_name = enum_type_attributes.name;
            let is_nested = enum_type_attributes.is_nested;
            // Note: Mapping has to be done with a match self strategy because the enum might not be copy so casting it using e.g. "self as i64" would
            // be consuming it.
            let discriminator_type = match enum_type_attributes.bit_bound {
                BitBound::I8 => {
                    quote! {<i8 as dust_dds::xtypes::type_support::Type>::TYPE}
                }
                BitBound::I16 => {
                    quote! {<i16 as dust_dds::xtypes::type_support::Type>::TYPE}
                }
                BitBound::I32 => {
                    quote! {<i32 as dust_dds::xtypes::type_support::Type>::TYPE}
                }
            };

            let discriminator_dynamic_value = match enum_type_attributes.bit_bound {
                BitBound::I8 => quote! {data.set_int8_value(0, self as i8).unwrap();},
                BitBound::I16 => quote! {data.set_int16_value(0, self as i16).unwrap();},
                BitBound::I32 => quote! {data.set_int32_value(0, self as i32).unwrap();},
            };

            let discriminator_sample = match enum_type_attributes.bit_bound {
                BitBound::I8 => quote! {src.get_int8_value(0).expect("Must exist");},
                BitBound::I16 => quote! {src.get_int16_value(0).expect("Must exist");},
                BitBound::I32 => quote! {src.get_int32_value(0).expect("Must exist");},
            };

            let enum_descriptor = quote! {
                &dust_dds::xtypes::dynamic_type::TypeDescriptor {
                    kind: dust_dds::xtypes::dynamic_type::TypeKind::ENUM,
                    name: #type_name,
                    base_type: None,
                    discriminator_type: Some(#discriminator_type),
                    bound: None,
                    element_type: None,
                    key_element_type: None,
                    extensibility_kind: dust_dds::xtypes::dynamic_type::ExtensibilityKind::Final,
                    is_nested: #is_nested,
                }
            };
            let get_type_quote = quote! {
                const TYPE: dust_dds::xtypes::dynamic_type::DynamicType<'static> =
                    dust_dds::xtypes::dynamic_type::DynamicType {
                        descriptor: #enum_descriptor,
                        member_list: &[]
                    };
            };

            let create_dynamic_sample_quote = quote! {
                #discriminator_dynamic_value
            };
            let enum_variant_mapping = read_enum_variant_discriminant_mapping(xtypes_enum);
            let mut create_sample_quote_variants = Vec::new();
            for (variant_ident, variant_discriminant) in enum_variant_mapping {
                let d = Index::from(variant_discriminant);
                create_sample_quote_variants.push(quote! {#d => Self::#variant_ident,});
            }
            let create_sample_quote = quote! {
                    let discriminator = #discriminator_sample;
                    match discriminator {
                        #(#create_sample_quote_variants)*
                        d => panic!("Invalid discriminator {d:?}"),
                    }
            };

            Ok((
                get_type_quote,
                create_dynamic_sample_quote,
                create_sample_quote,
            ))
        }
        syn::Data::Union(data_union) => Err(syn::Error::new(
            data_union.union_token.span,
            "Rust union not supported in Dust DDS. For IDL union mapping use enum with types in variants.",
        )),
    }?;

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics dust_dds::xtypes::type_support::TypeSupport for #ident #type_generics #where_clause {
            fn create_sample(src: &mut dust_dds::xtypes::dynamic_type::DynamicData) -> Self {
                #create_sample_quote
            }

            fn create_dynamic_sample(self) -> dust_dds::xtypes::dynamic_type::DynamicData<'static> {
                let mut data = dust_dds::xtypes::dynamic_type::DynamicDataFactory::create_data(Self::get_type());
                #create_dynamic_sample_quote
                data
            }
        }

        #[automatically_derived]
        impl #impl_generics dust_dds::xtypes::type_support::Type for #ident #type_generics #where_clause {
            #get_type_quote
        }
    })
}

pub fn is_enum_xtypes_union(data_enum: &DataEnum) -> bool {
    data_enum
        .variants
        .iter()
        .any(|v| !matches!(&v.fields, Fields::Unit))
}
