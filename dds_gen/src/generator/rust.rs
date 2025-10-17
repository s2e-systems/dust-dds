use crate::parser::{IdlPair, Rule, StructDef};

pub fn generate_rust_source(pair: IdlPair, writer: &mut String) {
    match pair.as_rule() {
        Rule::EOI => (),
        Rule::escape => todo!(),
        Rule::octal_escape => todo!(),
        Rule::hex_escape => todo!(),
        Rule::unicode_escape => todo!(),
        Rule::WHITESPACE => (),
        Rule::block_comment => (),
        Rule::line_comment => (),
        Rule::COMMENT => (),
        Rule::reserved_keyword => (),
        Rule::identifier => identifier(pair, writer),
        Rule::character_literal => todo!(),
        Rule::string_literal => todo!(),
        Rule::wide_character_literal => todo!(),
        Rule::wide_string_literal => todo!(),
        Rule::integer_literal => todo!(),
        Rule::decimal_integer_literal => todo!(),
        Rule::octal_integer_literal => todo!(),
        Rule::hex_integer_literal => todo!(),
        Rule::fixed_pt_literal => todo!(),
        Rule::floating_pt_literal => todo!(),
        Rule::integral_part => todo!(),
        Rule::fractional_part => todo!(),
        Rule::exponent => todo!(),
        Rule::float_suffix => todo!(),
        Rule::specification => specification(pair, writer),
        Rule::definition => definition(pair, writer),
        Rule::module_dcl => module_dcl(pair, writer),
        Rule::scoped_name => scoped_name(pair, writer),
        Rule::const_dcl => const_dcl(pair, writer),
        Rule::const_type => const_type(pair, writer),
        Rule::const_expr => const_expr(pair, writer),
        Rule::or_expr => todo!(),
        Rule::xor_expr => todo!(),
        Rule::and_expr => todo!(),
        Rule::lshift_expr => todo!(),
        Rule::rshift_expr => todo!(),
        Rule::add_expr => todo!(),
        Rule::sub_expr => todo!(),
        Rule::mul_expr => todo!(),
        Rule::div_expr => todo!(),
        Rule::mod_expr => todo!(),
        Rule::unary_expr => todo!(),
        Rule::unary_operator => todo!(),
        Rule::primary_expr => todo!(),
        Rule::literal => todo!(),
        Rule::boolean_literal => todo!(),
        Rule::positive_int_const => positive_int_const(pair, writer),
        Rule::type_dcl => type_dcl(pair, writer),
        Rule::type_spec => type_spec(pair, writer),
        Rule::simple_type_spec => simple_type_spec(pair, writer),
        Rule::base_type_spec => base_type_spec(pair, writer),
        Rule::floating_pt_type => floating_pt_type(pair, writer),
        Rule::integer_type => integer_type(pair, writer),
        Rule::signed_tiny_int => signed_tiny_int(pair, writer),
        Rule::signed_int => signed_int(pair, writer),
        Rule::signed_short_int => signed_short_int(pair, writer),
        Rule::signed_long_int => signed_long_int(pair, writer),
        Rule::signed_longlong_int => signed_longlong_int(pair, writer),
        Rule::unsigned_tiny_int => unsigned_tiny_int(pair, writer),
        Rule::unsigned_int => unsigned_int(pair, writer),
        Rule::unsigned_short_int => unsigned_short_int(pair, writer),
        Rule::unsigned_long_int => unsigned_long_int(pair, writer),
        Rule::unsigned_longlong_int => unsigned_longlong_int(pair, writer),
        Rule::char_type => char_type(pair, writer),
        Rule::wide_char_type => wide_char_type(pair, writer),
        Rule::boolean_type => boolean(pair, writer),
        Rule::octet_type => octet_type(pair, writer),
        Rule::template_type_spec => template_type_spec(pair, writer),
        Rule::sequence_type => sequence_type(pair, writer),
        Rule::string_type => string_type(pair, writer),
        Rule::wide_string_type => wide_string_type(pair, writer),
        Rule::fixed_pt_type => unimplemented!("Fixed point not supported in Rust mapping"),
        Rule::fixed_pt_const_type => unimplemented!("Fixed point not supported in Rust mapping"),
        Rule::constr_type_dcl => constr_type_dcl(pair, writer),
        Rule::struct_dcl => struct_dcl(pair, writer),
        Rule::struct_def => struct_def(pair, writer),
        Rule::member => member(pair, writer),
        Rule::struct_forward_dcl => (), // Forward declarations are irrelevant in Rust mapping
        Rule::union_dcl => todo!(),
        Rule::union_def => todo!(),
        Rule::switch_type_spec => todo!(),
        Rule::switch_body => todo!(),
        Rule::case => todo!(),
        Rule::case_label => todo!(),
        Rule::element_spec => todo!(),
        Rule::union_forward_dcl => (), // Forward declarations are irrelevant in Rust mapping
        Rule::enum_dcl => enum_dcl(pair, writer),
        Rule::enumerator => enumerator(pair, writer),
        Rule::array_declarator => todo!(),
        Rule::fixed_array_size => fixed_array_size(pair, writer),
        Rule::native_dcl => todo!(),
        Rule::simple_declarator => simple_declarator(pair, writer),
        Rule::typedef_dcl => typedef_dcl(pair, writer),
        Rule::type_declarator => type_declarator(pair, writer),
        Rule::any_declarators => (), // Handled inside typedef_dcl
        Rule::any_declarator => any_declarator(pair, writer),
        Rule::declarators => declarators(pair, writer),
        Rule::declarator => todo!(),
        Rule::any_type => todo!(),
        Rule::except_dcl => todo!(),
        Rule::interface_dcl => interface_dcl(pair, writer),
        Rule::interface_def => interface_def(pair, writer),
        Rule::interface_forward_dcl => todo!(), // Forward declarations are irrelevant in Rust mapping
        Rule::interface_header => interface_header(pair, writer),
        Rule::interface_kind => interface_kind(pair, writer),
        Rule::interface_inheritance_spec => todo!(),
        Rule::interface_name => todo!(),
        Rule::interface_body => interface_body(pair, writer),
        Rule::export => export(pair, writer),
        Rule::op_dcl => op_dcl(pair, writer),
        Rule::op_type_spec => op_type_spec(pair, writer),
        Rule::parameter_dcls => parameter_dcls(pair, writer),
        Rule::param_dcl => param_dcl(pair, writer),
        Rule::param_attribute => param_attribute(pair, writer),
        Rule::raises_expr => todo!(),
        Rule::attr_dcl => todo!(),
        Rule::readonly_attr_spec => todo!(),
        Rule::readonly_attr_declarator => todo!(),
        Rule::attr_spec => todo!(),
        Rule::attr_declarator => todo!(),
        Rule::attr_raises_expr => todo!(),
        Rule::get_excep_expr => todo!(),
        Rule::set_excep_expr => todo!(),
        Rule::exception_list => todo!(),
        Rule::value_dcl => todo!(),
        Rule::value_def => todo!(),
        Rule::value_header => todo!(),
        Rule::value_kind => todo!(),
        Rule::value_inheritance_spec => todo!(),
        Rule::value_name => todo!(),
        Rule::value_element => todo!(),
        Rule::state_member => todo!(),
        Rule::init_dcl => todo!(),
        Rule::init_param_dcls => todo!(),
        Rule::init_param_dcl => todo!(),
        Rule::value_forward_dcl => todo!(),
        Rule::type_id_dcl => todo!(),
        Rule::type_prefix_dcl => todo!(),
        Rule::import_dcl => todo!(),
        Rule::imported_scope => todo!(),
        Rule::object_type => todo!(),
        Rule::op_oneway_dcl => todo!(),
        Rule::in_parameter_dcls => todo!(),
        Rule::in_param_dcl => todo!(),
        Rule::op_with_context => todo!(),
        Rule::context_expr => todo!(),
        Rule::value_box_def => todo!(),
        Rule::value_abs_def => todo!(),
        Rule::value_base_type => todo!(),
        Rule::component_dcl => todo!(),
        Rule::component_forward_dcl => todo!(),
        Rule::component_def => todo!(),
        Rule::component_header => todo!(),
        Rule::component_inheritance_spec => todo!(),
        Rule::component_body => todo!(),
        Rule::component_export => todo!(),
        Rule::provides_dcl => todo!(),
        Rule::interface_type => todo!(),
        Rule::uses_dcl => todo!(),
        Rule::home_dcl => todo!(),
        Rule::home_header => todo!(),
        Rule::home_inheritance_spec => todo!(),
        Rule::home_body => todo!(),
        Rule::home_export => todo!(),
        Rule::factory_dcl => todo!(),
        Rule::factory_param_dcls => todo!(),
        Rule::factory_param_dcl => todo!(),
        Rule::supported_interface_spec => todo!(),
        Rule::emits_dcl => todo!(),
        Rule::publishes_dcl => todo!(),
        Rule::consumes_dcl => todo!(),
        Rule::primary_key_spec => todo!(),
        Rule::finder_dcl => todo!(),
        Rule::event_dcl => todo!(),
        Rule::event_forward_dcl => todo!(),
        Rule::event_abs_def => todo!(),
        Rule::event_def => todo!(),
        Rule::event_header => todo!(),
        Rule::porttype_dcl => todo!(),
        Rule::porttype_forward_dcl => todo!(),
        Rule::porttype_def => todo!(),
        Rule::port_body => todo!(),
        Rule::port_ref => todo!(),
        Rule::port_export => todo!(),
        Rule::port_dcl => todo!(),
        Rule::connector_dcl => todo!(),
        Rule::connector_header => todo!(),
        Rule::connector_inherit_spec => todo!(),
        Rule::connector_export => todo!(),
        Rule::template_module_dcl => todo!(),
        Rule::formal_parameters => todo!(),
        Rule::formal_parameter => todo!(),
        Rule::formal_parameter_type => todo!(),
        Rule::tpl_definition => todo!(),
        Rule::template_module_inst => todo!(),
        Rule::actual_parameters => todo!(),
        Rule::actual_parameter => todo!(),
        Rule::template_module_ref => todo!(),
        Rule::formal_parameter_names => todo!(),
        Rule::map_type => todo!(),
        Rule::bitset_dcl => todo!(),
        Rule::bitfield => todo!(),
        Rule::bitfield_spec => todo!(),
        Rule::destination_type => todo!(),
        Rule::bitmask_dcl => todo!(),
        Rule::bit_value => todo!(),
        Rule::annotation_dcl => todo!(),
        Rule::annotation_header => todo!(),
        Rule::annotation_body => todo!(),
        Rule::annotation_member => todo!(),
        Rule::annotation_member_type => todo!(),
        Rule::any_const_type => todo!(),
        Rule::annotation_appl => todo!(),
        Rule::annotation_appl_params => todo!(),
        Rule::annotation_appl_param => todo!(),
    }
}

