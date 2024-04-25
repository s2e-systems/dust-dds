use super::enum_support::{get_enum_bitbound, read_enum_variant_discriminant_mapping, BitBound};
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, DeriveInput, Field};
use xml::{writer::XmlEvent, EmitterConfig, EventWriter};

fn field_has_key_attribute(field: &Field) -> syn::Result<bool> {
    let mut has_key = false;
    if let Some(dust_dds_attribute) = field
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("dust_dds"))
    {
        dust_dds_attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("key") {
                has_key = true;
                Ok(())
            } else {
                Err(syn::Error::new(
                    meta.path.span(),
                    format!(
                        "Unexpected element {}. Valid options are \"key\"",
                        meta.path.into_token_stream(),
                    ),
                ))
            }
        })?;
    }
    Ok(has_key)
}

enum XTypeKind {
    Basic(&'static str),
    NonBasic,
}

fn convert_rust_type_to_xtypes(type_ident: &Ident) -> XTypeKind {
    let type_string = type_ident.to_string();
    match type_string.as_str() {
        "bool" => XTypeKind::Basic("boolean"),
        "char" => XTypeKind::Basic("char8"),
        "i32" => XTypeKind::Basic("int32"),
        "u32" => XTypeKind::Basic("uint32"),
        "i8" => XTypeKind::Basic("int8"),
        "u8" => XTypeKind::Basic("uint8"),
        "i16" => XTypeKind::Basic("int16"),
        "u16" => XTypeKind::Basic("uint16"),
        "i64" => XTypeKind::Basic("int64"),
        "u64" => XTypeKind::Basic("uint64"),
        "f32" => XTypeKind::Basic("float32"),
        "f64" => XTypeKind::Basic("float64"),
        "f128" => unimplemented!("Type not valid for XML representation"),
        "String" => XTypeKind::Basic("string"),
        _ => XTypeKind::NonBasic,
    }
}

pub fn expand_dds_type_xml(input: &DeriveInput) -> syn::Result<TokenStream> {
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let ident = &input.ident;
    let mut output_xml = Vec::new();
    let mut xml_writer = EventWriter::new_with_config(
        &mut output_xml,
        EmitterConfig::new().write_document_declaration(false),
    );

    match &input.data {
        syn::Data::Struct(data_struct) => {
            xml_writer
                .write(XmlEvent::start_element("struct").attr("name", &ident.to_string()))
                .expect("Failed to write struct XML element");
            for (field_index, field) in data_struct.fields.iter().enumerate() {
                let field_name = match &field.ident {
                    Some(ident) => ident.to_string(),
                    None => field_index.to_string(),
                };
                let field_has_key = field_has_key_attribute(field)?;
                let mut field_element = XmlEvent::start_element("member").attr("name", &field_name);
                if field_has_key {
                    field_element = field_element.attr("key", "true");
                }

                match &field.ty {
                    syn::Type::Array(_) => todo!(),
                    syn::Type::Path(p) => match p.path.get_ident() {
                        Some(i) => {
                            let xtypes_type = convert_rust_type_to_xtypes(i);
                            let type_name = i.to_string();
                            field_element = match xtypes_type {
                                XTypeKind::Basic(type_name) => field_element.attr("type", type_name),
                                XTypeKind::NonBasic => field_element.attr("type", "nonBasic").attr("nonBasicTypeName", &type_name),
                            };
                            xml_writer.write(field_element).unwrap_or_else(|_| {
                                panic!("Failed to write member start element for {}", field_name)
                            });

                        }
                        None => {
                            if p.path.segments[0].ident == "Vec" {
                                match &p.path.segments[0].arguments {
                                    syn::PathArguments::AngleBracketed(a) => match &a.args[0] {
                                        syn::GenericArgument::Type(ty) => match ty {
                                            syn::Type::Path(p) => match p.path.get_ident() {
                                                Some(i) => {
                                                    let xtypes_type =
                                                        convert_rust_type_to_xtypes(i);
                                                    let type_name = i.to_string();
                                                    field_element =
                                                        match xtypes_type {
                                                            XTypeKind::Basic(type_name) => field_element
                                                                .attr("type", type_name),
                                                            XTypeKind::NonBasic => field_element
                                                                .attr("type", "nonBasic").attr("nonBasicTypeName", &type_name),
                                                        }.attr("sequenceMaxLength", "-1")
                                                    ;
                                                    xml_writer.write(field_element).unwrap_or_else(|_| {
                                                        panic!("Failed to write member start element for {}", field_name)
                                                    });
                                                }
                                                None => todo!(),
                                            },

                                            _ => todo!(),
                                        },
                                        _ => panic!("Only type expected in arguments"),
                                    },
                                    _ => panic!("Only angle bracketed arguments expect for a Vec"),
                                }
                            } else {
                                todo!()
                            }
                        }
                    },
                    syn::Type::Reference(r) => {
                        match r.elem.as_ref() {
                            syn::Type::Slice(s) => match s.elem.as_ref() {
                                syn::Type::Path(p) => match p.path.get_ident() {
                                    Some(i) => {
                                        let xtypes_type =
                                            convert_rust_type_to_xtypes(i);
                                        let type_name = i.to_string();
                                        field_element = match xtypes_type {
                                            XTypeKind::Basic(type_name) => field_element
                                                .attr("type", type_name),
                                            XTypeKind::NonBasic => field_element
                                                .attr("type", "nonBasic").attr("nonBasicTypeName", &type_name),
                                        }.attr("sequenceMaxLength", "-1");
                                        xml_writer.write(field_element).unwrap_or_else(|_| {
                                            panic!("Failed to write member start element for {}", field_name)
                                        });

                                    },
                                    None => todo!(),
                                }
                                _ => unimplemented!("Only slice of ident supported"),
                            }
                            _=> unimplemented!("Only reference to slice supported"),
                        }
                    },
                    syn::Type::Tuple(t) => return Err(syn::Error::new(
                        t.paren_token.span.open(),
                        "Tuple types not supported for automatic XML drive. Use a custom struct instead",
                    )),
                    _ => return Err(syn::Error::new(
                        field
                            .colon_token
                            .expect("Field expect to contain colon token for type definition")
                            .span,
                        "Type not supported for automatic XML derive",
                    )),
                };

                xml_writer
                    .write(XmlEvent::end_element())
                    .unwrap_or_else(|_| {
                        panic!("Failed to write member end element for {}", field_name)
                    });
            }
            xml_writer
                .write(XmlEvent::end_element())
                .expect("Failed to write struct XML end element");
            let output_xml_string =
                String::from_utf8(output_xml).expect("Failed to convert XML to string");

            Ok(quote! {
                impl #impl_generics dust_dds::topic_definition::type_support::DdsTypeXml for #ident #type_generics #where_clause {
                    fn get_type_xml() -> Option<String> {
                        Some(#output_xml_string.to_string())
                    }
                }
            })
        }
        syn::Data::Enum(data_enum) => {
            let discriminant_mapping = read_enum_variant_discriminant_mapping(data_enum);
            let max_discriminant = discriminant_mapping
                .iter()
                .map(|(_, v)| v)
                .max()
                .expect("Map contains at least a value");
            let bitbound = match get_enum_bitbound(max_discriminant) {
                BitBound::Bit8 => "8",
                BitBound::Bit16 => "16",
                BitBound::Bit32 => "32",
            };
            xml_writer
                .write(
                    XmlEvent::start_element("enum")
                        .attr("name", &ident.to_string())
                        .attr("bitBound", bitbound),
                )
                .expect("Failed to write enum XML element");
            for (variant_name, variant_discriminant) in discriminant_mapping.iter() {
                xml_writer
                    .write(
                        XmlEvent::start_element("enumerator")
                            .attr("name", &variant_name.to_string())
                            .attr("value", &variant_discriminant.to_string()),
                    )
                    .expect("Failed to write enumerator XML element");
                xml_writer
                    .write(XmlEvent::end_element())
                    .expect("Failed to write enumerator XML end element");
            }
            xml_writer
                .write(XmlEvent::end_element())
                .expect("Failed to write enum XML end element");
            let output_xml_string =
                String::from_utf8(output_xml).expect("Failed to convert XML to string");

            Ok(quote! {
                impl #impl_generics dust_dds::topic_definition::type_support::DdsTypeXml for #ident #type_generics #where_clause {
                    fn get_type_xml() -> Option<String> {
                        Some(#output_xml_string.to_string())
                    }
                }
            })
        }
        syn::Data::Union(data_union) => Err(syn::Error::new(
            data_union.union_token.span,
            "Union not supported",
        )),
    }
}

#[cfg(test)]
mod tests {
    use quote::ToTokens;
    use syn::ItemImpl;

