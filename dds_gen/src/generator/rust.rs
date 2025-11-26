use super::Generator;
use crate::parser::{IdlPair, Rule};

/// _Rust_ generator.
#[derive(Debug)]
pub struct RustGenerator {
    /// Writer.
    writer: String,
    /// List of modules to keep track hierarchy.
    modules: Vec<String>,
}

impl RustGenerator {
    /// Constructs a new `Context` from `writer`.
    #[inline]
    #[must_use]
    pub fn new(writer: String) -> Self {
        Self {
            writer,
            modules: Vec::default(),
        }
    }

    /// Consumes `self` returning `writer`.
    #[inline]
    #[must_use]
    pub fn into_writer(self) -> String {
        self.writer
    }

    fn specification(&mut self, pair: IdlPair) {
        for definition in pair.into_inner() {
            self.generate(definition);
        }
    }

    #[inline]
    fn definition(&mut self, pair: IdlPair) {
        self.generate(
            pair.into_inner()
                .next()
                .expect("Must have an element according to the grammar"),
        )
    }

    fn module_dcl(&mut self, pair: IdlPair) {
        let inner_pairs = pair.into_inner();
        let identifier = inner_pairs
            .clone()
            .find(|p| p.as_rule() == Rule::identifier)
            .expect("Must have an identifier according to the grammar");
        self.modules.push(identifier.as_str().to_string());
        self.writer.push_str("pub mod ");
        self.generate(identifier);
        self.writer.push('{');

        for definition in inner_pairs
            .clone()
            .filter(|p| p.as_rule() == Rule::definition)
        {
            self.generate(definition);
        }

        self.writer.push('}');
        self.modules.pop();
    }

    #[inline]
    fn type_dcl(&mut self, pair: IdlPair) {
        self.generate(
            pair.into_inner()
                .next()
                .expect("Must have an element according to the grammar"),
        )
    }

    #[inline]
    fn constr_type_dcl(&mut self, pair: IdlPair) {
        self.generate(
            pair.into_inner()
                .next()
                .expect("Must have an element according to the grammar"),
        )
    }

    #[inline]
    fn struct_dcl(&mut self, pair: IdlPair) {
        self.generate(
            pair.into_inner()
                .next()
                .expect("Must have an element according to the grammar"),
        )
    }