fn specification(pair: IdlPair, writer: &mut String) {
    for definition in pair.into_inner() {
        generate_rust_source(definition, writer);
    }
}

fn definition(pair: IdlPair, writer: &mut String) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
    )
}

fn module_dcl(pair: IdlPair, writer: &mut String) {
    let inner_pairs = pair.into_inner();
    let identifier = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::identifier)
        .expect("Must have an identifier according to the grammar");
    writer.push_str("pub mod ");
    generate_rust_source(identifier, writer);
    writer.push('{');

    for definition in inner_pairs
        .clone()
        .filter(|p| p.as_rule() == Rule::definition)
    {
        generate_rust_source(definition, writer);
    }

    writer.push('}');
}

fn type_dcl(pair: IdlPair, writer: &mut String) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
    )
}

fn constr_type_dcl(pair: IdlPair, writer: &mut String) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
    )
}

fn struct_dcl(pair: IdlPair, writer: &mut String) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
    )
}

fn struct_def(pair: IdlPair, writer: &mut String) {
    let inner_pairs = pair.into_inner();
    let identifier = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::identifier)
        .expect("Identifier must exist according to the grammar");

    // Collect annotations for the struct
    let mut annotations = Vec::new();
    for annotation_appl in inner_pairs
        .clone()
        .filter(|p| p.as_rule() == Rule::annotation_appl)
    {
        let inner_pairs = annotation_appl.into_inner();
        let scoped_name = inner_pairs
            .clone()
            .find(|p| p.as_rule() == Rule::scoped_name)
            .expect("Must have a scoped name according to the grammar");
        let identifier = scoped_name
            .into_inner()
            .next()
            .expect("Must have an identifier according to the grammar");
        let params: Vec<String> = inner_pairs
            .clone()
            .filter(|p| p.as_rule() == Rule::annotation_appl_param)
            .map(|p| p.as_str().to_string())
            .collect();
        annotations.push((identifier.as_str().to_string(), params));
    }

    let has_derive = annotations.iter().any(|(ann_name, _)| ann_name == "derive");
    if !has_derive {
        writer.push_str(&format!("#[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]\n"));
    }
    for (ann_name, ann_params) in &annotations {
        match ann_name.as_str() {
            "derive" => {
                println!("{} params: {:?}", ann_name, ann_params);
                if !ann_params.is_empty() {
                    let joined = ann_params.join(", ");
                    writer.push_str(&format!(
                        "#[derive({}, dust_dds::infrastructure::type_support::DdsType)]\n",
                        joined
                    ));
                } else {
                    writer.push_str("#[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]\n");
                }
            },
            "repr" => {
                if let Some(param) = ann_params.first() {
                    let trimmed = strip_quotes(param);
                    writer.push_str(&format!("#[repr({})]\n", trimmed));
                }
            }
            "final" => writer.push_str("#[dust_dds(extensibility = \"final\")]\n"),
            "appendable" => writer.push_str("#[dust_dds(extensibility = \"appendable\")]\n"),
            "mutable" => writer.push_str("#[dust_dds(extensibility = \"mutable\")]\n"),
            _ => println!("Unknown struct annotation: {}", ann_name),
        }
    }

    writer.push_str(&format!("pub struct {} {{", identifier.as_str()));

    for member in inner_pairs.filter(|p| p.as_rule() == Rule::member) {
        let member_pairs = member.into_inner();
        let type_spec = member_pairs
            .clone()
            .find(|p| p.as_rule() == Rule::type_spec)
            .expect("Type spec must exist according to grammar");
        let declarators = member_pairs
            .clone()
            .find(|p| p.as_rule() == Rule::declarators)
            .expect("Declarator must exist according to grammar");

        // Collect member annotations
        let mut member_annotations = Vec::new();
        for annotation_appl in member_pairs
            .clone()
            .filter(|p| p.as_rule() == Rule::annotation_appl)
        {
            let inner_pairs = annotation_appl.into_inner();
            let scoped_name = inner_pairs
                .clone()
                .find(|p| p.as_rule() == Rule::scoped_name)
                .expect("Must have a scoped name according to the grammar");
            let identifier = scoped_name
                .into_inner()
                .next()
                .expect("Must have an identifier according to the grammar");
            let params: Vec<String> = inner_pairs
                .clone()
                .filter(|p| p.as_rule() == Rule::annotation_appl_param)
                .map(|p| p.as_str().to_string())
                .collect();
            member_annotations.push((identifier.as_str().to_string(), params));
        }

        // Handle member-level annotations
        for (ann_name, ann_params) in &member_annotations {
            match ann_name.as_str() {
                "serde_skip" => writer.push_str("#[serde(skip)]\n"),
                "serde_rename" => {
                    if let Some(param) = ann_params.first() {
                        let trimmed = strip_quotes(param);
                        writer.push_str(&format!("#[serde(rename = \"{}\")]\n", trimmed));
                    }
                }
                "key" => writer.push_str("#[dust_dds(key)]\n"),
                "appendable" => writer.push_str("#[dust_dds(appendable)]\n"),
                "mutable" => writer.push_str("#[dust_dds(mutable)]\n"),
                "final" => writer.push_str("#[dust_dds(final)]\n"),
                "hashid" => writer.push_str("#[dust_dds(hashid)]\n"),
                "must_understand" => writer.push_str("#[dust_dds(must_understand)]\n"),
                "Optional" | "optional" => writer.push_str("#[dust_dds(optional)]\n"),
                _ => {}
            }
        }

        for declarator in declarators.into_inner() {
            let array_or_simple_declarator = declarator
                .into_inner()
                .next()
                .expect("Must have an element according to the grammar");
            writer.push_str("pub ");
            match array_or_simple_declarator.as_rule() {
                Rule::array_declarator => {
                    let array_declarator = array_or_simple_declarator.into_inner();
                    let identifier = array_declarator
                        .clone()
                        .find(|p| p.as_rule() == Rule::identifier)
                        .expect("Identifier must exist according to grammar");
                    let fixed_array_size = array_declarator
                        .clone()
                        .find(|p| p.as_rule() == Rule::fixed_array_size)
                        .expect("Fixed array size must exist according to grammar");
                    generate_rust_source(identifier, writer);
                    writer.push(':');
                    writer.push('[');
                    let type_spec_str = type_spec.as_str();
                    let rust_type = map_idl_type(type_spec_str, &member_annotations);
                    writer.push_str(&rust_type);
                    writer.push(';');
                    let size_str = fixed_array_size.as_str();
                    let evaluated_size = evaluate_array_size(size_str);
                    writer.push_str(&evaluated_size);
                    writer.push(']');
                }
                Rule::simple_declarator => {
                    generate_rust_source(array_or_simple_declarator, writer);
                    writer.push(':');
                    let type_spec_str = type_spec.as_str();
                    println!("struct_def: type_spec_str = '{}'", type_spec_str);
                    // Handle sequence and bounded string types
                    let rust_type = if type_spec_str.starts_with("sequence<") {
                        let inner_type = type_spec_str
                            .strip_prefix("sequence<")
                            .and_then(|s| s.strip_suffix('>'))
                            .map(|s| {
                                let trimmed = s.split(',').next().unwrap_or(s).trim();
                                if trimmed.starts_with("sequence<") && trimmed.ends_with(">") {
                                    println!("Ends with >");
                                    let inner_inner_type = trimmed
                                        .strip_prefix("sequence<")
                                        .and_then(|s| s.strip_suffix('>'))
                                        .map(|s| s.split(',').next().unwrap_or(s).trim())
                                        .unwrap_or(trimmed);
                                    
                                    println!("Trimmed sequence: {:?}", trimmed);
                                    println!("Inner_inner type sequence: {:?}", inner_inner_type);
                                    println!("Inner_inner type sequence strip: {:?}", inner_inner_type.strip_prefix("sequence<"));
                                    format!("Vec<{}>", map_idl_type(inner_inner_type, &member_annotations))
                                } else if trimmed.starts_with("sequence<") && trimmed.ends_with("") {
                                    println!("Ends with nothing");
                                    let inner_inner_type = trimmed
                                        .strip_prefix("sequence<")
                                        .unwrap_or(trimmed);
                                    
                                    println!("Trimmed sequence: {:?}", trimmed);
                                    println!("Inner_inner type sequence: {:?}", inner_inner_type);
                                    println!("Inner_inner type sequence strip: {:?}", inner_inner_type.strip_prefix("sequence<"));
                                    format!("Vec<{}>", map_idl_type(inner_inner_type, &member_annotations))
                                } else {
                                    println!("Trimmed sequence: {:?}", trimmed);
                                    map_idl_type(trimmed, &member_annotations)
                                }
                            })
                            .unwrap_or_else(|| map_idl_type(type_spec_str, &member_annotations));
                        format!("Vec<{}>", inner_type)
                    } else if type_spec_str.starts_with("string<") || type_spec_str.starts_with("wstring<") {
                        "String".to_string()
                    } else {
                        map_idl_type(type_spec_str, &member_annotations)
                    };
                    println!("struct_def: rust_type = '{}'", rust_type);
                    writer.push_str(&rust_type);
                }
                _ => panic!("Not allowed by the grammar"),
            }
            writer.push(',');
        }
    }

    writer.push_str("}\n");
}


