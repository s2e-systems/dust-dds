use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, DeriveInput, GenericArgument, PathArguments, Result};

use super::attributes::{get_field_attributes, get_input_extensibility, Extensibility};

pub fn expand_type_support(input: &DeriveInput) -> Result<TokenStream> {
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let ident = &input.ident;

    let dynamic_type_quote = match &input.data {
        syn::Data::Struct(data_struct) => {
            let type_name = ident.to_string();
            let extensibility = get_input_extensibility(input)?;

            let extensibility_kind = match extensibility {
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

            let struct_builder = quote! {
                extern crate alloc;
                let mut builder = dust_dds::xtypes::dynamic_type::DynamicTypeBuilderFactory::create_type(
                    dust_dds::xtypes::dynamic_type::TypeDescriptor {
                        kind: dust_dds::xtypes::dynamic_type::TK_STRUCTURE,
                        name: alloc::string::String::from(#type_name),
                        base_type: None,
                        discriminator_type: None,
                        bound: alloc::vec::Vec::new(),
                        element_type: None,
                        key_element_type: None,
                        extensibility_kind: #extensibility_kind,
                        is_nested: false,
                    });
            };

            let mut member_seq = quote! {};
            for (field_index, field) in data_struct.fields.iter().enumerate() {
                let field_attributes = get_field_attributes(field)?;

                let index = field_index as u32;
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
                let (is_optional, member_type) = match &field.ty {
                    syn::Type::Path(field_type_path)
                        if field_type_path.path.segments[0].ident == "Option" =>
                    {
                        if let PathArguments::AngleBracketed(angle_bracketed) =
                            &field_type_path.path.segments[0].arguments
                        {
                            if let Some(GenericArgument::Type(inner_ty)) =
                                angle_bracketed.args.first()
                            {
                                (true, quote!(#inner_ty))
                            } else {
                                (true, quote!(#field_type_path))
                            }
                        } else {
                            (true, quote!(#field_type_path))
                        }
                    }
                    _ => {
                        let field_type = &field.ty;
                        (false, quote!(#field_type))
                    }
                };
                let is_key = field_attributes.key;
                member_seq.extend(
                    quote! {
                         builder.add_member(dust_dds::xtypes::dynamic_type::MemberDescriptor {
                            name: alloc::string::String::from(#field_name),
                            id: #member_id,
                            r#type: <#member_type as dust_dds::infrastructure::type_support::TypeSupport>::get_type(),
                            default_value: alloc::string::String::new(),
                            index: #index,
                            try_construct_kind: dust_dds::xtypes::dynamic_type::TryConstructKind::UseDefault,
                            label: alloc::vec::Vec::new(),
                            is_key: #is_key,
                            is_optional: #is_optional,
                            is_must_understand: true,
                            is_shared: false,
                            is_default_label: false,
                        })
                        .unwrap();
                    },
                );
            }
            Ok(quote! {
                #struct_builder
                #member_seq
                builder.build()
            })
        }
        syn::Data::Enum(_data_enum) => {
            let type_name = ident.to_string();
            let extensibility = get_input_extensibility(input)?;

            let extensibility_kind = match extensibility {
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
            let enum_builder = quote! {
                extern crate alloc;
                let mut builder = dust_dds::xtypes::dynamic_type::DynamicTypeBuilderFactory::create_type(
                    dust_dds::xtypes::dynamic_type::TypeDescriptor {
                        kind: dust_dds::xtypes::dynamic_type::TK_ENUM,
                        name: alloc::string::String::from(#type_name),
                        base_type: None,
                        discriminator_type: None,
                        bound: alloc::vec::Vec::new(),
                        element_type: None,
                        key_element_type: None,
                        extensibility_kind: #extensibility_kind,
                        is_nested: false,
                    });
            };

            Ok(quote! {
                #enum_builder

                builder.build()
            })
        }
        syn::Data::Union(data_union) => Err(syn::Error::new(
            data_union.union_token.span,
            "Union not supported",
        )),
    }?;

    Ok(quote! {
        impl #impl_generics dust_dds::infrastructure::type_support::TypeSupport for #ident #type_generics #where_clause {
            fn get_type() -> dust_dds::xtypes::dynamic_type::DynamicType
            {
                #dynamic_type_quote
            }

            fn create_sample(src: dust_dds::xtypes::dynamic_type::DynamicData) -> dust_dds::infrastructure::error::DdsResult<Self> {
                todo!()
            }

            fn create_dynamic_sample(self) -> dust_dds::xtypes::dynamic_type::DynamicData {
                todo!()
            }
        }
    })
}