    fn struct_def(&mut self, pair: IdlPair) {
        let inner_pairs = pair.into_inner();
        let identifier = inner_pairs
            .clone()
            .find(|p| p.as_rule() == Rule::identifier)
            .expect("Identifier must exist according to the grammar");

        self.writer
            .push_str("#[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]\n");
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
                "final" => self
                    .writer
                    .push_str("#[dust_dds(extensibility = \"final\")]\n"),
                "appendable" => self
                    .writer
                    .push_str("#[dust_dds(extensibility = \"appendable\")]\n"),
                "mutable" => self
                    .writer
                    .push_str("#[dust_dds(extensibility = \"mutable\")]\n"),
                _ => (),
            }
        }

        if !self.modules.is_empty() {
            let name = format!(
                "#[dust_dds(name = \"{}\")]\n",
                self.type_name(identifier.as_str())
            );
            self.writer.push_str(&name);
        }

        self.writer.push_str("pub struct ");
        self.generate(identifier);

        self.writer.push_str(" {");

        for member in inner_pairs.filter(|p| p.as_rule() == Rule::member) {
            self.generate(member);
        }

        self.writer.push_str("}\n");
    }

    fn enum_dcl(&mut self, pair: IdlPair) {
        let inner_pairs = pair.into_inner();
        let identifier = inner_pairs
            .clone()
            .find(|p| p.as_rule() == Rule::identifier)
            .expect("Must have an identifier according to the grammar");
        self.writer
            .push_str("#[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]\n");
        if !self.modules.is_empty() {
            let name = format!(
                "#[dust_dds(name = \"{}\")]\n",
                self.type_name(identifier.as_str())
            );
            self.writer.push_str(&name);
        }
        self.writer.push_str("pub enum ");
        self.generate(identifier);
        self.writer.push_str(" {");

        for enumerator in inner_pairs
            .clone()
            .filter(|p| p.as_rule() == Rule::enumerator)
        {
            self.generate(enumerator);
            self.writer.push(',');
        }

        self.writer.push_str("}\n");
    }

    #[inline]
    fn enumerator(&mut self, pair: IdlPair) {
        self.generate(
            pair.into_inner()
                .next()
                .expect("Must have an element according to the grammar"),
        )
    }

    fn member(&mut self, pair: IdlPair) {
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
                self.writer.push_str("#[dust_dds(key)]");
            }
        }

        for declarator in declarators.into_inner() {
            let array_or_simple_declarator = declarator
                .into_inner()
                .next()
                .expect("Must have an element according to the grammar");
            self.writer.push_str("pub ");
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
                    self.generate(identifier);
                    self.writer.push(':');
                    self.writer.push('[');
                    self.generate(type_spec.clone());
                    self.writer.push(';');
                    self.generate(fixed_array_size);
                    self.writer.push(']');
                }
                Rule::simple_declarator => {
                    self.generate(array_or_simple_declarator);
                    self.writer.push(':');
                    self.generate(type_spec.clone());
                }
                _ => panic!("Not allowed by the grammar"),
            }
            self.writer.push(',');
        }
    }

    fn declarators(&mut self, pair: IdlPair) {
        for declarator in pair.into_inner() {
            self.generate(declarator);
        }
    }

    #[inline]
    fn interface_dcl(&mut self, pair: IdlPair) {
        self.generate(
            pair.into_inner()
                .next()
                .expect("Must have an element according to the grammar"),
        )
    }

    fn interface_def(&mut self, pair: IdlPair) {
        let inner_pairs = pair.into_inner();

        let interface_header = inner_pairs
            .clone()
            .find(|p| p.as_rule() == Rule::interface_header)
            .expect("Must have an interface_header according to grammar");

        let interface_body = inner_pairs
            .clone()
            .find(|p| p.as_rule() == Rule::interface_body)
            .expect("Must have an interface_body according to grammar");

        self.generate(interface_header);
        self.writer.push('{');
        self.generate(interface_body);
        self.writer.push_str("}\n");
    }

    fn interface_header(&mut self, pair: IdlPair) {
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

        self.generate(interface_kind);
        self.generate(identifier);

        if let Some(interface_inheritance_spec) = interface_inheritance_spec {
            self.generate(interface_inheritance_spec);
        }
    }

    fn interface_kind(&mut self, pair: IdlPair) {
        match pair.as_str() {
            "interface" | "abstract interface" => self.writer.push_str("pub trait "),
            "local interface" => self.writer.push_str("trait "),
            _ => panic!("Invalid string according to grammar"),
        }
    }

    fn interface_body(&mut self, pair: IdlPair) {
        for export in pair.into_inner().filter(|p| p.as_rule() == Rule::export) {
            self.generate(export);
        }
    }

    #[inline]
    fn export(&mut self, pair: IdlPair) {
        self.generate(
            pair.into_inner()
                .next()
                .expect("Must have an element according to the grammar"),
        )
    }

    fn op_dcl(&mut self, pair: IdlPair) {
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
        self.writer.push_str("fn ");
        self.generate(identifier);
        self.writer.push('(');
        self.generate(parameter_dcls);
        self.writer.push(')');
        self.generate(op_type_spec);
        self.writer.push_str(";\n");
    }

    fn op_type_spec(&mut self, pair: IdlPair) {
        if let Some(type_spec) = pair.into_inner().find(|p| p.as_rule() == Rule::type_spec) {
            self.writer.push_str("->");
            self.generate(type_spec);
        }
    }

    fn parameter_dcls(&mut self, pair: IdlPair) {
        for param_dcl in pair.into_inner().filter(|p| p.as_rule() == Rule::param_dcl) {
            self.generate(param_dcl);
        }
    }

    fn param_dcl(&mut self, pair: IdlPair) {
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

        self.generate(simple_declarator);
        self.writer.push(':');
        self.generate(param_attribute);
        self.generate(type_spec);
        self.writer.push(',');
    }

    fn param_attribute(&mut self, pair: IdlPair) {
        match pair.as_str() {
            "inout" | "out" => self.writer.push_str("&mut "),
            "in" => self.writer.push('&'),
            _ => panic!("Invalid option by grammar"),
        }
    }

    #[inline]
    fn simple_declarator(&mut self, pair: IdlPair) {
        self.generate(
            pair.into_inner()
                .next()
                .expect("Must have an element according to the grammar"),
        )
    }

    #[inline]
    fn typedef_dcl(&mut self, pair: IdlPair) {
        self.generate(
            pair.into_inner()
                .next()
                .expect("Must have an element according to the grammar"),
        )
    }

    fn type_declarator(&mut self, pair: IdlPair) {
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
            self.writer.push_str("pub type ");
            self.generate(any_declarator);
            self.writer.push('=');
            self.generate(type_spec.clone());
            self.writer.push_str(";\n");
        }
    }

    #[inline]
    fn any_declarator(&mut self, pair: IdlPair) {
        self.generate(
            pair.into_inner()
                .next()
                .expect("Must have an element according to the grammar"),
        )
    }

    #[inline]
    fn identifier(&mut self, pair: IdlPair) {
        self.writer.push_str(pair.as_str())
    }

    #[inline]
    fn type_spec(&mut self, pair: IdlPair) {
        self.generate(
            pair.into_inner()
                .next()
                .expect("Must have an element according to the grammar"),
        )
    }

    #[inline]
    fn simple_type_spec(&mut self, pair: IdlPair) {
        self.generate(
            pair.into_inner()
                .next()
                .expect("Must have an element according to the grammar"),
        )
    }

    #[inline]
    fn base_type_spec(&mut self, pair: IdlPair) {
        self.generate(
            pair.into_inner()
                .next()
                .expect("Must have an element according to the grammar"),
        )
    }

    fn floating_pt_type(&mut self, pair: IdlPair) {
        match pair.as_str() {
            "float" => self.writer.push_str("f32"),
            "double" => self.writer.push_str("f64"),
            "long double" => unimplemented!("long double not supported in Rust"),
            _ => panic!("Invalid option by grammar"),
        }
    }

    #[inline]
    fn integer_type(&mut self, pair: IdlPair) {
        self.generate(
            pair.into_inner()
                .next()
                .expect("Must have an element according to the grammar"),
        )
    }

    #[inline]
    fn signed_tiny_int(&mut self, _pair: IdlPair) {
        self.writer.push_str("i8")
    }

    #[inline]
    fn signed_int(&mut self, pair: IdlPair) {
        self.generate(
            pair.into_inner()
                .next()
                .expect("Must have an element according to the grammar"),
        )
    }

    #[inline]
    fn signed_short_int(&mut self, _pair: IdlPair) {
        self.writer.push_str("i16")
    }

    #[inline]
    fn signed_long_int(&mut self, _pair: IdlPair) {
        self.writer.push_str("i32")
    }

    #[inline]
    fn signed_longlong_int(&mut self, _pair: IdlPair) {
        self.writer.push_str("i64")
    }

    #[inline]
    fn unsigned_tiny_int(&mut self, _pair: IdlPair) {
        self.writer.push_str("u8")
    }

    #[inline]
    fn unsigned_int(&mut self, pair: IdlPair) {
        self.generate(
            pair.into_inner()
                .next()
                .expect("Must have an element according to the grammar"),
        )
    }

    #[inline]
    fn unsigned_short_int(&mut self, _pair: IdlPair) {
        self.writer.push_str("u16")
    }

    #[inline]
    fn unsigned_long_int(&mut self, _pair: IdlPair) {
        self.writer.push_str("u32")
    }

    #[inline]
    fn unsigned_longlong_int(&mut self, _pair: IdlPair) {
        self.writer.push_str("u64")
    }

    #[inline]
    fn octet_type(&mut self, _pair: IdlPair) {
        self.writer.push_str("u8")
    }

    #[inline]
    fn template_type_spec(&mut self, pair: IdlPair) {
        self.generate(
            pair.into_inner()
                .next()
                .expect("Must have an element according to the grammar"),
        )
    }

    fn sequence_type(&mut self, pair: IdlPair) {
        let inner_pairs = pair.into_inner();

        let type_spec = inner_pairs
            .clone()
            .find(|p| p.as_rule() == Rule::type_spec)
            .expect("Must have a type_spec according to the grammar");

        self.writer.push_str("Vec<");
        self.generate(type_spec);
        self.writer.push('>');
    }

    #[inline]
    fn string_type(&mut self, _pair: IdlPair) {
        self.writer.push_str("String")
    }

    #[inline]
    fn wide_string_type(&mut self, _pair: IdlPair) {
        self.writer.push_str("String")
    }

    #[inline]
    fn fixed_array_size(&mut self, pair: IdlPair) {
        self.generate(
            pair.into_inner()
                .next()
                .expect("Must have an element according to the grammar"),
        )
    }

    #[inline]
    fn positive_int_const(&mut self, pair: IdlPair) {
        self.generate(
            pair.into_inner()
                .next()
                .expect("Must have an element according to the grammar"),
        )
    }

    #[inline]
    fn const_expr(&mut self, pair: IdlPair) {
        self.writer.push_str(pair.as_str())
    }

    fn scoped_name(&mut self, pair: IdlPair) {
        let identifier = pair
            .into_inner()
            .next()
            .expect("Must have an identifier according to the grammar");

        self.writer.push_str(identifier.as_str())
    }

    fn const_dcl(&mut self, pair: IdlPair) {
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

        self.writer.push_str("pub const ");
        self.generate(identifier);
        self.writer.push(':');
        self.generate(const_type);
        self.writer.push('=');
        self.generate(const_expr);
        self.writer.push_str(";\n");
    }

    #[inline]
    fn const_type(&mut self, pair: IdlPair) {
        self.generate(
            pair.into_inner()
                .next()
                .expect("Must have an element according to grammar"),
        )
    }

    #[inline]
    fn char_type(&mut self, _pair: IdlPair) {
        self.writer.push_str("char")
    }

    #[inline]
    fn wide_char_type(&mut self, _pair: IdlPair) {
        self.writer.push_str("char")
    }

    #[inline]
    fn boolean(&mut self, _pair: IdlPair) {
        self.writer.push_str("bool")
    }
}