fn enum_dcl(pair: IdlPair, writer: &mut String) {
    let inner_pairs = pair.into_inner();
    let identifier = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::identifier)
        .expect("Must have an identifier according to the grammar");
    writer.push_str("#[derive(Debug)]\n");
    writer.push_str("pub enum ");
    generate_rust_source(identifier, writer);
    writer.push('{');

    for enumerator in inner_pairs
        .clone()
        .filter(|p| p.as_rule() == Rule::enumerator)
    {
        generate_rust_source(enumerator, writer);
        writer.push(',');
    }

    writer.push('}');
}

fn enumerator(pair: IdlPair, writer: &mut String) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
    )
}

fn member(pair: IdlPair, writer: &mut String) {
    let inner_pairs = pair.into_inner();

    let type_spec = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::type_spec)
        .expect("Type spec must exist according to grammar");
    let declarators = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::declarators)
        .expect("Declarator must exist according to grammar");

    for annotation_appl in inner_pairs
        .clone()
        .filter(|p| p.as_rule() == Rule::annotation_appl)
    {
        let inner_pairs = annotation_appl.into_inner();

        let scoped_name = inner_pairs
            .clone()
            .find(|p| p.as_rule() == Rule::scoped_name)
            .expect("Must have a scoped name according to the grammar");

        let identifier = scoped_name
            .into_inner()
            .next()
            .expect("Must have an identifier according to the grammar");

        if identifier.as_str() == "key" {
            writer.push_str("#[dust_dds(key)]");
        }
    }

    for declarator in declarators.into_inner() {
        let array_or_simple_declarator = declarator
            .into_inner()
            .next()
            .expect("Must have an element according to the grammar");
        writer.push_str("pub ");
        match array_or_simple_declarator.as_rule() {
            Rule::array_declarator => {
                let array_declarator = array_or_simple_declarator.into_inner();
                let identifier = array_declarator
                    .clone()
                    .find(|p| p.as_rule() == Rule::identifier)
                    .expect("Identifier must exist according to grammar");
                // TODO: Only single array supported
                let fixed_array_size = array_declarator
                    .clone()
                    .find(|p| p.as_rule() == Rule::fixed_array_size)
                    .expect("Identifier must exist according to grammar");
                generate_rust_source(identifier, writer);
                writer.push(':');
                writer.push('[');
                generate_rust_source(type_spec.clone(), writer);
                writer.push(';');
                generate_rust_source(fixed_array_size, writer);
                writer.push(']');
            }
            Rule::simple_declarator => {
                generate_rust_source(array_or_simple_declarator, writer);
                writer.push(':');
                generate_rust_source(type_spec.clone(), writer);
            }
            _ => panic!("Not allowed by the grammar"),
        }
        writer.push(',');
    }
}