    use super::*;

    #[test]
    fn struct_with_simple_types() {
        let input = syn::parse2::<DeriveInput>(
            "
            struct TestStruct {
                #[dust_dds(key)]
                id: u8,
                name: String,
                value: f32,
            }
        "
            .parse()
            .unwrap(),
        )
        .unwrap();

        let result = syn::parse2::<ItemImpl>(expand_dds_type_xml(&input).unwrap()).unwrap();
        let expected = syn::parse2::<ItemImpl>(
            r#"impl dust_dds::topic_definition::type_support::DdsTypeXml for TestStruct {
                fn get_type_xml() -> Option<String> {
                    Some("<struct name=\"TestStruct\"><member name=\"id\" key=\"true\" type=\"uint8\" /><member name=\"name\" type=\"string\" /><member name=\"value\" type=\"float32\" /></struct>".to_string())
                }
            }"#
            .parse()
            .unwrap(),
        )
        .unwrap();

        assert_eq!(
            result,
            expected,
            "\n\n Result: {:?} \n Expected: {:?}",
            result.clone().into_token_stream().to_string(),
            expected.clone().into_token_stream().to_string()
        );
    }

    #[test]
    fn struct_with_sequence_types() {
        let input = syn::parse2::<DeriveInput>(
            "
            struct TestStruct {
                #[dust_dds(key)]
                id: u8,
                value_list: Vec<u8>,
            }
        "
            .parse()
            .unwrap(),
        )
        .unwrap();

        let result = syn::parse2::<ItemImpl>(expand_dds_type_xml(&input).unwrap()).unwrap();
        let expected = syn::parse2::<ItemImpl>(
            r#"impl dust_dds::topic_definition::type_support::DdsTypeXml for TestStruct {
                fn get_type_xml() -> Option<String> {
                    Some("<struct name=\"TestStruct\"><member name=\"id\" key=\"true\" type=\"uint8\" /><member name=\"value_list\" type=\"uint8\" sequenceMaxLength=\"-1\" /></struct>".to_string())
                }
            }"#
            .parse()
            .unwrap(),
        )
        .unwrap();

