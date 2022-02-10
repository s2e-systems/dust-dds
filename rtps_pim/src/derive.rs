extern crate proc_macro2;

// #[derive(FromDeriveInput)]
// struct Attributes {
//     attr_name: syn::Ident,
// }

#[proc_macro_derive(RtpsEntityAttributes, attributes(entity))]
pub fn macro_derive_rtps_entity_attributes(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input);
    let entity = Attributes::from_derive_input(&ast)
        .expect("(;_;) expected identifier")
        .attr_name;

    derive_rtps_entity_attributes(&ast.ident, &entity).into()
}

fn derive_rtps_entity_attributes(impl_name: &syn::Ident, entity: &syn::Ident) -> impl Into<TokenStream> {
    quote! {
        impl RtpsEntityAttributes for #impl_name {
            fn guid(&self) -> &Guid {
                self.#entity.guid()
            }
        }
    }
}