fn declarators(pair: IdlPair, writer: &mut String) {
    for declarator in pair.into_inner() {
        generate_rust_source(declarator, writer);
    }
}

fn interface_dcl(pair: IdlPair, writer: &mut String) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
    )
}

fn interface_def(pair: IdlPair, writer: &mut String) {
    let inner_pairs = pair.into_inner();

    let interface_header = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::interface_header)
        .expect("Must have an interface_header according to grammar");

    let interface_body = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::interface_body)
        .expect("Must have an interface_body according to grammar");

    generate_rust_source(interface_header, writer);
    writer.push('{');
    generate_rust_source(interface_body, writer);
    writer.push_str("}\n");
}

fn interface_header(pair: IdlPair, writer: &mut String) {
    let inner_pairs = pair.into_inner();

    let interface_kind = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::interface_kind)
        .expect("Must have an interface_kind according to grammar");

    let identifier = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::identifier)
        .expect("Must have an identifier according to grammar");

    let interface_inheritance_spec = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::interface_inheritance_spec);

    generate_rust_source(interface_kind, writer);
    generate_rust_source(identifier, writer);

    if let Some(interface_inheritance_spec) = interface_inheritance_spec {
        generate_rust_source(interface_inheritance_spec, writer);
    }
}

fn interface_kind(pair: IdlPair, writer: &mut String) {
    match pair.as_str() {
        "interface" | "abstract interface" => writer.push_str("pub trait "),
        "local interface" => writer.push_str("trait "),
        _ => panic!("Invalid string according to grammar"),
    }
}