        assert_eq!(
            result,
            expected,
            "\n\n Result: {:?} \n Expected: {:?}",
            result.clone().into_token_stream().to_string(),
            expected.clone().into_token_stream().to_string()
        );
    }

    #[test]
    fn enumeration() {
        let input = syn::parse2::<DeriveInput>(
            "
            enum MyEnum {
                VariantA=500,
                VariantB=600,
            }
        "
            .parse()
            .unwrap(),
        )
        .unwrap();

        let result = syn::parse2::<ItemImpl>(expand_dds_type_xml(&input).unwrap()).unwrap();
        let expected = syn::parse2::<ItemImpl>(
            r#"impl dust_dds::topic_definition::type_support::DdsTypeXml for MyEnum {
                fn get_type_xml() -> Option<String> {
                    Some("<enum name=\"MyEnum\" bitBound=\"16\"><enumerator name=\"VariantA\" value=\"500\" /><enumerator name=\"VariantB\" value=\"600\" /></enum>".to_string())
                }
            }"#
            .parse()
            .unwrap(),
        )
        .unwrap();

        assert_eq!(
            result,
            expected,
            "\n\n Result: {:?} \n Expected: {:?}",
            result.clone().into_token_stream().to_string(),
            expected.clone().into_token_stream().to_string()
        );
    }
}