impl Generator for RustGenerator {
    fn generate(&mut self, pair: IdlPair) {
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
            Rule::identifier => self.identifier(pair),
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
            Rule::specification => self.specification(pair),
            Rule::definition => self.definition(pair),
            Rule::module_dcl => self.module_dcl(pair),
            Rule::scoped_name => self.scoped_name(pair),
            Rule::const_dcl => self.const_dcl(pair),
            Rule::const_type => self.const_type(pair),
            Rule::const_expr => self.const_expr(pair),
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
            Rule::positive_int_const => self.positive_int_const(pair),
            Rule::type_dcl => self.type_dcl(pair),
            Rule::type_spec => self.type_spec(pair),
            Rule::simple_type_spec => self.simple_type_spec(pair),
            Rule::base_type_spec => self.base_type_spec(pair),
            Rule::floating_pt_type => self.floating_pt_type(pair),
            Rule::integer_type => self.integer_type(pair),
            Rule::signed_tiny_int => self.signed_tiny_int(pair),
            Rule::signed_int => self.signed_int(pair),
            Rule::signed_short_int => self.signed_short_int(pair),
            Rule::signed_long_int => self.signed_long_int(pair),
            Rule::signed_longlong_int => self.signed_longlong_int(pair),
            Rule::unsigned_tiny_int => self.unsigned_tiny_int(pair),
            Rule::unsigned_int => self.unsigned_int(pair),
            Rule::unsigned_short_int => self.unsigned_short_int(pair),
            Rule::unsigned_long_int => self.unsigned_long_int(pair),
            Rule::unsigned_longlong_int => self.unsigned_longlong_int(pair),
            Rule::char_type => self.char_type(pair),
            Rule::wide_char_type => self.wide_char_type(pair),
            Rule::boolean_type => self.boolean(pair),
            Rule::octet_type => self.octet_type(pair),
            Rule::template_type_spec => self.template_type_spec(pair),
            Rule::sequence_type => self.sequence_type(pair),
            Rule::string_type => self.string_type(pair),
            Rule::wide_string_type => self.wide_string_type(pair),
            Rule::fixed_pt_type => unimplemented!("Fixed point not supported in Rust mapping"),
            Rule::fixed_pt_const_type => {
                unimplemented!("Fixed point not supported in Rust mapping")
            }
            Rule::constr_type_dcl => self.constr_type_dcl(pair),
            Rule::struct_dcl => self.struct_dcl(pair),
            Rule::struct_def => self.struct_def(pair),
            Rule::member => self.member(pair),
            Rule::struct_forward_dcl => (), // Forward declarations are irrelevant in Rust mapping
            Rule::union_dcl => todo!(),
            Rule::union_def => todo!(),
            Rule::switch_type_spec => todo!(),
            Rule::switch_body => todo!(),
            Rule::case => todo!(),
            Rule::case_label => todo!(),
            Rule::element_spec => todo!(),
            Rule::union_forward_dcl => (), // Forward declarations are irrelevant in Rust mapping
            Rule::enum_dcl => self.enum_dcl(pair),
            Rule::enumerator => self.enumerator(pair),
            Rule::array_declarator => todo!(),
            Rule::fixed_array_size => self.fixed_array_size(pair),
            Rule::native_dcl => todo!(),
            Rule::simple_declarator => self.simple_declarator(pair),
            Rule::typedef_dcl => self.typedef_dcl(pair),
            Rule::type_declarator => self.type_declarator(pair),
            Rule::any_declarators => (), // Handled inside typedef_dcl
            Rule::any_declarator => self.any_declarator(pair),
            Rule::declarators => self.declarators(pair),
            Rule::declarator => todo!(),
            Rule::any_type => todo!(),
            Rule::except_dcl => todo!(),
            Rule::interface_dcl => self.interface_dcl(pair),
            Rule::interface_def => self.interface_def(pair),
            Rule::interface_forward_dcl => (), // Forward declarations are irrelevant in Rust mapping
            Rule::interface_header => self.interface_header(pair),
            Rule::interface_kind => self.interface_kind(pair),
            Rule::interface_inheritance_spec => todo!(),
            Rule::interface_name => todo!(),
            Rule::interface_body => self.interface_body(pair),
            Rule::export => self.export(pair),
            Rule::op_dcl => self.op_dcl(pair),
            Rule::op_type_spec => self.op_type_spec(pair),
            Rule::parameter_dcls => self.parameter_dcls(pair),
            Rule::param_dcl => self.param_dcl(pair),
            Rule::param_attribute => self.param_attribute(pair),
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

    #[inline]
    fn modules(&self) -> &[String] {
        &self.modules
    }
}

impl Default for RustGenerator {
    #[inline]
    fn default() -> Self {
        Self::new(String::default())
    }
}

#[cfg(test)]
mod tests {
    use super::{Generator, RustGenerator};
    use crate::parser::{IdlParser, Rule};
    use pest::Parser;

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
        let mut rust_generator = RustGenerator::default();
        rust_generator.generate(p);
        let writer = rust_generator.into_writer();
        assert_eq!(
            &writer,
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
        let mut rust_generator = RustGenerator::default();
        rust_generator.generate(p);
        let writer = rust_generator.into_writer();
        assert_eq!(
            &writer,
            "#[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]\npub enum MyEnum {A,B,C,}\n",
        );
    }