fn interface_body(pair: IdlPair, writer: &mut String) {
    for export in pair.into_inner().filter(|p| p.as_rule() == Rule::export) {
        generate_rust_source(export, writer);
    }
}

fn export(pair: IdlPair, writer: &mut String) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
    )
}

fn op_dcl(pair: IdlPair, writer: &mut String) {
    let inner_pairs = pair.into_inner();
    let identifier = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::identifier)
        .expect("Must have an identifier according to the grammar");
    let parameter_dcls = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::parameter_dcls)
        .expect("Must have a parameter_dcls according to the grammar");
    let op_type_spec = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::op_type_spec)
        .expect("Must have an op_type_spec according to the grammar");
    writer.push_str("fn ");
    generate_rust_source(identifier, writer);
    writer.push('(');
    generate_rust_source(parameter_dcls, writer);
    writer.push(')');
    generate_rust_source(op_type_spec, writer);
    writer.push_str(";\n");
}

fn op_type_spec(pair: IdlPair, writer: &mut String) {
    if let Some(type_spec) = pair.into_inner().find(|p| p.as_rule() == Rule::type_spec) {
        writer.push_str("->");
        generate_rust_source(type_spec, writer);
    }
}

fn parameter_dcls(pair: IdlPair, writer: &mut String) {
    for param_dcl in pair.into_inner().filter(|p| p.as_rule() == Rule::param_dcl) {
        generate_rust_source(param_dcl, writer);
    }
}

fn param_dcl(pair: IdlPair, writer: &mut String) {
    let inner_pairs = pair.into_inner();
    let param_attribute = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::param_attribute)
        .expect("Must have a param_attribute according to the grammar");
    let type_spec = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::type_spec)
        .expect("Must have a type_spec according to the grammar");
    let simple_declarator = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::simple_declarator)
        .expect("Must have a simple_declarator according to the grammar");

    generate_rust_source(simple_declarator, writer);
    writer.push(':');
    generate_rust_source(param_attribute, writer);
    generate_rust_source(type_spec, writer);
    writer.push(',');
}

fn param_attribute(pair: IdlPair, writer: &mut String) {
    match pair.as_str() {
        "inout" | "out" => writer.push_str("&mut "),
        "in" => writer.push('&'),
        _ => panic!("Invalid option by grammar"),
    }
}

fn simple_declarator(pair: IdlPair, writer: &mut String) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
    )
}

fn typedef_dcl(pair: IdlPair, writer: &mut String) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
    )
}

fn type_declarator(pair: IdlPair, writer: &mut String) {
    let inner_pairs = pair.into_inner();
    let type_spec = inner_pairs.clone().find(|p| {
        p.as_rule() == Rule::template_type_spec
            || p.as_rule() == Rule::constr_type_dcl
            || p.as_rule() == Rule::simple_type_spec
    }).expect("template_type_spec, constr_type_dcl or simple_type_spec must exist according to grammar");
    let any_declarators = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::any_declarators)
        .expect("Must have any_declarators according to grammar");
    for any_declarator in any_declarators.into_inner() {
        writer.push_str("pub type ");
        generate_rust_source(any_declarator, writer);
        writer.push('=');
        generate_rust_source(type_spec.clone(), writer);
        writer.push_str(";\n");
    }
}

