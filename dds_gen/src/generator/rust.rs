use super::Context;
use crate::parser::{IdlPair, Rule};
use std::fmt::{Error, Write};

pub fn generate_rust_source<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    match pair.as_rule() {
        Rule::EOI => Ok(()),
        Rule::escape => todo!(),
        Rule::octal_escape => todo!(),
        Rule::hex_escape => todo!(),
        Rule::unicode_escape => todo!(),
        Rule::WHITESPACE => Ok(()),
        Rule::block_comment => Ok(()),
        Rule::line_comment => Ok(()),
        Rule::COMMENT => Ok(()),
        Rule::reserved_keyword => Ok(()),
        Rule::identifier => identifier(pair, ctx),
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
        Rule::specification => specification(pair, ctx),
        Rule::definition => definition(pair, ctx),
        Rule::module_dcl => module_dcl(pair, ctx),
        Rule::scoped_name => scoped_name(pair, ctx),
        Rule::const_dcl => const_dcl(pair, ctx),
        Rule::const_type => const_type(pair, ctx),
        Rule::const_expr => const_expr(pair, ctx),
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
        Rule::positive_int_const => positive_int_const(pair, ctx),
        Rule::type_dcl => type_dcl(pair, ctx),
        Rule::type_spec => type_spec(pair, ctx),
        Rule::simple_type_spec => simple_type_spec(pair, ctx),
        Rule::base_type_spec => base_type_spec(pair, ctx),
        Rule::floating_pt_type => floating_pt_type(pair, ctx),
        Rule::integer_type => integer_type(pair, ctx),
        Rule::signed_tiny_int => signed_tiny_int(pair, ctx),
        Rule::signed_int => signed_int(pair, ctx),
        Rule::signed_short_int => signed_short_int(pair, ctx),
        Rule::signed_long_int => signed_long_int(pair, ctx),
        Rule::signed_longlong_int => signed_longlong_int(pair, ctx),
        Rule::unsigned_tiny_int => unsigned_tiny_int(pair, ctx),
        Rule::unsigned_int => unsigned_int(pair, ctx),
        Rule::unsigned_short_int => unsigned_short_int(pair, ctx),
        Rule::unsigned_long_int => unsigned_long_int(pair, ctx),
        Rule::unsigned_longlong_int => unsigned_longlong_int(pair, ctx),
        Rule::char_type => char_type(pair, ctx),
        Rule::wide_char_type => wide_char_type(pair, ctx),
        Rule::boolean_type => boolean(pair, ctx),
        Rule::octet_type => octet_type(pair, ctx),
        Rule::template_type_spec => template_type_spec(pair, ctx),
        Rule::sequence_type => sequence_type(pair, ctx),
        Rule::string_type => string_type(pair, ctx),
        Rule::wide_string_type => wide_string_type(pair, ctx),
        Rule::fixed_pt_type => unimplemented!("Fixed point not supported in Rust mapping"),
        Rule::fixed_pt_const_type => unimplemented!("Fixed point not supported in Rust mapping"),
        Rule::constr_type_dcl => constr_type_dcl(pair, ctx),
        Rule::struct_dcl => struct_dcl(pair, ctx),
        Rule::struct_def => struct_def(pair, ctx),
        Rule::member => member(pair, ctx),
        Rule::struct_forward_dcl => Ok(()), // Forward declarations are irrelevant in Rust mapping
        Rule::union_dcl => todo!(),
        Rule::union_def => todo!(),
        Rule::switch_type_spec => todo!(),
        Rule::switch_body => todo!(),
        Rule::case => todo!(),
        Rule::case_label => todo!(),
        Rule::element_spec => todo!(),
        Rule::union_forward_dcl => Ok(()), // Forward declarations are irrelevant in Rust mapping
        Rule::enum_dcl => enum_dcl(pair, ctx),
        Rule::enumerator => enumerator(pair, ctx),
        Rule::array_declarator => todo!(),
        Rule::fixed_array_size => fixed_array_size(pair, ctx),
        Rule::native_dcl => todo!(),
        Rule::simple_declarator => simple_declarator(pair, ctx),
        Rule::typedef_dcl => typedef_dcl(pair, ctx),
        Rule::type_declarator => type_declarator(pair, ctx),
        Rule::any_declarators => Ok(()), // Handled inside typedef_dcl
        Rule::any_declarator => any_declarator(pair, ctx),
        Rule::declarators => declarators(pair, ctx),
        Rule::declarator => todo!(),
        Rule::any_type => todo!(),
        Rule::except_dcl => todo!(),
        Rule::interface_dcl => interface_dcl(pair, ctx),
        Rule::interface_def => interface_def(pair, ctx),
        Rule::interface_forward_dcl => Ok(()), // Forward declarations are irrelevant in Rust mapping
        Rule::interface_header => interface_header(pair, ctx),
        Rule::interface_kind => interface_kind(pair, ctx),
        Rule::interface_inheritance_spec => todo!(),
        Rule::interface_name => todo!(),
        Rule::interface_body => interface_body(pair, ctx),
        Rule::export => export(pair, ctx),
        Rule::op_dcl => op_dcl(pair, ctx),
        Rule::op_type_spec => op_type_spec(pair, ctx),
        Rule::parameter_dcls => parameter_dcls(pair, ctx),
        Rule::param_dcl => param_dcl(pair, ctx),
        Rule::param_attribute => param_attribute(pair, ctx),
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

fn specification<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    for definition in pair.into_inner() {
        generate_rust_source(definition, ctx)?;
    }

    Ok(())
}