    #[test]
    fn parse_appendable_struct() {
        let p = IdlParser::parse(Rule::struct_def, "@appendable struct MyStruct { long a; };")
            .unwrap()
            .next()
            .unwrap();
        let mut rust_generator = RustGenerator::default();
        rust_generator.generate(p);
        let writer = rust_generator.into_writer();
        assert_eq!(
            &writer,
            "#[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]\n#[dust_dds(extensibility = \"appendable\")]\npub struct MyStruct {pub a:i32,}\n",
        );
    }

    #[test]
    fn parse_member_with_key() {
        let p = IdlParser::parse(Rule::member, "@key long a;")
            .unwrap()
            .next()
            .unwrap();
        let mut rust_generator = RustGenerator::default();
        rust_generator.generate(p);
        let writer = rust_generator.into_writer();
        assert_eq!(&writer, "#[dust_dds(key)]pub a:i32,");
    }

    #[test]
    fn parse_member_sequence_type() {
        let p = IdlParser::parse(Rule::member, "sequence<octet> a;")
            .unwrap()
            .next()
            .unwrap();
        let mut rust_generator = RustGenerator::default();
        rust_generator.generate(p);
        let writer = rust_generator.into_writer();
        assert_eq!(&writer, "pub a:Vec<u8>,");
    }