fn any_declarator(pair: IdlPair, writer: &mut String) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
    )
}

fn identifier(pair: IdlPair, writer: &mut String) {
    writer.push_str(pair.as_str());
}

fn type_spec(pair: IdlPair, writer: &mut String) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
    )
}

fn simple_type_spec(pair: IdlPair, writer: &mut String) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
    )
}

fn base_type_spec(pair: IdlPair, writer: &mut String) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
    )
}

fn floating_pt_type(pair: IdlPair, writer: &mut String) {
    match pair.as_str() {
        "float" => writer.push_str("f32"),
        "double" => writer.push_str("f64"),
        "long double" => unimplemented!("long double not supported in Rust"),
        _ => panic!("Invalid option by grammar"),
    }
}

fn integer_type(pair: IdlPair, writer: &mut String) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
    )
}

fn signed_tiny_int(_pair: IdlPair, writer: &mut String) {
    writer.push_str("i8");
}

fn signed_int(pair: IdlPair, writer: &mut String) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
    )
}

fn signed_short_int(_pair: IdlPair, writer: &mut String) {
    writer.push_str("i16");
}

fn signed_long_int(_pair: IdlPair, writer: &mut String) {
    writer.push_str("i32");
}

fn signed_longlong_int(_pair: IdlPair, writer: &mut String) {
    writer.push_str("i64");
}

fn unsigned_tiny_int(_pair: IdlPair, writer: &mut String) {
    writer.push_str("u8");
}

fn unsigned_int(pair: IdlPair, writer: &mut String) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
    )
}

fn unsigned_short_int(_pair: IdlPair, writer: &mut String) {
    writer.push_str("u16");
}

fn unsigned_long_int(_pair: IdlPair, writer: &mut String) {
    writer.push_str("u32");
}

fn unsigned_longlong_int(_pair: IdlPair, writer: &mut String) {
    writer.push_str("u64");
}

fn octet_type(_pair: IdlPair, writer: &mut String) {
    writer.push_str("u8");
}

fn template_type_spec(pair: IdlPair, writer: &mut String) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
    )
}

fn sequence_type(pair: IdlPair, writer: &mut String) {
    let inner_pairs = pair.into_inner();

    let type_spec = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::type_spec)
        .expect("Must have a type_spec according to the grammar");

    writer.push_str("Vec<");
    generate_rust_source(type_spec, writer);
    writer.push('>');
}

fn string_type(_pair: IdlPair, writer: &mut String) {
    writer.push_str("String");
}

fn wide_string_type(_pair: IdlPair, writer: &mut String) {
    writer.push_str("String");
}

fn fixed_array_size(pair: IdlPair, writer: &mut String) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
    )
}

fn positive_int_const(pair: IdlPair, writer: &mut String) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
    )
}

fn const_expr(pair: IdlPair, writer: &mut String) {
    writer.push_str(pair.as_str());
}

fn scoped_name(pair: IdlPair, writer: &mut String) {
    let identifier = pair
        .into_inner()
        .next()
        .expect("Must have an identifier according to the grammar");

    writer.push_str(identifier.as_str());
}

fn const_dcl(pair: IdlPair, writer: &mut String) {
    let inner_pairs = pair.into_inner();
    let identifier = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::identifier)
        .expect("Must have an identifier according to the grammar");

    let const_type = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::const_type)
        .expect("Must have a const_type according to the grammar");

    let const_expr = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::const_expr)
        .expect("Must have a const_expr according to the grammar");

    writer.push_str("pub const ");
    generate_rust_source(identifier, writer);
    writer.push(':');
    generate_rust_source(const_type, writer);
    writer.push('=');
    generate_rust_source(const_expr, writer);
    writer.push_str(";\n");
}

fn const_type(pair: IdlPair, writer: &mut String) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to grammar"),
        writer,
    );
}

fn char_type(_pair: IdlPair, writer: &mut String) {
    writer.push_str("char");
}

fn wide_char_type(_pair: IdlPair, writer: &mut String) {
    writer.push_str("char");
}

fn boolean(_pair: IdlPair, writer: &mut String) {
    writer.push_str("bool");
}

fn evaluate_array_size(size_str: &str) -> String {
    let size_str = size_str.trim().trim_matches(|c| c == '[' || c == ']');
    println!("Evaluating array size: '{}'", size_str);
    size_str.to_string() // Return the expression unchanged
}

fn strip_quotes(s: &str) -> &str {
    s.strip_prefix('"').and_then(|s| s.strip_suffix('"')).unwrap_or(s)
}

