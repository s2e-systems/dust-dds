use std::collections::HashMap;

use crate::parser::{IdlPair, Rule};

pub fn generate_rust_source<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
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
        Rule::identifier => identifier(pair, writer, define_table),
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
        Rule::specification => specification(pair, writer, define_table),
        Rule::definition => definition(pair, writer, define_table),
        Rule::module_dcl => module_dcl(pair, writer, define_table),
        Rule::scoped_name => scoped_name(pair, writer, define_table),
        Rule::const_dcl => const_dcl(pair, writer, define_table),
        Rule::const_type => const_type(pair, writer, define_table),
        Rule::const_expr => const_expr(pair, writer, define_table),
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
        Rule::positive_int_const => positive_int_const(pair, writer, define_table),
        Rule::type_dcl => type_dcl(pair, writer, define_table),
        Rule::type_spec => type_spec(pair, writer, define_table),
        Rule::simple_type_spec => simple_type_spec(pair, writer, define_table),
        Rule::base_type_spec => base_type_spec(pair, writer, define_table),
        Rule::floating_pt_type => floating_pt_type(pair, writer, define_table),
        Rule::integer_type => integer_type(pair, writer, define_table),
        Rule::signed_tiny_int => signed_tiny_int(pair, writer, define_table),
        Rule::signed_int => signed_int(pair, writer, define_table),
        Rule::signed_short_int => signed_short_int(pair, writer, define_table),
        Rule::signed_long_int => signed_long_int(pair, writer, define_table),
        Rule::signed_longlong_int => signed_longlong_int(pair, writer, define_table),
        Rule::unsigned_tiny_int => unsigned_tiny_int(pair, writer, define_table),
        Rule::unsigned_int => unsigned_int(pair, writer, define_table),
        Rule::unsigned_short_int => unsigned_short_int(pair, writer, define_table),
        Rule::unsigned_long_int => unsigned_long_int(pair, writer, define_table),
        Rule::unsigned_longlong_int => unsigned_longlong_int(pair, writer, define_table),
        Rule::char_type => char_type(pair, writer, define_table),
        Rule::wide_char_type => wide_char_type(pair, writer, define_table),
        Rule::boolean_type => boolean(pair, writer, define_table),
        Rule::octet_type => octet_type(pair, writer, define_table),
        Rule::template_type_spec => template_type_spec(pair, writer, define_table),
        Rule::sequence_type => sequence_type(pair, writer, define_table),
        Rule::string_type => string_type(pair, writer, define_table),
        Rule::wide_string_type => wide_string_type(pair, writer, define_table),
        Rule::fixed_pt_type => unimplemented!("Fixed point not supported in Rust mapping"),
        Rule::fixed_pt_const_type => unimplemented!("Fixed point not supported in Rust mapping"),
        Rule::constr_type_dcl => constr_type_dcl(pair, writer, define_table),
        Rule::struct_dcl => struct_dcl(pair, writer, define_table),
        Rule::struct_def => struct_def(pair, writer, define_table),
        Rule::member => member(pair, writer, define_table),
        Rule::struct_forward_dcl => (), // Forward declarations are irrelevant in Rust mapping
        Rule::union_dcl => todo!(),
        Rule::union_def => todo!(),
        Rule::switch_type_spec => todo!(),
        Rule::switch_body => todo!(),
        Rule::case => todo!(),
        Rule::case_label => todo!(),
        Rule::element_spec => todo!(),
        Rule::union_forward_dcl => (), // Forward declarations are irrelevant in Rust mapping
        Rule::enum_dcl => enum_dcl(pair, writer, define_table),
        Rule::enumerator => enumerator(pair, writer, define_table),
        Rule::array_declarator => todo!(),
        Rule::fixed_array_size => fixed_array_size(pair, writer, define_table),
        Rule::native_dcl => todo!(),
        Rule::simple_declarator => simple_declarator(pair, writer, define_table),
        Rule::typedef_dcl => typedef_dcl(pair, writer, define_table),
        Rule::type_declarator => type_declarator(pair, writer, define_table),
        Rule::any_declarators => (), // Handled inside typedef_dcl
        Rule::any_declarator => any_declarator(pair, writer, define_table),
        Rule::declarators => declarators(pair, writer, define_table),
        Rule::declarator => todo!(),
        Rule::any_type => todo!(),
        Rule::except_dcl => todo!(),
        Rule::interface_dcl => interface_dcl(pair, writer, define_table),
        Rule::interface_def => interface_def(pair, writer, define_table),
        Rule::interface_forward_dcl => todo!(), // Forward declarations are irrelevant in Rust mapping
        Rule::interface_header => interface_header(pair, writer, define_table),
        Rule::interface_kind => interface_kind(pair, writer, define_table),
        Rule::interface_inheritance_spec => todo!(),
        Rule::interface_name => todo!(),
        Rule::interface_body => interface_body(pair, writer, define_table),
        Rule::export => export(pair, writer, define_table),
        Rule::op_dcl => op_dcl(pair, writer, define_table),
        Rule::op_type_spec => op_type_spec(pair, writer, define_table),
        Rule::parameter_dcls => parameter_dcls(pair, writer, define_table),
        Rule::param_dcl => param_dcl(pair, writer, define_table),
        Rule::param_attribute => param_attribute(pair, writer, define_table),
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
        Rule::annotation_appl => annotation_appl(pair, writer, define_table),
        Rule::annotation_appl_params => todo!(),
        Rule::annotation_appl_param => todo!(),
        Rule::path_spec => todo!(),
        Rule::define_directive => define_directive(pair, writer, define_table),
        Rule::include_directive => todo!(),
        Rule::other_directive => todo!(),
        Rule::define_replacement_list => todo!(),
    }
}

