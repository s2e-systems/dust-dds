pub struct ParsedField {
    name: String,
    is_key: bool,
}

impl ParsedField {
    pub fn new(name: String, is_key: bool) -> Self {
        Self { name, is_key }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn is_key(&self) -> bool {
        self.is_key
    }
}

pub struct ParsedStruct {
    name: String,
    field_list: Vec<ParsedField>,
}

impl ParsedStruct {
    pub fn new(name: String, field_list: Vec<ParsedField>) -> Self {
        Self { name, field_list }
    }

    pub fn try_from_derive_input(input: &syn::DeriveInput) -> syn::Result<Self> {
        match &input.data {
            syn::Data::Struct(data_struct) => {
                let name = input.ident.to_string();
                let mut field_list = Vec::with_capacity(data_struct.fields.len());
                for (index, field) in data_struct.fields.iter().enumerate() {
                    let field_name = match &field.ident {
                        Some(n) => n.to_string(),
                        None => index.to_string(),
                    };
                    let is_key = field.attrs.iter().any(|attr| attr.path().is_ident("key"));
                    field_list.push(ParsedField::new(field_name, is_key));
                }

                Ok(Self { name, field_list })
            }
            syn::Data::Enum(data_enum) => Err(syn::Error::new(
                data_enum.enum_token.span,
                "Enum not supported",
            )),
            syn::Data::Union(data_union) => Err(syn::Error::new(
                data_union.union_token.span,
                "Union not supported",
            )),
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn field_list(&self) -> &[ParsedField] {
        &self.field_list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn struct_without_key() {
        let input: syn::DeriveInput = syn::parse2(
            " struct WithoutKey{
                        id: u8,
                    }"
            .parse()
            .unwrap(),
        )
        .unwrap();
        let parsed_struct = ParsedStruct::try_from_derive_input(&input).unwrap();
        assert_eq!(parsed_struct.name(), "WithoutKey");
        assert_eq!(parsed_struct.field_list()[0].name(), "id");
        assert_eq!(parsed_struct.field_list()[0].is_key(), false);
    }

    #[test]
    fn struct_with_key() {
        let input: syn::DeriveInput = syn::parse2(
            " struct WithKey{
                        #[key]
                        id: u8,
                        data: Vec<u8>
                    }"
            .parse()
            .unwrap(),
        )
        .unwrap();
        let parsed_struct = ParsedStruct::try_from_derive_input(&input).unwrap();
        assert_eq!(parsed_struct.name(), "WithKey");
        assert_eq!(parsed_struct.field_list()[0].name(), "id");
        assert_eq!(parsed_struct.field_list()[0].is_key(), true);
        assert_eq!(parsed_struct.field_list()[1].name(), "data");
        assert_eq!(parsed_struct.field_list()[1].is_key(), false);
    }

    #[test]
    fn tuple_without_key() {
        let input: syn::DeriveInput =
            syn::parse2(" struct TupleWithoutKey(u8, u32);".parse().unwrap()).unwrap();
        let parsed_struct = ParsedStruct::try_from_derive_input(&input).unwrap();
        assert_eq!(parsed_struct.name(), "TupleWithoutKey");
        assert_eq!(parsed_struct.field_list()[0].name(), "0");
        assert_eq!(parsed_struct.field_list()[0].is_key(), false);
        assert_eq!(parsed_struct.field_list()[1].name(), "1");
        assert_eq!(parsed_struct.field_list()[1].is_key(), false);
    }

    #[test]
    fn tuple_with_key() {
        let input: syn::DeriveInput =
            syn::parse2(" struct TupleWithKey(u8, #[key] u32);".parse().unwrap()).unwrap();
        let parsed_struct = ParsedStruct::try_from_derive_input(&input).unwrap();
        assert_eq!(parsed_struct.name(), "TupleWithKey");
        assert_eq!(parsed_struct.field_list()[0].name(), "0");
        assert_eq!(parsed_struct.field_list()[0].is_key(), false);
        assert_eq!(parsed_struct.field_list()[1].name(), "1");
        assert_eq!(parsed_struct.field_list()[1].is_key(), true);
    }
}
