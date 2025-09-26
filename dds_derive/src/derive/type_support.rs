use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    spanned::Spanned, DeriveInput, Expr, Field, GenericArgument, Index, PathArguments, Result,
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
    let (get_type_quote, create_sample_quote, create_dynamic_sample_quote) = match &input.data {
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
            let mut member_dynamic_sample_seq = quote! {};

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
                let field_type = &field.ty;
                match &field.ident {
                    Some(field_ident) => {
                        member_sample_seq.extend(quote! {
                            #field_ident: <#field_type as dust_dds::infrastructure::type_support::TypeSupport>::create_sample(src.remove_value(#member_id)?)?,
                        });
                        member_dynamic_sample_seq.extend(quote! {
                            .insert_value(#member_id,  <#field_type as dust_dds::infrastructure::type_support::TypeSupport>::create_dynamic_sample(self.#field_ident))
                        })
                    }
                    None => {
                        let index = Index::from(field_index);
                        member_sample_seq.extend(quote! {  <#field_type as dust_dds::infrastructure::type_support::TypeSupport>::create_sample(src.remove_value(#member_id)?)?,});
                        member_dynamic_sample_seq.extend(quote! {
                            .insert_value(#member_id, <#field_type as dust_dds::infrastructure::type_support::TypeSupport>::create_dynamic_sample(self.#index))
                        })
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
                #struct_builder
                #member_builder_seq
                builder.build()
            };

            let create_sample_quote = if is_tuple {
                quote! {Ok(Self (
                    #member_sample_seq
                ))}
            } else {
                quote! {Ok(Self {
                    #member_sample_seq
                })}
            };
            let create_dynamic_sample_quote = quote! {
                dust_dds::xtypes::dynamic_type::DynamicDataFactory::create_data(Self::get_type())#member_dynamic_sample_seq
            };
            Ok((
                get_type_quote,
                create_sample_quote,
                create_dynamic_sample_quote,
            ))
        }
        syn::Data::Enum(_data_enum) => {
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
                        is_nested: #is_nested,
                    });
            };
            let get_type_quote = quote! {
                #enum_builder

                builder.build()
            };
            let create_sample_quote = quote! {todo!()};
            let create_dynamic_sample_quote = quote! {todo!()};
            Ok((
                get_type_quote,
                create_sample_quote,
                create_dynamic_sample_quote,
            ))
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

            fn create_sample(mut src: dust_dds::xtypes::dynamic_type::DynamicData) -> dust_dds::infrastructure::error::DdsResult<Self> {
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
                    "Final" => {
                        extensibility = Extensibility::Final;
                        Ok(())
                    }
                    "Appendable" => {
                        extensibility = Extensibility::Appendable;
                        Ok(())
                    }
                    "Mutable" => {
                        extensibility = Extensibility::Mutable;
                        Ok(())
                    }
                    _ => Err(syn::Error::new(
                        meta.path.span(),
                        r#"Invalid format specified. Valid options are "Final", "Appendable", "Mutable". "#,
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

struct FieldAttributes {
    pub key: bool,
    pub id: Option<Expr>,
}

fn get_field_attributes(field: &Field) -> syn::Result<FieldAttributes> {
    let mut key = false;
    let mut id = None;
    if let Some(xtypes_attribute) = field
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("dust_dds"))
    {
        xtypes_attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("key") {
                key = true;
            } else if meta.path.is_ident("id") {
                id = Some(meta.value()?.parse()?);
            }
            Ok(())
        })?;
    }
    Ok(FieldAttributes { key, id })
}