fn specification<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
    for definition in pair.into_inner() {
        generate_rust_source(definition, writer, define_table);
    }
}

fn definition<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
        define_table,
    )
}

fn module_dcl<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
    let inner_pairs = pair.into_inner();
    let identifier = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::identifier)
        .expect("Must have an identifier according to the grammar");
    writer.push_str("pub mod ");
    generate_rust_source(identifier, writer, define_table);
    writer.push('{');

    for definition in inner_pairs
        .clone()
        .filter(|p| p.as_rule() == Rule::definition)
    {
        generate_rust_source(definition, writer, define_table);
    }

    writer.push('}');
}

fn type_dcl<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
        define_table,
    )
}

fn constr_type_dcl<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
        define_table,
    )
}

fn struct_dcl<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
        define_table,
    )
}

fn struct_def<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
    let inner_pairs = pair.into_inner();
    let identifier = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::identifier)
        .expect("Identifier must exist according to the grammar");

    writer.push_str("#[derive(Debug, dust_dds::topic_definition::type_support::DdsType)]\n");
    writer.push_str("pub struct ");
    generate_rust_source(identifier, writer, define_table);

    writer.push_str(" {");

    for member in inner_pairs.filter(|p| p.as_rule() == Rule::member) {
        generate_rust_source(member, writer, define_table);
    }

    writer.push_str("}\n");
}

fn enum_dcl<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
    let inner_pairs = pair.into_inner();
    let identifier = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::identifier)
        .expect("Must have an identifier according to the grammar");
    writer.push_str("#[derive(Debug)]\n");
    writer.push_str("pub enum ");
    generate_rust_source(identifier, writer, define_table);
    writer.push('{');

    for enumerator in inner_pairs
        .clone()
        .filter(|p| p.as_rule() == Rule::enumerator)
    {
        generate_rust_source(enumerator, writer, define_table);
        writer.push(',');
    }

    writer.push('}');
}

fn enumerator<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
        define_table,
    )
}

fn member<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
    let inner_pairs = pair.into_inner();

    let type_spec = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::type_spec)
        .expect("Declarator must exist according to grammar");
    let declarators = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::declarators)
        .expect("Declarator must exist according to grammar");

    for annotation_appl in inner_pairs
        .clone()
        .filter(|p| p.as_rule() == Rule::annotation_appl)
    {
        generate_rust_source(annotation_appl, writer, define_table);
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
                generate_rust_source(identifier, writer, define_table);
                writer.push(':');
                writer.push('[');
                generate_rust_source(type_spec.clone(), writer, define_table);
                writer.push(';');
                generate_rust_source(fixed_array_size, writer, define_table);
                writer.push(']');
            }
            Rule::simple_declarator => {
                generate_rust_source(array_or_simple_declarator, writer, define_table);
                writer.push(':');
                generate_rust_source(type_spec.clone(), writer, define_table);
            }
            _ => panic!("Not allowed by the grammar"),
        }
        writer.push(',');
    }
}

fn declarators<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
    for declarator in pair.into_inner() {
        generate_rust_source(declarator, writer, define_table);
    }
}

fn interface_dcl<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
        define_table,
    )
}

