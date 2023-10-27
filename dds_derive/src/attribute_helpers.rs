use syn::{DataStruct, Field};

pub fn field_has_key_attribute(field: &Field) -> bool {
    field.attrs.iter().any(|attr| attr.path().is_ident("key"))
}

pub fn struct_has_key(data_struct: &DataStruct) -> bool {
    data_struct.fields.iter().any(field_has_key_attribute)
}
