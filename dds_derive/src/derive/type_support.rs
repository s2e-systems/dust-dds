use crate::derive::{
    attributes::get_field_attributes, enum_support::read_enum_variant_discriminant_mapping,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DataEnum, DeriveInput, Fields, Index, Result, spanned::Spanned};

pub fn expand_type_support(input: &DeriveInput) -> Result<TokenStream> {
    let input_attributes = get_input_attributes(input)?;
    let ident = &input.ident;

    let type_name = input_attributes.name.as_str();
    let extensibility_kind = match input_attributes.extensibility {
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
    let is_nested = input_attributes.is_nested;

    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let (get_type_quote, create_dynamic_sample_quote, create_sample_quote) = match &input.data {
        syn::Data::Struct(data_struct) => {
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
                let member_id = match input_attributes.extensibility {
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
                get_type_quote,
                create_dynamic_sample_quote,
                create_sample_quote,
            ))
        }
        syn::Data::Enum(data_enum) => {
            // Separate between Unions and Enumeration which are both
            // mapped as Rust enum types
            if is_enum_xtypes_union(data_enum) {
                let union_descriptor = quote! {
                    &dust_dds::xtypes::dynamic_type::TypeDescriptor {
                        kind: dust_dds::xtypes::dynamic_type::TypeKind::UNION,
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
                let get_type_quote = quote! {
                    const r#TYPE: &'static dyn dust_dds::xtypes::dynamic_type::DynamicType =
                        &dust_dds::xtypes::dynamic_type::StaticTypeInformation {
                            descriptor: #union_descriptor,
                            member_list: &[]
                        };
                };

                let create_dynamic_sample_quote = quote! {todo!()};
                let create_sample_quote = quote! {
                    todo!()
                };
                Ok((
                    get_type_quote,
                    create_dynamic_sample_quote,
                    create_sample_quote,
                ))
            } else {
                // Note: Mapping has to be done with a match self strategy because the enum might not be copy so casting it using e.g. "self as i64" would
                // be consuming it.
                let discriminator_type =
                    quote! {<i32 as dust_dds::xtypes::binding::XTypesBinding>::TYPE_INFORMATION};
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

struct InputAttributes {
    name: String,
    extensibility: Extensibility,
    is_nested: bool,
}

fn get_input_attributes(input: &DeriveInput) -> Result<InputAttributes> {
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
            }
            else {
                Ok(())
            }
        })?;
    }
    Ok(InputAttributes {
        name,
        extensibility,
        is_nested,
    })
}

pub fn is_enum_xtypes_union(data_enum: &DataEnum) -> bool {
    data_enum
        .variants
        .iter()
        .any(|v| !matches!(&v.fields, Fields::Unit))
}