fn interface_def<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
    let inner_pairs = pair.into_inner();

    let interface_header = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::interface_header)
        .expect("Must have an interface_header according to grammar");

    let interface_body = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::interface_body)
        .expect("Must have an interface_body according to grammar");

    generate_rust_source(interface_header, writer, define_table);
    writer.push('{');
    generate_rust_source(interface_body, writer, define_table);
    writer.push_str("}\n");
}

fn interface_header<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
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

    generate_rust_source(interface_kind, writer, define_table);
    generate_rust_source(identifier, writer, define_table);

    if let Some(interface_inheritance_spec) = interface_inheritance_spec {
        generate_rust_source(interface_inheritance_spec, writer, define_table);
    }
}

fn interface_kind<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    _define_table: &mut HashMap<&'a str, &'a str>,
) {
    match pair.as_str() {
        "interface" | "abstract interface" => writer.push_str("pub trait "),
        "local interface" => writer.push_str("trait "),
        _ => panic!("Invalid string according to grammar"),
    }
}

fn interface_body<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
    for export in pair.into_inner().filter(|p| p.as_rule() == Rule::export) {
        generate_rust_source(export, writer, define_table);
    }
}

fn export<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
        define_table,
    )
}

fn op_dcl<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
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
    generate_rust_source(identifier, writer, define_table);
    writer.push('(');
    generate_rust_source(parameter_dcls, writer, define_table);
    writer.push(')');
    generate_rust_source(op_type_spec, writer, define_table);
    writer.push_str(";\n");
}

fn op_type_spec<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
    if let Some(type_spec) = pair.into_inner().find(|p| p.as_rule() == Rule::type_spec) {
        writer.push_str("->");
        generate_rust_source(type_spec, writer, define_table);
    }
}

fn parameter_dcls<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
    for param_dcl in pair.into_inner().filter(|p| p.as_rule() == Rule::param_dcl) {
        generate_rust_source(param_dcl, writer, define_table);
    }
}

fn param_dcl<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
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

    generate_rust_source(simple_declarator, writer, define_table);
    writer.push(':');
    generate_rust_source(param_attribute, writer, define_table);
    generate_rust_source(type_spec, writer, define_table);
    writer.push(',');
}

fn param_attribute<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    _define_table: &mut HashMap<&'a str, &'a str>,
) {
    match pair.as_str() {
        "inout" | "out" => writer.push_str("&mut "),
        "in" => writer.push_str("&"),
        _ => panic!("Invalid option by grammar"),
    }
}

fn simple_declarator<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
        define_table,
    )
}

fn typedef_dcl<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
        define_table,
    )
}

fn type_declarator<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
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
        generate_rust_source(any_declarator, writer, define_table);
        writer.push('=');
        generate_rust_source(type_spec.clone(), writer, define_table);
        writer.push_str(";\n");
    }
}

fn any_declarator<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
        define_table,
    )
}

fn identifier<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    _define_table: &mut HashMap<&'a str, &'a str>,
) {
    writer.push_str(pair.as_str());
}

fn type_spec<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
        define_table,
    )
}

fn simple_type_spec<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
        define_table,
    )
}

fn base_type_spec<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
        define_table,
    )
}

fn floating_pt_type<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    _define_table: &mut HashMap<&'a str, &'a str>,
) {
    match pair.as_str() {
        "float" => writer.push_str("f32"),
        "double" => writer.push_str("f64"),
        "long double" => unimplemented!("long double not supported in Rust"),
        _ => panic!("Invalid option by grammar"),
    }
}

fn integer_type<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
        define_table,
    )
}

fn signed_tiny_int<'a>(
    _pair: IdlPair<'a>,
    writer: &mut String,
    _define_table: &mut HashMap<&'a str, &'a str>,
) {
    writer.push_str("i8");
}

fn signed_int<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
        define_table,
    )
}

fn signed_short_int<'a>(
    _pair: IdlPair<'a>,
    writer: &mut String,
    _define_table: &mut HashMap<&'a str, &'a str>,
) {
    writer.push_str("i16");
}

fn signed_long_int<'a>(
    _pair: IdlPair<'a>,
    writer: &mut String,
    _define_table: &mut HashMap<&'a str, &'a str>,
) {
    writer.push_str("i32");
}

fn signed_longlong_int<'a>(
    _pair: IdlPair<'a>,
    writer: &mut String,
    _define_table: &mut HashMap<&'a str, &'a str>,
) {
    writer.push_str("i64");
}