#[inline]
fn definition<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        ctx,
    )
}

fn module_dcl<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    let inner_pairs = pair.into_inner();
    let identifier = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::identifier)
        .expect("Must have an identifier according to the grammar");
    ctx.modules.push(identifier.as_str().to_string());
    ctx.writer.write_str("pub mod ")?;
    generate_rust_source(identifier, ctx)?;
    ctx.writer.write_char('{')?;

    for definition in inner_pairs
        .clone()
        .filter(|p| p.as_rule() == Rule::definition)
    {
        generate_rust_source(definition, ctx)?;
    }

    ctx.writer.write_char('}')?;
    ctx.modules.pop();

    Ok(())
}

#[inline]
fn type_dcl<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        ctx,
    )
}

#[inline]
fn constr_type_dcl<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        ctx,
    )
}

#[inline]
fn struct_dcl<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        ctx,
    )
}

fn struct_def<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    let inner_pairs = pair.into_inner();
    let identifier = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::identifier)
        .expect("Identifier must exist according to the grammar");

    ctx.writer
        .write_str("#[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]\n")?;
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

        match identifier.as_str() {
            "final" => ctx
                .writer
                .write_str("#[dust_dds(extensibility = \"final\")]\n")?,
            "appendable" => ctx
                .writer
                .write_str("#[dust_dds(extensibility = \"appendable\")]\n")?,
            "mutable" => ctx
                .writer
                .write_str("#[dust_dds(extensibility = \"mutable\")]\n")?,
            _ => (),
        }
    }

    if !ctx.modules.is_empty() {
        let name = format!(
            "#[dust_dds(name = \"{}\")]\n",
            ctx.type_name(identifier.as_str())
        );
        ctx.writer.write_str(&name)?;
    }

    ctx.writer.write_str("pub struct ")?;
    generate_rust_source(identifier, ctx)?;

    ctx.writer.write_str(" {")?;

    for member in inner_pairs.filter(|p| p.as_rule() == Rule::member) {
        generate_rust_source(member, ctx)?;
    }

    ctx.writer.write_str("}\n")?;

    Ok(())
}

fn enum_dcl<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    let inner_pairs = pair.into_inner();
    let identifier = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::identifier)
        .expect("Must have an identifier according to the grammar");
    ctx.writer
        .write_str("#[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]\n")?;
    if !ctx.modules.is_empty() {
        let name = format!(
            "#[dust_dds(name = \"{}\")]\n",
            ctx.type_name(identifier.as_str())
        );
        ctx.writer.write_str(&name)?;
    }
    ctx.writer.write_str("pub enum ")?;
    generate_rust_source(identifier, ctx)?;
    ctx.writer.write_str(" {")?;

    for enumerator in inner_pairs
        .clone()
        .filter(|p| p.as_rule() == Rule::enumerator)
    {
        generate_rust_source(enumerator, ctx)?;
        ctx.writer.write_char(',')?;
    }

    ctx.writer.write_str("}\n")?;

    Ok(())
}

#[inline]
fn enumerator<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        ctx,
    )
}

