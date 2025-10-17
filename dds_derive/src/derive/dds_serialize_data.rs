use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{DeriveInput, Result, LitStr};
use crate::derive::attributes::{get_input_extensibility, get_field_attributes, Extensibility};

fn collect_keys_and_ids(input: &DeriveInput) -> Result<(Vec<String>, Vec<String>)> {
    let mut keys = Vec::new();
    let mut ids = Vec::new();

    if let syn::Data::Struct(data_struct) = &input.data {
        if let syn::Fields::Named(fields_named) = &data_struct.fields {
            for field in &fields_named.named {
                let field_attrs = get_field_attributes(field)?;
                if field_attrs.key {
                    keys.push(field.ident.as_ref().unwrap().to_string());
                }
                if let Some(expr) = field_attrs.id {
                    ids.push(format!("{}={:?}", field.ident.as_ref().unwrap(), expr));
                }
            }
        }
    }

    Ok((keys, ids))
}

pub fn expand_dds_deserialize_data(input: &DeriveInput) -> Result<TokenStream> {
    // Reject unions early
    if let syn::Data::Union(data_union) = &input.data {
        return Err(syn::Error::new(
            data_union.union_token.span,
            "Union not supported",
        ));
    }

    let extensibility = get_input_extensibility(input)?;
    let (keys, ids) = collect_keys_and_ids(input)?;

    let appendable_flag = matches!(extensibility, Extensibility::Appendable);
    let final_flag = matches!(extensibility, Extensibility::Final);
    let mutable_flag = matches!(extensibility, Extensibility::Mutable);

    let key_fields = keys.join(", ");
    let id_fields = ids.join(", ");
    let flags_info_string = format!(
        "appendable: {}, final: {}, mutable: {}, keys: {}, ids: {}",
        appendable_flag, final_flag, mutable_flag, key_fields, id_fields
    );
    let flags_info_literal = LitStr::new(&flags_info_string, Span::call_site());

    // Lifetime and generics
    let (_, type_generics, where_clause) = input.generics.split_for_impl();

    let mut de_lifetime_param = syn::LifetimeParam::new(syn::Lifetime::new("'__de", Span::call_site()));
    for lifetime_def in input.generics.lifetimes() {
        de_lifetime_param.bounds.push(lifetime_def.lifetime.clone());
    }

    let mut generics = input.generics.clone();
    generics.params = Some(syn::GenericParam::Lifetime(de_lifetime_param))
        .into_iter()
        .chain(generics.params)
        .collect();

    let ident = &input.ident;

    let deserialize_function = quote! {
        dust_dds::infrastructure::type_support::deserialize_rtps_encapsulated_data(&mut serialized_data)
    };

    Ok(quote! {
        #[automatically_derived]
        impl #generics dust_dds::infrastructure::type_support::DdsDeserialize<'__de> for #ident #type_generics #where_clause {
            fn deserialize_data(mut serialized_data: &'__de [u8]) -> dust_dds::infrastructure::error::DdsResult<Self> {
                #deserialize_function
            }
        }

        impl #generics #ident #type_generics #where_clause {
            #[allow(dead_code)]
            fn _dds_deserialize_flags_info() -> &'static str {
                #flags_info_literal
            }
        }
    })
}