fn unsigned_tiny_int<'a>(
    _pair: IdlPair<'a>,
    writer: &mut String,
    _define_table: &mut HashMap<&'a str, &'a str>,
) {
    writer.push_str("u8");
}

fn unsigned_int<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
        define_table,
    )
}

fn unsigned_short_int<'a>(
    _pair: IdlPair<'a>,
    writer: &mut String,
    _define_table: &mut HashMap<&'a str, &'a str>,
) {
    writer.push_str("u16");
}

fn unsigned_long_int<'a>(
    _pair: IdlPair<'a>,
    writer: &mut String,
    _define_table: &mut HashMap<&'a str, &'a str>,
) {
    writer.push_str("u32");
}

fn unsigned_longlong_int<'a>(
    _pair: IdlPair<'a>,
    writer: &mut String,
    _define_table: &mut HashMap<&'a str, &'a str>,
) {
    writer.push_str("u64");
}

fn octet_type<'a>(
    _pair: IdlPair<'a>,
    writer: &mut String,
    _define_table: &mut HashMap<&'a str, &'a str>,
) {
    writer.push_str("u8");
}

fn template_type_spec<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
        define_table,
    )
}

fn sequence_type<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
    let inner_pairs = pair.into_inner();

    let type_spec = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::type_spec)
        .expect("Must have a type_spec according to the grammar");

    writer.push_str("Vec<");
    generate_rust_source(type_spec, writer, define_table);
    writer.push('>');
}

fn string_type<'a>(
    _pair: IdlPair<'a>,
    writer: &mut String,
    _define_table: &mut HashMap<&'a str, &'a str>,
) {
    writer.push_str("String");
}

fn wide_string_type<'a>(
    _pair: IdlPair<'a>,
    writer: &mut String,
    _define_table: &mut HashMap<&'a str, &'a str>,
) {
    writer.push_str("String");
}

fn fixed_array_size<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
        define_table,
    )
}

fn positive_int_const<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
        define_table,
    )
}

fn const_expr<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    _define_table: &mut HashMap<&'a str, &'a str>,
) {
    writer.push_str(pair.as_str());
}

fn annotation_appl<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
    let inner_pairs = pair.into_inner();

    let scoped_name = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::scoped_name)
        .expect("Must have a scoped name according to the grammar");

    generate_rust_source(scoped_name, writer, define_table);
}

fn scoped_name<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    _define_table: &mut HashMap<&'a str, &'a str>,
) {
    let identifier = pair
        .into_inner()
        .next()
        .expect("Must have an identifier according to the grammar");

    if identifier.as_str() == "key" {
        writer.push_str("#[dust_dds(key)]");
    }
}

fn const_dcl<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
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
    generate_rust_source(identifier, writer, define_table);
    writer.push_str(":");
    generate_rust_source(const_type, writer, define_table);
    writer.push('=');
    generate_rust_source(const_expr, writer, define_table);
    writer.push_str(";\n");
}

fn const_type<'a>(
    pair: IdlPair<'a>,
    writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to grammar"),
        writer,
        define_table,
    );
}

fn char_type<'a>(
    _pair: IdlPair<'a>,
    writer: &mut String,
    _define_table: &mut HashMap<&'a str, &'a str>,
) {
    writer.push_str("char");
}

fn wide_char_type<'a>(
    _pair: IdlPair<'a>,
    writer: &mut String,
    _define_table: &mut HashMap<&'a str, &'a str>,
) {
    writer.push_str("char");
}

fn boolean<'a>(
    _pair: IdlPair<'a>,
    writer: &mut String,
    _define_table: &mut HashMap<&'a str, &'a str>,
) {
    writer.push_str("bool");
}

fn define_directive<'a>(
    pair: IdlPair<'a>,
    _writer: &mut String,
    define_table: &mut HashMap<&'a str, &'a str>,
) {
    let inner_pairs = pair.into_inner();

    // let identifier = inner_pairs
    //     .clone()
    //     .find(|p| p.as_rule() == Rule::identifier)
    //     .expect("Must have an identifier according to the grammar");

    // let replacement_list = inner_pairs
    //     .clone()
    //     .find(|p| p.as_rule() == Rule::define_replacement_list)
    //     .map(|p| p.as_str())
    //     .unwrap_or_default();

    // define_table.insert(identifier.as_str(), replacement_list);
}

#[cfg(test)]
mod tests {
    use pest::Parser;

    use crate::parser::IdlParser;

    use super::*;