fn member<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
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
            ctx.writer.write_str("#[dust_dds(key)]")?;
        }
    }

    for declarator in declarators.into_inner() {
        let array_or_simple_declarator = declarator
            .into_inner()
            .next()
            .expect("Must have an element according to the grammar");
        ctx.writer.write_str("pub ")?;
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
                generate_rust_source(identifier, ctx)?;
                ctx.writer.write_char(':')?;
                ctx.writer.write_char('[')?;
                generate_rust_source(type_spec.clone(), ctx)?;
                ctx.writer.write_char(';')?;
                generate_rust_source(fixed_array_size, ctx)?;
                ctx.writer.write_char(']')?;
            }
            Rule::simple_declarator => {
                generate_rust_source(array_or_simple_declarator, ctx)?;
                ctx.writer.write_char(':')?;
                generate_rust_source(type_spec.clone(), ctx)?;
            }
            _ => panic!("Not allowed by the grammar"),
        }
        ctx.writer.write_char(',')?;
    }

    Ok(())
}

fn declarators<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    for declarator in pair.into_inner() {
        generate_rust_source(declarator, ctx)?;
    }

    Ok(())
}

#[inline]
fn interface_dcl<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        ctx,
    )
}

fn interface_def<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    let inner_pairs = pair.into_inner();

    let interface_header = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::interface_header)
        .expect("Must have an interface_header according to grammar");

    let interface_body = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::interface_body)
        .expect("Must have an interface_body according to grammar");

    generate_rust_source(interface_header, ctx)?;
    ctx.writer.write_char('{')?;
    generate_rust_source(interface_body, ctx)?;
    ctx.writer.write_str("}\n")?;

    Ok(())
}

fn interface_header<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
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

    generate_rust_source(interface_kind, ctx)?;
    generate_rust_source(identifier, ctx)?;

    if let Some(interface_inheritance_spec) = interface_inheritance_spec {
        generate_rust_source(interface_inheritance_spec, ctx)?;
    }

    Ok(())
}

fn interface_kind<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    match pair.as_str() {
        "interface" | "abstract interface" => ctx.writer.write_str("pub trait "),
        "local interface" => ctx.writer.write_str("trait "),
        _ => panic!("Invalid string according to grammar"),
    }
}

fn interface_body<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    for export in pair.into_inner().filter(|p| p.as_rule() == Rule::export) {
        generate_rust_source(export, ctx)?;
    }

    Ok(())
}

#[inline]
fn export<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        ctx,
    )
}

fn op_dcl<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
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
    ctx.writer.write_str("fn ")?;
    generate_rust_source(identifier, ctx)?;
    ctx.writer.write_char('(')?;
    generate_rust_source(parameter_dcls, ctx)?;
    ctx.writer.write_char(')')?;
    generate_rust_source(op_type_spec, ctx)?;
    ctx.writer.write_str(";\n")?;

    Ok(())
}

fn op_type_spec<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    if let Some(type_spec) = pair.into_inner().find(|p| p.as_rule() == Rule::type_spec) {
        ctx.writer.write_str("->")?;
        generate_rust_source(type_spec, ctx)?;
    }

    Ok(())
}

fn parameter_dcls<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    for param_dcl in pair.into_inner().filter(|p| p.as_rule() == Rule::param_dcl) {
        generate_rust_source(param_dcl, ctx)?;
    }

    Ok(())
}

fn param_dcl<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
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

    generate_rust_source(simple_declarator, ctx)?;
    ctx.writer.write_char(':')?;
    generate_rust_source(param_attribute, ctx)?;
    generate_rust_source(type_spec, ctx)?;
    ctx.writer.write_char(',')?;

    Ok(())
}

fn param_attribute<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    match pair.as_str() {
        "inout" | "out" => ctx.writer.write_str("&mut "),
        "in" => ctx.writer.write_char('&'),
        _ => panic!("Invalid option by grammar"),
    }
}

#[inline]
fn simple_declarator<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        ctx,
    )
}

#[inline]
fn typedef_dcl<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        ctx,
    )
}

fn type_declarator<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
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
        ctx.writer.write_str("pub type ")?;
        generate_rust_source(any_declarator, ctx)?;
        ctx.writer.write_char('=')?;
        generate_rust_source(type_spec.clone(), ctx)?;
        ctx.writer.write_str(";\n")?;
    }

    Ok(())
}

#[inline]
fn any_declarator<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        ctx,
    )
}

#[inline]
fn identifier<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    ctx.writer.write_str(pair.as_str())
}