fn map_idl_type(idl_type: &str, annotations: &Vec<(String, Vec<String>)>) -> String {
    for (name, parameters) in annotations {
        if name == "rust_type" {
            if let Some(value) = parameters.first() {
                return strip_quotes(value).to_string();
            }
        }
    }

    let idl_type = idl_type.trim();

    if idl_type.is_empty() {
        return "/* empty type */".to_string();
    }

    if idl_type.starts_with("sequence<") && idl_type.ends_with('>') {
        let inner_type = &idl_type["sequence<".len()..idl_type.len() - 1].trim();
        if inner_type.is_empty() {
            return "Vec<()>".to_string(); // or handle as error
        }
        return format!("Vec<{}>", map_idl_type(inner_type, annotations));
    }

    if let Some(pos) = idl_type.find('[') {
        if idl_type.ends_with(']') {
            let base = &idl_type[..pos].trim();
            let size_str = &idl_type[pos + 1..idl_type.len() - 1];
            if let Ok(size) = size_str.parse::<usize>() {
                return format!("[{}; {}]", map_idl_type(base, annotations), size);
            } else {
                return "/* unknown fixed array size */".to_string();
            }
        }
    }

    match idl_type {
        "long" => "i32".to_string(),
        "unsigned long" => "u32".to_string(),
        "short" => "i16".to_string(),
        "unsigned short" => "u16".to_string(),
        "int8" => "i8".to_string(),
        "int16" => "i16".to_string(),
        "int32" => "i32".to_string(),
        "int64" => "i64".to_string(),
        "uint8" => "u8".to_string(),
        "uint16" => "u16".to_string(),
        "uint32" => "u32".to_string(),
        "uint64" => "u64".to_string(),
        "long long" => "i64".to_string(),
        "unsigned long long" => "u64".to_string(),
        "string" => "String".to_string(),
        "wstring" => "String".to_string(),
        "octet" => "u8".to_string(),
        "float" => "f32".to_string(),
        "double" => "f64".to_string(),
        "boolean" => "bool".to_string(),
        "char" => "char".to_string(),
        "wchar" => "char".to_string(),
        _ => {
            eprintln!("Warning: unmapped IDL type '{}'", idl_type);
            idl_type.to_string()
        }
    }
}

pub fn generate_rust_def(s: &StructDef) -> String {
    let mut code = String::new();

    // Handle struct-level annotations (derive, repr)
    for ann in &s.annotations {
        match ann.name.as_str() {
            "derive" => {
                if let Some(value) = ann.parameters.first() {
                    let trimmed = strip_quotes(value);
                    code.push_str(&format!("#[derive({})]\n", trimmed));
                }
            },
            "repr" => {
                if let Some(param) = ann.parameters.first() {
                    let trimmed = strip_quotes(param);
                    code.push_str(&format!("#[repr({})]\n", trimmed));
                }
            },
            "final" => code.push_str("#[dust_dds(extensibility = \"final\")]\n"),
            "appendable" => code.push_str("#[dust_dds(extensibility = \"appendable\")]\n"),
            "mutable" => code.push_str("#[dust_dds(extensibility = \"mutable\")]\n"),
            _ => {}
        }
    }

    if !s.annotations.iter().any(|ann| ann.name == "derive") {
        code.push_str("#[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]\n");
    }

    code.push_str(&format!("pub struct {} {{\n", s.name));

    for member in &s.members {
        // Member-level annotations
        for ann in &member.annotations {
            match ann.name.as_str() {
                "serde_skip" => code.push_str("#[serde(skip)]\n"),
                "serde_rename" => {
                    if let Some(param) = ann.parameters.first() {
                        let trimmed = strip_quotes(param);
                        code.push_str(&format!("#[serde(rename = \"{}\")]\n", trimmed));
                    }
                },
                "key" => {
                    code.push_str("#[dust_dds(key)]\n");
                },
                "appendable" => {
                    code.push_str("#[dust_dds(extensibility = appendable)]\n");
                },
                "mutable" => {
                    code.push_str("#[dust_dds(extensibility = mutable)]\n");
                },
                "final" => {
                    code.push_str("#[dust_dds(extensibility = final)]\n");
                },
                "hashid" => {
                    code.push_str("#[dust_dds(hashid)]\n");
                },
                "must_understand" => {
                    code.push_str("#[dust_dds(must_understand)]\n");
                },
                "Optional" | "optional" => {
                    code.push_str("#[dust_dds(optional)]\n");
                },
                _ => {}
            }
        }

        let member_annotations: Vec<(String, Vec<String>)> = member
            .annotations
            .iter()
            .map(|ann| (ann.name.clone(), ann.parameters.clone()))
            .collect();

        let field_type = if member.idl_type.starts_with("sequence<") {
            // Extract inner type from sequence<T> or sequence<T, N>
            let inner_type = member
                .idl_type
                .strip_prefix("sequence<")
                .and_then(|s| s.strip_suffix('>'))
                .map(|s| {
                    // Handle bounds (e.g., sequence<short, 128>) by taking the type before the comma
                    let trimmed = s.split(',').next().unwrap_or(s).trim();
                    if trimmed.starts_with("sequence<") {
                        // Handle nested sequences
                        let inner_inner_type = trimmed
                            .strip_prefix("sequence<")
                            .and_then(|s| s.strip_suffix('>'))
                            .map(|s| s.split(',').next().unwrap_or(s).trim())
                            .unwrap_or(trimmed);
                        format!("Vec<{}>", map_idl_type(inner_inner_type, &member_annotations))
                    } else {
                        map_idl_type(trimmed, &member_annotations)
                    }
                })
                .unwrap_or_else(|| map_idl_type(&member.idl_type, &member_annotations));
            format!("Vec<{}>", inner_type)
        } else if member.idl_type.starts_with("string<") || member.idl_type.starts_with("wstring<") {
            // Handle bounded strings
            "String".to_string()
        } else {
            // Handle non-sequence, non-bounded types
            let rust_type = map_idl_type(&member.idl_type, &member_annotations);
            match &member.array_size {
                Some(size) => format!("[{}; {}]", rust_type, size),
                None => rust_type,
            }
        };
            
        code.push_str(&format!("pub {}: {},", member.name, field_type));
    }

    code.push_str("}\n");
    code
}