    #[test]
    fn parse_struct() {
        let mut out = String::new();
        let p = IdlParser::parse(
            Rule::struct_def,
            "struct MyStruct {
            long a;
            long long b, c;
            octet xary[32], yary[64];
        };",
        )
        .unwrap()
        .next()
        .unwrap();
        generate_rust_source(p, &mut out, &mut HashMap::new());
        println!("RESULT: {}", out);
        assert_eq!(
            "#[derive(Debug, dust_dds::topic_definition::type_support::DdsType)]\npub struct MyStruct {pub a:i32,pub b:i64,pub c:i64,pub xary:[u8;32],pub yary:[u8;64],}\n",
            &out
        );
    }

    #[test]
    fn parse_member_with_key() {
        let mut out = String::new();
        let p = IdlParser::parse(Rule::member, "@key long a;")
            .unwrap()
            .next()
            .unwrap();
        generate_rust_source(p, &mut out, &mut HashMap::new());
        println!("RESULT: {}", out);
        assert_eq!("#[dust_dds(key)]pub a:i32,", &out);
    }

    #[test]
    fn parse_member_sequence_type() {
        let mut out = String::new();
        let p = IdlParser::parse(Rule::member, "sequence<octet> a;")
            .unwrap()
            .next()
            .unwrap();
        generate_rust_source(p, &mut out, &mut HashMap::new());
        println!("RESULT: {}", out);
        assert_eq!("pub a:Vec<u8>,", &out);
    }

    #[test]
    fn parse_member_sequence_of_sequence_type() {
        let mut out = String::new();
        let p = IdlParser::parse(Rule::member, "sequence<sequence<octet>> a;")
            .unwrap()
            .next()
            .unwrap();
        generate_rust_source(p, &mut out, &mut HashMap::new());
        println!("RESULT: {}", out);
        assert_eq!("pub a:Vec<Vec<u8>>,", &out);
    }

    #[test]
    fn parse_enum() {
        let mut out = String::new();
        let p = IdlParser::parse(
            Rule::enum_dcl,
            "enum Suits { Spades, Hearts, Diamonds, Clubs };",
        )
        .unwrap()
        .next()
        .unwrap();
        generate_rust_source(p, &mut out, &mut HashMap::new());
        println!("RESULT: {}", out);
        assert_eq!(
            "#[derive(Debug)]\npub enum Suits{Spades,Hearts,Diamonds,Clubs,}",
            &out
        );
    }

    #[test]
    fn parse_const_with_literals() {
        let mut out = String::new();
        let p = IdlParser::parse(Rule::specification, r#"const string a = 'a';"#)
            .unwrap()
            .next()
            .unwrap();

        generate_rust_source(p, &mut out, &mut HashMap::new());
        assert_eq!("pub const a:String='a';\n", &out);
    }

    #[test]
    fn parse_typedef() {
        let mut out = String::new();

        let p = IdlParser::parse(Rule::typedef_dcl, r#"typedef long Name;"#)
            .unwrap()
            .next()
            .unwrap();

        generate_rust_source(p, &mut out, &mut HashMap::new());
        assert_eq!("pub type Name=i32;\n", &out);
    }

    #[test]
    fn parse_interface_export() {
        let mut out = String::new();

        let p = IdlParser::parse(Rule::export, r#"short op(out string s);"#)
            .unwrap()
            .next()
            .unwrap();

        generate_rust_source(p, &mut out, &mut HashMap::new());
        assert_eq!("fn op(s:&mut String,)->i16;\n", &out);
    }

    #[test]
    fn parse_interface() {
        let mut out = String::new();

        let p = IdlParser::parse(
            Rule::interface_def,
            "interface MyInterface {
                void op(out string s, inout short a, in char u);
                unsigned short sum(in unsigned short a, in unsigned short b);
            };",
        )
        .unwrap()
        .next()
        .unwrap();

        generate_rust_source(p, &mut out, &mut HashMap::new());
        assert_eq!(
            "pub trait MyInterface{fn op(s:&mut String,a:&mut i16,u:&char,);\nfn sum(a:&u16,b:&u16,)->u16;\n}\n",
            &out
        );
    }

    #[test]
    fn parse_define() {
        let mut out = String::new();

        let p = IdlParser::parse(
            Rule::define_directive,
            "#define LONG i32
            ",
        )
        .unwrap()
        .next()
        .unwrap();

        generate_rust_source(p, &mut out, &mut HashMap::new());
        assert_eq!("type DomainId_t = i32;", &out);
    }
}