#[inline]
fn type_spec<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        ctx,
    )
}

#[inline]
fn simple_type_spec<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        ctx,
    )
}

#[inline]
fn base_type_spec<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        ctx,
    )
}

fn floating_pt_type<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    match pair.as_str() {
        "float" => ctx.writer.write_str("f32"),
        "double" => ctx.writer.write_str("f64"),
        "long double" => unimplemented!("long double not supported in Rust"),
        _ => panic!("Invalid option by grammar"),
    }
}

#[inline]
fn integer_type<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        ctx,
    )
}

#[inline]
fn signed_tiny_int<W: Write>(_pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    ctx.writer.write_str("i8")
}

#[inline]
fn signed_int<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        ctx,
    )
}

#[inline]
fn signed_short_int<W: Write>(_pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    ctx.writer.write_str("i16")
}

#[inline]
fn signed_long_int<W: Write>(_pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    ctx.writer.write_str("i32")
}

#[inline]
fn signed_longlong_int<W: Write>(_pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    ctx.writer.write_str("i64")
}

#[inline]
fn unsigned_tiny_int<W: Write>(_pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    ctx.writer.write_str("u8")
}

#[inline]
fn unsigned_int<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        ctx,
    )
}

#[inline]
fn unsigned_short_int<W: Write>(_pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    ctx.writer.write_str("u16")
}

#[inline]
fn unsigned_long_int<W: Write>(_pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    ctx.writer.write_str("u32")
}

#[inline]
fn unsigned_longlong_int<W: Write>(_pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    ctx.writer.write_str("u64")
}

#[inline]
fn octet_type<W: Write>(_pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    ctx.writer.write_str("u8")
}

#[inline]
fn template_type_spec<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        ctx,
    )
}

fn sequence_type<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    let inner_pairs = pair.into_inner();

    let type_spec = inner_pairs
        .clone()
        .find(|p| p.as_rule() == Rule::type_spec)
        .expect("Must have a type_spec according to the grammar");

    ctx.writer.write_str("Vec<")?;
    generate_rust_source(type_spec, ctx)?;
    ctx.writer.write_char('>')?;

    Ok(())
}

#[inline]
fn string_type<W: Write>(_pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    ctx.writer.write_str("String")
}

#[inline]
fn wide_string_type<W: Write>(_pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    ctx.writer.write_str("String")
}

#[inline]
fn fixed_array_size<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        ctx,
    )
}

#[inline]
fn positive_int_const<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to the grammar"),
        ctx,
    )
}

#[inline]
fn const_expr<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    ctx.writer.write_str(pair.as_str())
}

fn scoped_name<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    let identifier = pair
        .into_inner()
        .next()
        .expect("Must have an identifier according to the grammar");

    ctx.writer.write_str(identifier.as_str())
}

fn const_dcl<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
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

    ctx.writer.write_str("pub const ")?;
    generate_rust_source(identifier, ctx)?;
    ctx.writer.write_char(':')?;
    generate_rust_source(const_type, ctx)?;
    ctx.writer.write_char('=')?;
    generate_rust_source(const_expr, ctx)?;
    ctx.writer.write_str(";\n")?;

    Ok(())
}

#[inline]
fn const_type<W: Write>(pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    generate_rust_source(
        pair.into_inner()
            .next()
            .expect("Must have an element according to grammar"),
        ctx,
    )
}

#[inline]
fn char_type<W: Write>(_pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    ctx.writer.write_str("char")
}

#[inline]
fn wide_char_type<W: Write>(_pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    ctx.writer.write_str("char")
}

#[inline]
fn boolean<W: Write>(_pair: IdlPair, ctx: &mut Context<W>) -> Result<(), Error> {
    ctx.writer.write_str("bool")
}

#[cfg(test)]
mod tests {
    use super::generate_rust_source;
    use crate::parser::{IdlParser, Rule};
    use pest::Parser;

    type Context = super::Context<String>;

    #[test]
    fn parse_struct() {
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
        let mut ctx = Context::default();
        generate_rust_source(p, &mut ctx).unwrap();
        assert_eq!(
            &ctx.writer,
            "#[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]\npub struct MyStruct {pub a:i32,pub b:i64,pub c:i64,pub xary:[u8;32],pub yary:[u8;64],}\n",
        );
    }