pub fn generate_rust_code(
    parsed_idl_result: Result<(Vec<StructDef>, IdlPair), Box<pest::error::Error<Rule>>>,
    output: &mut String,
) -> Result<(), Box<pest::error::Error<Rule>>> {
    match parsed_idl_result {
        Ok((structs, idl_pair)) => {
            // Check if only structs are present
            let only_structs = idl_pair
                .clone()
                .into_inner()
                .all(|def| {
                    if def.as_rule() == Rule::definition {
                        let inner = def.into_inner().next().unwrap();
                        inner.as_rule() == Rule::type_dcl
                            && inner
                                .clone()
                                .into_inner()
                                .next()
                                .unwrap()
                                .as_rule()
                                == Rule::constr_type_dcl
                            && inner
                                .into_inner()
                                .next()
                                .unwrap()
                                .as_rule()
                                == Rule::struct_dcl
                    } else {
                        false
                    }
                });

            if only_structs {
                // Use generate_rust_def for efficiency if only structs
                println!("OnlyStructs. Using generate_rust_def");
                for struct_def in structs {
                    output.push_str(&generate_rust_def(&struct_def));
                }
            } else {
                // Use generate_rust_source for full IDL processing
                println!("Using generate_rust_source");
                generate_rust_source(idl_pair, output);
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    pub use crate::parse_idl;

    use super::*;

    #[test]
    fn parse_struct() {
        let mut out = String::new();
        let p = parse_idl(
            "@derive(\"Debug, Clone\")
            struct MyStruct {
            long a;
            long long b, c;
            octet xary[32], yary[64];
        };",
        Rule::struct_def).unwrap();
        let _ = generate_rust_code(Ok(p), &mut out);
        println!("RESULT: {}", out);
        assert_eq!(
            "#[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]\npub struct MyStruct {pub a:i32,pub b:i64,pub c:i64,pub xary:[u8;32],pub yary:[u8;64],}\n",
            &out
        );
    }

    #[test]
    fn parse_appendable_struct() {
        let mut out = String::new();
        let p = parse_idl("@derive(\"Debug, Clone\") @appendable struct MyStruct { long a; };", Rule::struct_def).unwrap();
        let _ = generate_rust_code(Ok(p), &mut out);
        println!("RESULT: {}", out);
        assert_eq!(
            &out,
            "#[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]\n#[dust_dds(extensibility = \"appendable\")]\npub struct MyStruct {pub a:i32,}\n",
        );
    }

    #[test]
    fn parse_member_with_key() {
        let mut out = String::new();
        let p = parse_idl("@key long a;", Rule::member).unwrap();
        let _ = generate_rust_code(Ok(p), &mut out);
        println!("RESULT: {}", out);
        assert_eq!("#[dust_dds(key)]pub a:i32,", &out);
    }

    #[test]
    fn parse_member_sequence_type() {
        let mut out = String::new();
        let p = parse_idl("sequence<octet> a;", Rule::member).unwrap();
        let _ = generate_rust_code(Ok(p), &mut out);
        println!("RESULT: {}", out);
        assert_eq!("pub a:Vec<u8>,", &out);
    }

    #[test]
    fn parse_member_sequence_of_sequence_type() {
        let mut out = String::new();
        let p = parse_idl("sequence<sequence<octet>> a;", Rule::member).unwrap();
        let _ = generate_rust_code(Ok(p), &mut out);
        println!("RESULT: {}", out);
        assert_eq!("pub a:Vec<Vec<u8>>,", &out);
    }

    #[test]
    fn parse_enum() {
        let mut out = String::new();
        let p = parse_idl(
            "enum Suits { Spades, Hearts, Diamonds, Clubs };", Rule::enum_dcl).unwrap();
        let _ = generate_rust_code(Ok(p), &mut out);
        println!("RESULT: {}", out);
        assert_eq!(
            "#[derive(Debug)]\npub enum Suits{Spades,Hearts,Diamonds,Clubs,}",
            &out
        );
    }

    #[test]
    fn parse_const_with_literals() {
        let mut out = String::new();
        let p = parse_idl(r#"const string a = 'a';"#, Rule::specification).unwrap();

        let _ = generate_rust_code(Ok(p), &mut out);
        assert_eq!("pub const a:String='a';\n", &out);
    }

    #[test]
    fn parse_typedef() {
        let mut out = String::new();

        let p = parse_idl(r#"typedef long Name;"#, Rule::typedef_dcl).unwrap();

        let _ = generate_rust_code(Ok(p), &mut out);
        assert_eq!("pub type Name=i32;\n", &out);
    }

    #[test]
    fn parse_interface_export() {
        let mut out = String::new();

        let p = parse_idl(r#"short op(out string s);"#, Rule::export).unwrap();

        let _ = generate_rust_code(Ok(p), &mut out);
        assert_eq!("fn op(s:&mut String,)->i16;\n", &out);
    }

    #[test]
    fn parse_interface() {
        let mut out = String::new();

        let p = parse_idl(
            "interface MyInterface {
                void op(out string s, inout short a, in char u);
                unsigned short sum(in unsigned short a, in unsigned short b);
            };",
            Rule::interface_def).unwrap();

        let _ = generate_rust_code(Ok(p), &mut out);
        assert_eq!(
            "pub trait MyInterface{fn op(s:&mut String,a:&mut i16,u:&char,);\nfn sum(a:&u16,b:&u16,)->u16;\n}\n",
            &out
        );
    }
}
