use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::DeriveInput;
use xml::{writer::XmlEvent, EmitterConfig, EventWriter};

fn convert_rust_type_to_xtypes(type_ident: &Ident) -> &str {
    let type_string = type_ident.to_string();
    match type_string.as_str() {
        "bool" => "boolean",
        "char" => "char8",
        "i32" => "int32",
        "u32" => "uint32",
        "i8" => "int8",
        "u8" => "uint8",
        "i16" => "int16",
        "u16" => "uint16",
        "i64" => "int64",
        "u64" => "uint64",
        "f32" => "float32",
        "f64" => "float64",
        "f128" => unimplemented!("Type not valid for XML"),
        "String" => "string",
        _ => todo!("Other objects not yet support"),
    }
}

pub fn expand_dds_type_xml(input: &DeriveInput) -> syn::Result<TokenStream> {
    match &input.data {
        syn::Data::Struct(data_struct) => {
            let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
            let ident = &input.ident;
            let mut output_xml = Vec::new();
            let mut xml_writer = EventWriter::new_with_config(
                &mut output_xml,
                EmitterConfig::new().write_document_declaration(false),
            );
            xml_writer
                .write(XmlEvent::start_element("struct").attr("name", &ident.to_string()))
                .expect("Failed to write struct XML element");
            for (field_index, field) in data_struct.fields.iter().enumerate() {
                let field_name = match &field.ident {
                    Some(ident) => ident.to_string(),
                    None => field_index.to_string(),
                };
                let field_element = XmlEvent::start_element("member").attr("name", &field_name);

                let field_element = match &field.ty {
                    syn::Type::Array(_) => todo!(),
                    syn::Type::Path(p) => match p.path.get_ident() {
                        Some(i) => {
                            let xtypes_type = convert_rust_type_to_xtypes(i);
                            Ok(field_element.attr("type", xtypes_type))
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
                                                    Ok(field_element
                                                        .attr("type", xtypes_type)
                                                        .attr("sequenceMaxLength", "-1"))
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
                                        Ok(field_element
                                            .attr("type", xtypes_type)
                                            .attr("sequenceMaxLength", "-1"))
                                    },
                                    None => todo!(),
                                }
                                _ => unimplemented!("Only slice of ident supported"),
                            }
                            _=> unimplemented!("Only reference to slice supported"),
                        }
                    },
                    syn::Type::Tuple(t) => Err(syn::Error::new(
                        t.paren_token.span.open(),
                        "Tuple types not supported for automatic XML drive. Use a custom struct instead",
                    )),
                    _ => Err(syn::Error::new(
                        field
                            .colon_token
                            .expect("Field expect to contain colon token for type definition")
                            .span,
                        "Type not supported for automatic XML derive",
                    )),
                }?;
                xml_writer.write(field_element).unwrap_or_else(|_| {
                    panic!("Failed to write member start element for {}", field_name)
                });
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
                    Some("<struct name=\"TestStruct\"><member name=\"id\" type=\"uint8\" /><member name=\"name\" type=\"string\" /><member name=\"value\" type=\"float32\" /></struct>".to_string())
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
                    Some("<struct name=\"TestStruct\"><member name=\"id\" type=\"uint8\" /><member name=\"value_list\" type=\"uint8\" sequenceMaxLength=\"-1\" /></struct>".to_string())
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