    #[test]
    fn parse_enum() {
        let p = IdlParser::parse(
            Rule::enum_dcl,
            "enum MyEnum {
                A,
                B,
                C
            };",
        )
        .unwrap()
        .next()
        .unwrap();
        let mut ctx = Context::default();
        generate_rust_source(p, &mut ctx).unwrap();
        assert_eq!(
            &ctx.writer,
            "#[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]\npub enum MyEnum {A,B,C,}\n",
        );
    }

    #[test]
    fn parse_appendable_struct() {
        let p = IdlParser::parse(Rule::struct_def, "@appendable struct MyStruct { long a; };")
            .unwrap()
            .next()
            .unwrap();
        let mut ctx = Context::default();
        generate_rust_source(p, &mut ctx).unwrap();
        assert_eq!(
            &ctx.writer,
            "#[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]\n#[dust_dds(extensibility = \"appendable\")]\npub struct MyStruct {pub a:i32,}\n",
        );
    }

    #[test]
    fn parse_member_with_key() {
        let p = IdlParser::parse(Rule::member, "@key long a;")
            .unwrap()
            .next()
            .unwrap();
        let mut ctx = Context::default();
        generate_rust_source(p, &mut ctx).unwrap();
        assert_eq!(&ctx.writer, "#[dust_dds(key)]pub a:i32,");
    }

    #[test]
    fn parse_member_sequence_type() {
        let p = IdlParser::parse(Rule::member, "sequence<octet> a;")
            .unwrap()
            .next()
            .unwrap();
        let mut ctx = Context::default();
        generate_rust_source(p, &mut ctx).unwrap();
        assert_eq!(&ctx.writer, "pub a:Vec<u8>,");
    }

    #[test]
    fn parse_member_sequence_of_sequence_type() {
        let p = IdlParser::parse(Rule::member, "sequence<sequence<octet>> a;")
            .unwrap()
            .next()
            .unwrap();
        let mut ctx = Context::default();
        generate_rust_source(p, &mut ctx).unwrap();
        assert_eq!(&ctx.writer, "pub a:Vec<Vec<u8>>,");
    }

    #[test]
    fn parse_const_with_literals() {
        let p = IdlParser::parse(Rule::specification, r#"const string a = 'a';"#)
            .unwrap()
            .next()
            .unwrap();
        let mut ctx = Context::default();
        generate_rust_source(p, &mut ctx).unwrap();
        assert_eq!(&ctx.writer, "pub const a:String='a';\n");
    }

    #[test]
    fn parse_typedef() {
        let p = IdlParser::parse(Rule::typedef_dcl, r#"typedef long Name;"#)
            .unwrap()
            .next()
            .unwrap();
        let mut ctx = Context::default();
        generate_rust_source(p, &mut ctx).unwrap();
        assert_eq!(&ctx.writer, "pub type Name=i32;\n");
    }

    #[test]
    fn parse_interface_export() {
        let p = IdlParser::parse(Rule::export, r#"short op(out string s);"#)
            .unwrap()
            .next()
            .unwrap();
        let mut ctx = Context::default();
        generate_rust_source(p, &mut ctx).unwrap();
        assert_eq!(&ctx.writer, "fn op(s:&mut String,)->i16;\n");
    }

    #[test]
    fn parse_interface() {
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
        let mut ctx = Context::default();
        generate_rust_source(p, &mut ctx).unwrap();
        assert_eq!(
            &ctx.writer,
            "pub trait MyInterface{fn op(s:&mut String,a:&mut i16,u:&char,);\nfn sum(a:&u16,b:&u16,)->u16;\n}\n",
        );
    }

    #[test]
    fn parse_with_modules() {
        let p = IdlParser::parse(
            Rule::module_dcl,
            "module root {
                module a {
                    struct A {
                        uint8 x;
                        uint16 y;
                        uint32 z;
                    };
                };

                module b {
                    enum B {
                        X,
                        Y,
                        Z
                    };
                };
            };",
        )
        .unwrap()
        .next()
        .unwrap();
        let mut ctx = Context::default();
        generate_rust_source(p, &mut ctx).unwrap();
        assert_eq!(
            &ctx.writer,
            "pub mod root{pub mod a{#[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]\n#[dust_dds(name = \"root::a::A\")]\npub struct A {pub x:u8,pub y:u16,pub z:u32,}\n}pub mod b{#[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]\n#[dust_dds(name = \"root::b::B\")]\npub enum B {X,Y,Z,}\n}}",
        );
    }
}
