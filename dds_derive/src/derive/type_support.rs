use crate::derive::{
    attributes::{get_field_attributes, get_variant_attributes},
    enum_support::read_enum_variant_discriminant_mapping,
};
use proc_macro2::TokenStream;
use quote::quote;
use std::borrow::Cow;
use syn::{DataEnum, DeriveInput, Fields, Index, Result, spanned::Spanned};

pub fn expand_type_support(input: &DeriveInput) -> Result<TokenStream> {
    let ident = &input.ident;

    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    let (type_name, get_type_quote, create_dynamic_sample_quote, create_sample_quote) = match &input
        .data
    {
        syn::Data::Struct(data_struct) => {
            let struct_attributes = get_struct_or_enum_attributes(input)?;

            let type_name = struct_attributes.name;
            let extensibility_kind = match struct_attributes.extensibility {
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
            let is_nested = struct_attributes.is_nested;

            let struct_descriptor = quote! {
                &dust_dds::xtypes::dynamic_type::TypeDescriptor {
                    kind: dust_dds::xtypes::dynamic_type::TypeKind::STRUCTURE,
                    name: #type_name,
                    base_type: None,
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

            for (field_index, field) in data_struct.fields.iter().enumerate() {
                let field_attributes = get_field_attributes(field)?;

                let index = field_index as u32;
                let member_id = match struct_attributes.extensibility {
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

                let member_type = &field.ty;
                let is_key = field_attributes.key;
                let is_optional = field_attributes.optional;
                let default_value = field_attributes
                    .default_value
                    .map(|x| quote! {#x})
                    .unwrap_or(quote! {Default::default()});

                member_list.push(
                    quote! {
                         dust_dds::xtypes::dynamic_type::DynamicTypeMember {
                            descriptor: dust_dds::xtypes::dynamic_type::MemberDescriptor {
                                name: #field_name,
                                id: #member_id,
                                r#type: <#member_type as dust_dds::xtypes::binding::XTypesBinding>::TYPE_INFORMATION,
                                default_value: None,
                                index: #index,
                                try_construct_kind: dust_dds::xtypes::dynamic_type::TryConstructKind::UseDefault,
                                label: None,
                                is_key: #is_key,
                                is_optional: #is_optional,
                                is_must_understand: true,
                                is_shared: false,
                                is_default_label: false,
                            }
                        }
                    }
                );

                if !field_attributes.non_serialized {
                    match &field.ident {
                        Some(field_ident) => {
                            if is_optional {
                                member_sample_seq.push(quote! {
                                    #field_ident: src.remove_value(#member_id).map_or(#default_value, |x| {
                                        dust_dds::xtypes::data_storage::DataStorageMapping::try_from_storage(x).expect("Must match")
                                    }),
                                });
                                member_dynamic_sample_seq
                                    .push(quote! {
                                        if self.#field_ident != #default_value {
                                            data.set_value(#member_id, dust_dds::xtypes::data_storage::DataStorageMapping::into_storage(self.#field_ident));
                                        }
                                    });
                            } else {
                                member_sample_seq.push(quote! {
                                    #field_ident: dust_dds::xtypes::data_storage::DataStorageMapping::try_from_storage(src.remove_value(#member_id).expect("Must exist")).expect("Must match"),
                                });
                                member_dynamic_sample_seq
                                    .push(quote! {data.set_value(#member_id, dust_dds::xtypes::data_storage::DataStorageMapping::into_storage(self.#field_ident));});
                            }
                        }
                        None => {
                            let index = Index::from(field_index);
                            if is_optional {
                                member_sample_seq.push(quote! {
                                    src.remove_value(#member_id).map_or(#default_value, |x| {
                                        DataStorageMapping::try_from_storage(x).expect("Must match")
                                    }),
                                });
                                member_dynamic_sample_seq.push(quote! {
                                    if self.#index != #default_value {
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
            let is_tuple = data_struct
                .fields
                .iter()
                .next()
                .expect("Not empty")
                .ident
                .is_none();

            let get_type_quote = quote! {
                const r#TYPE: &'static dyn dust_dds::xtypes::dynamic_type::DynamicType =
                    &dust_dds::xtypes::dynamic_type::StaticTypeInformation {
                        descriptor: #struct_descriptor,
                        member_list: &[#(#member_list,)*]
                    };
            };

            let create_dynamic_sample_quote = quote! {
                let mut data = dust_dds::xtypes::dynamic_type::DynamicDataFactory::create_data();
                #(#member_dynamic_sample_seq)*
                data
            };
            let create_sample_quote = if is_tuple {
                quote! {Self(#(#member_sample_seq)*)}
            } else {
                quote! {Self{#(#member_sample_seq)*}}
            };
            Ok((
                type_name,
                get_type_quote,
                create_dynamic_sample_quote,
                create_sample_quote,
            ))
        }
        syn::Data::Enum(data_enum) => {
            // Separate between Unions and Enumeration which are both
            // mapped as Rust enum types
            if is_enum_xtypes_union(data_enum) {
                if data_enum.variants.len() > (u32::MAX as usize + 1) {
                    return Err(syn::Error::new(
                        input.span(),
                        "Enum can hold at most `u32::MAX + 1` variants",
                    ));
                }

                let union_attributes = get_union_attributes(input)?;

                let union_name = union_attributes.name;
                let discriminator_type = &union_attributes.discriminator;

                let union_descriptor = quote! {
                    &::dust_dds::xtypes::dynamic_type::TypeDescriptor {
                        kind: ::dust_dds::xtypes::dynamic_type::TypeKind::UNION,
                        name: #union_name,
                        base_type: ::core::option::Option::None,
                        discriminator_type: ::core::option::Option::Some(<#discriminator_type as ::dust_dds::xtypes::binding::XTypesBinding>::TYPE_INFORMATION),
                        bound: ::core::option::Option::None,
                        element_type: ::core::option::Option::None,
                        key_element_type: ::core::option::Option::None,
                        extensibility_kind: ::dust_dds::xtypes::dynamic_type::ExtensibilityKind::Final,
                        is_nested: false,
                    }
                };

                let mut variant_list = Vec::new();
                let mut variant_sample_seq = Vec::new();
                let mut variant_dynamic_sample_seq = Vec::new();

                for (variant_index, variant) in data_enum.variants.iter().enumerate() {
                    let variant_attributes = get_variant_attributes(variant)?;

                    let index = variant_index as u32;
                    let variant_discriminators = variant_attributes.discriminators;
                    let variant_discriminator = &variant_discriminators[0];
                    let variant_ident = &variant.ident;
                    let variant_name = variant_ident.to_string();
                    let variant_ty = match &variant.fields {
                        Fields::Named(_) => {
                            return Err(syn::Error::new(
                                variant.span(),
                                "Enum variant must be unnamed or unit",
                            ));
                        }
                        Fields::Unnamed(fields) => {
                            if fields.unnamed.len() != 1 {
                                return Err(syn::Error::new(
                                    variant.span(),
                                    "Enum variant must contain a single field",
                                ));
                            }

                            let variant_ty = &fields.unnamed[0].ty;

                            variant_sample_seq.push(quote! {
                                #(#variant_discriminators)|* => Self::#variant_ident(<#variant_ty as ::dust_dds::xtypes::data_storage::DataStorageMapping>::try_from_storage(src.remove_value(1).expect("Must exist")).expect("Must match"))
                            });

                            variant_dynamic_sample_seq.push(quote! {
                                Self::#variant_ident(variant) => {
                                    data.set_value(0u32, <#discriminator_type as ::dust_dds::xtypes::data_storage::DataStorageMapping>::into_storage(#variant_discriminator));
                                    data.set_value(1u32, <#variant_ty as ::dust_dds::xtypes::data_storage::DataStorageMapping>::into_storage(variant));
                                }
                            });

                            Cow::Borrowed(variant_ty)
                        }
                        Fields::Unit => {
                            variant_sample_seq.push(quote! {
                                #(#variant_discriminators)|* => Self::#variant_ident
                            });

                            variant_dynamic_sample_seq.push(quote! {
                                Self::#variant_ident => {
                                    data.set_value(0u32, <#discriminator_type as ::dust_dds::xtypes::data_storage::DataStorageMapping>::into_storage(#variant_discriminator));
                                }
                            });

                            Cow::Owned(syn::parse_quote!(()))
                        }
                    };

                    for variant_discriminator in variant_discriminators {
                        variant_list.push(
                            quote! {
                                ::dust_dds::xtypes::dynamic_type::DynamicTypeMember {
                                    descriptor: ::dust_dds::xtypes::dynamic_type::MemberDescriptor {
                                        name: #variant_name,
                                        id: const {
                                            let id = (#variant_discriminator) as i64;
                                            ::core::assert!(id >= 0, ::core::concat!("discriminator `", ::core::stringify!(#variant_discriminator), "` of `", ::core::stringify!(#ident), "::", ::core::stringify!(#variant_ident),"` must evaluate to a value `>= 0`"));
                                            ::core::assert!(id <= (u32::MAX as i64), ::core::concat!("discriminator `", ::core::stringify!(#variant_discriminator), "` of `", ::core::stringify!(#ident), "::", ::core::stringify!(#variant_ident),"` must evaluate to a value `<= u32::MAX`"));
                                            id as u32
                                        },
                                        r#type: <#variant_ty as ::dust_dds::xtypes::binding::XTypesBinding>::TYPE_INFORMATION,
                                        default_value: ::core::option::Option::None,
                                        index: #index,
                                        try_construct_kind: ::dust_dds::xtypes::dynamic_type::TryConstructKind::UseDefault,
                                        label: ::core::option::Option::None,
                                        is_key: false,
                                        is_optional: false,
                                        is_must_understand: true,
                                        is_shared: false,
                                        is_default_label: false,
                                    }
                                }
                            },
                        );
                    }
                }

                let get_type_quote = quote! {
                    const r#TYPE: &'static dyn ::dust_dds::xtypes::dynamic_type::DynamicType =
                        &::dust_dds::xtypes::dynamic_type::StaticTypeInformation {
                            descriptor: #union_descriptor,
                            member_list: &[#(#variant_list,)*]
                        };
                };

                let create_dynamic_sample_quote = quote! {
                    let mut data = ::dust_dds::xtypes::dynamic_type::DynamicDataFactory::create_data();

                    match self {
                        #(#variant_dynamic_sample_seq,)*
                    }

                    data
                };

                let create_sample_quote = quote! {
                    let discriminator = <#discriminator_type as ::dust_dds::xtypes::data_storage::DataStorageMapping>::try_from_storage(
                            src.remove_value(0).expect("Must exist")
                        )
                        .expect("Must match");

                    match discriminator {
                        #(#variant_sample_seq,)*
                        _ => panic!("invalid discriminator"),
                    }
                };

                Ok((
                    union_name,
                    get_type_quote,
                    create_dynamic_sample_quote,
                    create_sample_quote,
                ))
            } else {
                let enum_attributes = get_struct_or_enum_attributes(input)?;

                let type_name = enum_attributes.name;
                let extensibility_kind = match enum_attributes.extensibility {
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
                let is_nested = enum_attributes.is_nested;
                let discriminator_type =
                    quote! {<i32 as ::dust_dds::xtypes::binding::XTypesBinding>::TYPE_INFORMATION};

                // Note: Mapping has to be done with a match self strategy because the enum might not be copy so casting it using e.g. "self as i64" would
                // be consuming it.
                let discriminator_dynamic_value =
                    quote! {data.set_int32_value(0, self as i32).unwrap();};

                let enum_descriptor = quote! {
                    &dust_dds::xtypes::dynamic_type::TypeDescriptor {
                        kind: dust_dds::xtypes::dynamic_type::TypeKind::ENUM,
                        name: #type_name,
                        base_type: None,
                        discriminator_type: Some(#discriminator_type),
                        bound: None,
                        element_type: None,
                        key_element_type: None,
                        extensibility_kind: #extensibility_kind,
                        is_nested: #is_nested,
                    }
                };
                let get_type_quote = quote! {
                    const r#TYPE: &'static dyn dust_dds::xtypes::dynamic_type::DynamicType =
                        &dust_dds::xtypes::dynamic_type::StaticTypeInformation {
                            descriptor: #enum_descriptor,
                            member_list: &[]
                        };
                };

                let create_dynamic_sample_quote = quote! {
                    let mut data = dust_dds::xtypes::dynamic_type::DynamicDataFactory::create_data();
                    #discriminator_dynamic_value
                    data
                };
                let enum_variant_mapping = read_enum_variant_discriminant_mapping(data_enum);
                let mut create_sample_quote_variants = Vec::new();
                for (variant_ident, variant_discriminant) in enum_variant_mapping {
                    let d = Index::from(variant_discriminant);
                    create_sample_quote_variants.push(quote! {#d => Self::#variant_ident,});
                }
                let create_sample_quote = quote! {
                        let discriminator = src.get_int32_value(0).expect("Must exist");
                        match discriminator {
                            #(#create_sample_quote_variants)*
                            d => panic!("Invalid discriminator {d:?}"),
                        }
                };

                Ok((
                    type_name,
                    get_type_quote,
                    create_dynamic_sample_quote,
                    create_sample_quote,
                ))
            }
        }
        syn::Data::Union(data_union) => Err(syn::Error::new(
            data_union.union_token.span,
            "Union not supported",
        )),
    }?;

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics dust_dds::infrastructure::type_support::TypeSupport for #ident #type_generics #where_clause {
            const TYPE_NAME: &'static str = #type_name;

            #get_type_quote

            fn create_sample(mut src: dust_dds::xtypes::dynamic_type::DynamicData) -> Self {
                #create_sample_quote
            }

            fn create_dynamic_sample(self) -> dust_dds::xtypes::dynamic_type::DynamicData {
                #create_dynamic_sample_quote
            }
        }
    })
}

enum Extensibility {
    Final,
    Appendable,
    Mutable,
}

struct StructOrEnumAttributes {
    name: String,
    extensibility: Extensibility,
    is_nested: bool,
}

fn get_struct_or_enum_attributes(input: &DeriveInput) -> Result<StructOrEnumAttributes> {
    let mut name = input.ident.to_string();
    let mut extensibility = Extensibility::Final;
    let mut is_nested = false;
    if let Some(xtypes_attribute) = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("dust_dds"))
    {
        xtypes_attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                name = meta.value()?.parse::<syn::LitStr>()?.value();
                Ok(())
            } else if meta.path.is_ident("extensibility") {
                let format_str: syn::LitStr = meta.value()?.parse()?;
                match format_str.value().as_ref() {
                    "final" => {
                        extensibility = Extensibility::Final;
                        Ok(())
                    }
                    "appendable" => {
                        extensibility = Extensibility::Appendable;
                        Ok(())
                    }
                    "mutable" => {
                        extensibility = Extensibility::Mutable;
                        Ok(())
                    }
                    _ => Err(syn::Error::new(
                        meta.path.span(),
                        r#"Invalid format specified. Valid options are "final", "appendable", "mutable". "#,
                    )),
                }
            } else if meta.path.is_ident("nested") {
                is_nested = true;
                Ok(())
            } else {
               Err(syn::Error::new(
                    meta.path.span(),
                    format!("unknown attribute `{}`", meta.path.require_ident()?),
                ))
            }
        })?;
    }

    Ok(StructOrEnumAttributes {
        name,
        extensibility,
        is_nested,
    })
}

struct UnionAttributes {
    name: String,
    discriminator: syn::Type,
}

fn get_union_attributes(input: &DeriveInput) -> Result<UnionAttributes> {
    let mut name = input.ident.to_string();
    let mut discriminator = None;

    if let Some(xtypes_attribute) = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("dust_dds"))
    {
        xtypes_attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                name = meta.value()?.parse::<syn::LitStr>()?.value();
                Ok(())
            } else if meta.path.is_ident("discriminator") {
                discriminator = Some(meta.value()?.parse()?);
                Ok(())
            } else {
                Err(syn::Error::new(
                    meta.path.span(),
                    format!("unknown attribute `{}`", meta.path.require_ident()?),
                ))
            }
        })?;
    }

    let Some(discriminator) = discriminator else {
        return Err(syn::Error::new(input.span(), "`discriminator` is required"));
    };

    Ok(UnionAttributes {
        name,
        discriminator,
    })
}

pub fn is_enum_xtypes_union(data_enum: &DataEnum) -> bool {
    data_enum
        .variants
        .iter()
        .any(|v| !matches!(&v.fields, Fields::Unit))
}
