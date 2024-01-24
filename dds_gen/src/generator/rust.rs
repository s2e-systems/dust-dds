use crate::parser::{IdlPair, Rule};

pub fn generate_rust_source(pair: IdlPair, writer: &mut String) {
    match pair.as_rule() {
        Rule::EOI => (),
        Rule::alpha => todo!(),
        Rule::digit => todo!(),
        Rule::octal_digit => todo!(),
        Rule::hex_digit => todo!(),
        Rule::escape => todo!(),
        Rule::octal_escape => todo!(),
        Rule::hex_escape => todo!(),
        Rule::unicode_escape => todo!(),
        Rule::newline => (),
        Rule::WHITESPACE => (),
        Rule::path_spec => todo!(),
        Rule::include_directive => todo!(),
        Rule::other_directive => todo!(),
        Rule::block_comment => (),
        Rule::line_comment => (),
        Rule::COMMENT => (),
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
        Rule::const_dcl => todo!(),
        Rule::const_type => todo!(),
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
        Rule::float => float(pair, writer),
        Rule::double => double(pair, writer),
        Rule::long_double => unimplemented!("long double not support in Rust mapping"),
        Rule::integer_type => integer_type(pair, writer),
        Rule::signed_int => signed_int(pair, writer),
        Rule::signed_short_int => signed_short_int(pair, writer),
        Rule::signed_long_int => signed_long_int(pair, writer),
        Rule::signed_longlong_int => signed_longlong_int(pair, writer),
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
        Rule::fixed_pt_type => todo!(),
        Rule::fixed_pt_const_type => todo!(),
        Rule::constr_type_dcl => constr_type_dcl(pair, writer),
        Rule::struct_dcl => struct_dcl(pair, writer),
        Rule::struct_def => struct_def(pair, writer),
        Rule::member => member(pair, writer),
        Rule::struct_forward_dcl => todo!(),
        Rule::union_dcl => todo!(),
        Rule::union_def => todo!(),
        Rule::switch_type_spec => todo!(),
        Rule::switch_body => todo!(),
        Rule::case => todo!(),
        Rule::case_label => todo!(),
        Rule::element_spec => todo!(),
        Rule::union_forward_dcl => todo!(),
        Rule::enum_dcl => enum_dcl(pair, writer),
        Rule::enumerator => enumerator(pair, writer),
        Rule::array_declarator => todo!(),
        Rule::fixed_array_size => fixed_array_size(pair, writer),
        Rule::native_dcl => todo!(),
        Rule::simple_declarator => simple_declarator(pair, writer),
        Rule::typedef_dcl => todo!(),
        Rule::type_declarator => todo!(),
        Rule::ANY_declarators => todo!(),
        Rule::ANY_declarator => todo!(),
        Rule::declarators => declarators(pair, writer),
        Rule::declarator => todo!(),
        Rule::ANY_type => todo!(),
        Rule::except_dcl => todo!(),
        Rule::interface_dcl => todo!(),
        Rule::interface_def => todo!(),
        Rule::interface_forward_dcl => todo!(),
        Rule::interface_header => todo!(),
        Rule::interface_kind => todo!(),
        Rule::interface_inheritance_spec => todo!(),
        Rule::interface_name => todo!(),
        Rule::interface_body => todo!(),
        Rule::export => todo!(),
        Rule::op_dcl => todo!(),
        Rule::op_type_spec => todo!(),
        Rule::parameter_dcls => todo!(),
        Rule::param_dcl => todo!(),
        Rule::param_attribute => todo!(),
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
        Rule::ANY_const_type => todo!(),
        Rule::annotation_appl => annotation_appl(pair, writer),
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

    writer.push_str("#[derive(Debug, dust_dds::topic_definition::type_support::DdsType)]\n");
    writer.push_str("pub struct ");
    generate_rust_source(identifier, writer);

    writer.push_str(" {");

    for member in inner_pairs.filter(|p| p.as_rule() == Rule::member) {
        generate_rust_source(member, writer);
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
        .expect("Declarator must exist according to grammar");
    let declarators = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::declarators)
        .expect("Declarator must exist according to grammar");

    for annotation_appl in inner_pairs
        .clone()
        .filter(|p| p.as_rule() == Rule::annotation_appl)
    {
        generate_rust_source(annotation_appl, writer);
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

fn simple_declarator(pair: IdlPair, writer: &mut String) {
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
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
    )
}

fn float(_pair: IdlPair, writer: &mut String) {
    writer.push_str("f32");
}

fn double(_pair: IdlPair, writer: &mut String) {
    writer.push_str("f64");
}

fn integer_type(pair: IdlPair, writer: &mut String) {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        writer,
    )
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

fn annotation_appl(pair: IdlPair, writer: &mut String) {
    let inner_pairs = pair.into_inner();

    let scoped_name = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::scoped_name)
        .expect("Must have a scoped name according to the grammar");

    generate_rust_source(scoped_name, writer);
}

fn scoped_name(pair: IdlPair, writer: &mut String) {
    let identifier = pair
        .into_inner()
        .next()
        .expect("Must have an identifier according to the grammar");

    if identifier.as_str() == "key" {
        writer.push_str("#[dust_dds(key)]");
    }
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
        generate_rust_source(p, &mut out);
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
        generate_rust_source(p, &mut out);
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
        generate_rust_source(p, &mut out);
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
        generate_rust_source(p, &mut out);
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
        generate_rust_source(p, &mut out);
        println!("RESULT: {}", out);
        assert_eq!(
            "#[derive(Debug)]\npub enum Suits{Spades,Hearts,Diamonds,Clubs,}",
            &out
        );
    }
}