    #[test]
    fn parse_member_sequence_of_sequence_type() {
        let p = IdlParser::parse(Rule::member, "sequence<sequence<octet>> a;")
            .unwrap()
            .next()
            .unwrap();
        let mut rust_generator = RustGenerator::default();
        rust_generator.generate(p);
        let writer = rust_generator.into_writer();
        assert_eq!(&writer, "pub a:Vec<Vec<u8>>,");
    }

    #[test]
    fn parse_const_with_literals() {
        let p = IdlParser::parse(Rule::specification, r#"const string a = 'a';"#)
            .unwrap()
            .next()
            .unwrap();
        let mut rust_generator = RustGenerator::default();
        rust_generator.generate(p);
        let writer = rust_generator.into_writer();
        assert_eq!(&writer, "pub const a:String='a';\n");
    }

    #[test]
    fn parse_typedef() {
        let p = IdlParser::parse(Rule::typedef_dcl, r#"typedef long Name;"#)
            .unwrap()
            .next()
            .unwrap();
        let mut rust_generator = RustGenerator::default();
        rust_generator.generate(p);
        let writer = rust_generator.into_writer();
        assert_eq!(&writer, "pub type Name=i32;\n");
    }

    #[test]
    fn parse_interface_export() {
        let p = IdlParser::parse(Rule::export, r#"short op(out string s);"#)
            .unwrap()
            .next()
            .unwrap();
        let mut rust_generator = RustGenerator::default();
        rust_generator.generate(p);
        let writer = rust_generator.into_writer();
        assert_eq!(&writer, "fn op(s:&mut String,)->i16;\n");
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
        let mut rust_generator = RustGenerator::default();
        rust_generator.generate(p);
        let writer = rust_generator.into_writer();
        assert_eq!(
            &writer,
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
        let mut rust_generator = RustGenerator::default();
        rust_generator.generate(p);
        let writer = rust_generator.into_writer();
        assert_eq!(
            &writer,
            "pub mod root{pub mod a{#[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]\n#[dust_dds(name = \"root::a::A\")]\npub struct A {pub x:u8,pub y:u16,pub z:u32,}\n}pub mod b{#[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]\n#[dust_dds(name = \"root::b::B\")]\npub enum B {X,Y,Z,}\n}}",
        );
    }
}
