use crate::derive::attributes::get_field_attributes;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    spanned::Spanned, DataEnum, DeriveInput, Expr, ExprLit, Fields, GenericArgument, Ident, Index,
    Lit, PathArguments, Result,
};

pub fn expand_type_support(input: &DeriveInput) -> Result<TokenStream> {
    let input_attributes = get_input_attributes(input)?;
    let ident = &input.ident;
    let type_name = ident.to_string();

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
    let (get_type_quote, create_dynamic_sample_quote) = match &input.data {
        syn::Data::Struct(data_struct) => {
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
                        is_nested: #is_nested,
                    });
            };

            let mut member_builder_seq = quote! {};
            let mut member_sample_seq = quote! {};
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
                member_builder_seq.extend(
                    quote! {
                         builder.add_member(dust_dds::xtypes::dynamic_type::MemberDescriptor {
                            name: alloc::string::String::from(#field_name),
                            id: #member_id,
                            r#type: <#member_type as dust_dds::xtypes::dynamic_type::XTypesBinding>::get_dynamic_type(),
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
                match &field.ident {
                    Some(field_ident) => {
                        if is_optional {
                            member_dynamic_sample_seq.push(quote! {
                            if let Some(x) = self.#field_ident {
                                dust_dds::xtypes::dynamic_type::XTypesBinding::insert_value(x, &mut data, #member_id).unwrap();
                            }
                        });
                        } else {
                            member_sample_seq.extend(quote! {
                            #field_ident: dust_dds::infrastructure::type_support::TypeSupport::create_sample(src.remove_value(#member_id)?)?,
                        });
                            member_dynamic_sample_seq.push(quote! {
                            dust_dds::xtypes::dynamic_type::XTypesBinding::insert_value(self.#field_ident, &mut data, #member_id).unwrap();
                        });
                        }
                    }
                    None => {
                        let index = Index::from(field_index);
                        member_sample_seq.extend(quote! {  dust_dds::infrastructure::type_support::TypeSupport::create_sample(src.remove_value(#member_id)?)?,});
                        member_dynamic_sample_seq.push(quote! {
                            dust_dds::xtypes::dynamic_type::XTypesBinding::insert_value(self.#index, &mut data, #member_id).unwrap();
                        })
                    }
                }
            }
            let _is_tuple = data_struct
                .fields
                .iter()
                .next()
                .expect("Not empty")
                .ident
                .is_none();

            let get_type_quote = quote! {
                #struct_builder
                #member_builder_seq
                builder.build()
            };

            let create_dynamic_sample_quote = quote! {
                let mut data = dust_dds::xtypes::dynamic_type::DynamicDataFactory::create_data(Self::get_type());
                #(#member_dynamic_sample_seq)*
                data
            };
            Ok((get_type_quote, create_dynamic_sample_quote))
        }
        syn::Data::Enum(data_enum) => {
            // Separate between Unions and Enumeration which are both
            // mapped as Rust enum types
            if is_enum_xtypes_union(data_enum) {
                let union_builder = quote! {
                    extern crate alloc;
                    let mut builder = dust_dds::xtypes::dynamic_type::DynamicTypeBuilderFactory::create_type(
                        dust_dds::xtypes::dynamic_type::TypeDescriptor {
                            kind: dust_dds::xtypes::dynamic_type::TK_UNION,
                            name: alloc::string::String::from(#type_name),
                            base_type: None,
                            discriminator_type: None,
                            bound: alloc::vec::Vec::new(),
                            element_type: None,
                            key_element_type: None,
                            extensibility_kind: #extensibility_kind,
                            is_nested: #is_nested,
                        });
                };
                let get_type_quote = quote! {
                    #union_builder

                    builder.build()
                };

                let create_dynamic_sample_quote = quote! {todo!()};
                Ok((get_type_quote, create_dynamic_sample_quote))
            } else {
                // Note: Mapping has to be done with a match self strategy because the enum might not be copy so casting it using e.g. "self as i64" would
                // be consuming it.
                let discriminator_mapping = read_enum_variant_discriminant_mapping(data_enum);
                let max_discriminator = *discriminator_mapping
                    .iter()
                    .map(|(_, v)| v)
                    .max()
                    .expect("Map contains at least a value");

                let (discriminator_type, discriminator_dynamic_value) = if max_discriminator
                    > 0usize
                    && max_discriminator <= u8::MAX as usize
                {
                    (
                        quote! {dust_dds::xtypes::dynamic_type::DynamicTypeBuilderFactory::get_primitive_type(dust_dds::xtypes::dynamic_type::TK_UINT8)},
                        quote! {data.set_uint8_value(0, self as u8).unwrap();},
                    )
                } else if max_discriminator > u8::MAX as usize
                    && max_discriminator <= u16::MAX as usize
                {
                    (
                        quote! {dust_dds::xtypes::dynamic_type::DynamicTypeBuilderFactory::get_primitive_type(dust_dds::xtypes::dynamic_type::TK_UINT16)},
                        quote! {data.set_uint16_value(0, self as u16).unwrap();},
                    )
                } else if max_discriminator > u16::MAX as usize
                    && max_discriminator <= u32::MAX as usize
                {
                    (
                        quote! {dust_dds::xtypes::dynamic_type::DynamicTypeBuilderFactory::get_primitive_type(dust_dds::xtypes::dynamic_type::TK_UINT32)},
                        quote! {data.set_uint32_value(0, self as u32).unwrap();},
                    )
                } else {
                    return Err(syn::Error::new(
                        data_enum.enum_token.span,
                        "Enum discriminant size above maximum of u32::MAX",
                    ));
                };
                let enum_builder = quote! {
                    extern crate alloc;
                    let mut builder = dust_dds::xtypes::dynamic_type::DynamicTypeBuilderFactory::create_type(
                        dust_dds::xtypes::dynamic_type::TypeDescriptor {
                            kind: dust_dds::xtypes::dynamic_type::TK_ENUM,
                            name: alloc::string::String::from(#type_name),
                            base_type: None,
                            discriminator_type: Some(#discriminator_type),
                            bound: alloc::vec::Vec::new(),
                            element_type: None,
                            key_element_type: None,
                            extensibility_kind: #extensibility_kind,
                            is_nested: #is_nested,
                        });
                };
                let get_type_quote = quote! {
                    #enum_builder

                    builder.build()
                };

                let create_dynamic_sample_quote = quote! {
                    let mut data = dust_dds::xtypes::dynamic_type::DynamicDataFactory::create_data(Self::get_type());
                    #discriminator_dynamic_value
                    data
                };
                Ok((get_type_quote, create_dynamic_sample_quote))
            }
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
                #get_type_quote
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
    extensibility: Extensibility,
    is_nested: bool,
}

fn get_input_attributes(input: &DeriveInput) -> Result<InputAttributes> {
    let mut extensibility = Extensibility::Final;
    let mut is_nested = false;
    if let Some(xtypes_attribute) = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("dust_dds"))
    {
        xtypes_attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("extensibility") {
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
        extensibility,
        is_nested,
    })
}

// The return of this function is a Vec instead of a HashMap so that the tests give
// consistent results. Iterating over a HashMap gives different order of members every time.
// The order is also important for the XML string generation.
pub fn read_enum_variant_discriminant_mapping(data_enum: &DataEnum) -> Vec<(Ident, usize)> {
    let mut map = Vec::new();
    let mut discriminant = 0;
    for variant in data_enum.variants.iter() {
        if let Some((_, discriminant_expr)) = &variant.discriminant {
            match discriminant_expr {
                Expr::Lit(ExprLit { lit, .. }) => match lit {
                    Lit::Int(lit_int) => {
                        discriminant = lit_int
                            .base10_parse()
                            .expect("Integer should be verified by compiler")
                    }
                    _ => panic!("Only literal integer discrimimants are expected"),
                },
                _ => panic!("Only literal discrimimants are expected"),
            }
        }
        map.push((variant.ident.clone(), discriminant));
        discriminant += 1;
    }

    map
}

pub fn is_enum_xtypes_union(data_enum: &DataEnum) -> bool {
    data_enum
        .variants
        .iter()
        .any(|v| !matches!(&v.fields, Fields::Unit))
}
