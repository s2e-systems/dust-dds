use crate::parser::{IdlPair, Rule};

/// _Rust_ generator.
#[derive(Debug)]
pub struct CGenerator<'a> {
    writer: &'a mut String,
    /// List of modules to keep track of hierarchy.
    modules: Vec<String>,
}

impl<'a> CGenerator<'a> {
    pub fn new(writer: &'a mut String) -> Self {
        Self {
            writer,
            modules: Vec::default(),
        }
    }

    pub fn generate(&mut self, pair: IdlPair) {
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
            Rule::semicolon => (),
            Rule::definition => self.definition(pair),
            Rule::module_dcl => todo!(),
            Rule::scoped_name => todo!(),
            Rule::const_dcl => todo!(),
            Rule::const_type => todo!(),
            Rule::const_expr => todo!(),
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
            Rule::positive_int_const => todo!(),
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
            Rule::sequence_type => todo!(),
            Rule::string_type => self.string_type(pair),
            Rule::wide_string_type => self.wide_string_type(pair),
            Rule::fixed_pt_type => todo!(),
            Rule::fixed_pt_const_type => todo!(),
            Rule::constr_type_dcl => self.constr_type_dcl(pair),
            Rule::struct_dcl => self.struct_dcl(pair),
            Rule::struct_def => self.struct_def(pair),
            Rule::member => self.member(pair),
            Rule::struct_forward_dcl => (),
            Rule::union_dcl => todo!(),
            Rule::union_def => todo!(),
            Rule::switch_type_spec => todo!(),
            Rule::switch_body => todo!(),
            Rule::case => todo!(),
            Rule::case_label => todo!(),
            Rule::element_spec => todo!(),
            Rule::union_forward_dcl => todo!(),
            Rule::enum_dcl => todo!(),
            Rule::enumerator => todo!(),
            Rule::array_declarator => todo!(),
            Rule::fixed_array_size => todo!(),
            Rule::native_dcl => todo!(),
            Rule::simple_declarator => self.simple_declarator(pair),
            Rule::typedef_dcl => todo!(),
            Rule::type_declarator => todo!(),
            Rule::any_declarators => todo!(),
            Rule::any_declarator => todo!(),
            Rule::declarators => todo!(),
            Rule::declarator => todo!(),
            Rule::any_type => todo!(),
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
            Rule::any_const_type => todo!(),
            Rule::annotation_appl => (),
            Rule::annotation_appl_params => (),
            Rule::annotation_appl_param => (),
        }
    }

    fn specification(&mut self, pair: IdlPair) {
        self.writer
            .push_str("\n    #include <stdbool.h>\n    #include <stdint.h>\n    #include <stddef.h>\n    #include <stdlib.h>\n    #include <string.h>\n    #include \"dust_dds.h\"\n\n");
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

    /// Returns the extensibility annotation string for a struct, if present.
    /// Returns `None` when no extensibility annotation is found (Final by convention
    /// in the DDS type system, so no explicit call is needed).
    fn get_extensibility_annotation(inner_pairs: pest::iterators::Pairs<Rule>) -> Option<&'static str> {
        for annotation_appl in inner_pairs.filter(|p| p.as_rule() == Rule::annotation_appl) {
            let inner = annotation_appl.into_inner();
            if let Some(scoped_name) = inner.clone().find(|p| p.as_rule() == Rule::scoped_name) {
                if let Some(ident) = scoped_name.into_inner().next() {
                    match ident.as_str() {
                        "appendable" => return Some("EXTENSIBILITY_KIND_APPENDABLE"),
                        "mutable" => return Some("EXTENSIBILITY_KIND_MUTABLE"),
                        "final" => return Some("EXTENSIBILITY_KIND_FINAL"),
                        _ => {}
                    }
                }
            }
        }
        None
    }

    /// Returns `(is_key, is_optional)` for a member based on its annotations.
    fn get_member_annotations(inner_pairs: pest::iterators::Pairs<Rule>) -> (bool, bool) {
        let mut is_key = false;
        let mut is_optional = false;
        for annotation_appl in inner_pairs.filter(|p| p.as_rule() == Rule::annotation_appl) {
            let inner = annotation_appl.into_inner();
            if let Some(scoped_name) = inner.clone().find(|p| p.as_rule() == Rule::scoped_name) {
                if let Some(ident) = scoped_name.into_inner().next() {
                    match ident.as_str() {
                        "key" => is_key = true,
                        "optional" => is_optional = true,
                        _ => {}
                    }
                }
            }
        }
        (is_key, is_optional)
    }

    fn struct_def(&mut self, pair: IdlPair) {
        let inner_pairs = pair.into_inner();
        let identifier = inner_pairs
            .clone()
            .find(|p| p.as_rule() == Rule::identifier)
            .expect("Identifier must exist according to the grammar");

        self.writer.push_str("    struct ");
        self.generate(identifier.clone());
        self.writer.push_str(" {\n");

        for member in inner_pairs.clone().filter(|p| p.as_rule() == Rule::member) {
            self.generate(member);
        }

        self.writer.push_str("    };\n");

        let struct_name = identifier.as_str();

        // --- get_type() ---
        self.writer
            .push_str("\n    static inline const DustDdsDynamicType* ");
        self.writer.push_str(struct_name);
        self.writer.push_str("_get_type(void) {\n");
        self.writer
            .push_str("        static const DustDdsDynamicType* type = NULL;\n");
        self.writer.push_str("        if (type == NULL) {\n");
        self.writer.push_str(&format!(
            "            DustDdsDynamicTypeBuilder* builder = dust_dds_dynamic_type_builder_create_struct(\"{}\");\n",
            struct_name
        ));

        // Emit extensibility setter if an annotation is present
        if let Some(ext) = Self::get_extensibility_annotation(inner_pairs.clone()) {
            self.writer.push_str(&format!(
                "            dust_dds_dynamic_type_builder_set_extensibility(builder, {});\n",
                ext
            ));
        }

        // Collect members with their types and annotation flags
        let mut members = Vec::new();
        let mut member_id = 0u32;
        for member in inner_pairs.clone().filter(|p| p.as_rule() == Rule::member) {
            let m_inner = member.into_inner();
            let type_spec = m_inner
                .clone()
                .find(|p| p.as_rule() == Rule::type_spec)
                .expect("Type spec must exist according to grammar");
            let declarators = m_inner
                .clone()
                .find(|p| p.as_rule() == Rule::declarators)
                .expect("Declarator must exist according to grammar");
            let (is_key, is_optional) =
                Self::get_member_annotations(m_inner.clone());

            for declarator in declarators.into_inner() {
                let array_or_simple_declarator = declarator
                    .into_inner()
                    .next()
                    .expect("Must have an element according to the grammar");
                let field_name = match array_or_simple_declarator.as_rule() {
                    Rule::simple_declarator => array_or_simple_declarator.as_str().to_string(),
                    _ => todo!(),
                };
                members.push((member_id, type_spec.clone(), field_name, is_key, is_optional));
                member_id += 1;
            }
        }

        // Emit add_member calls using DustDdsMemberDescriptor
        for (member_id, type_spec, field_name, is_key, is_optional) in &members {
            let type_expr = self.get_dynamic_type_expr(type_spec.clone());
            let needs_type_var = type_expr.contains("create_string_type")
                || type_expr.contains("_get_type");

            self.writer.push_str("            {\n");
            if needs_type_var {
                self.writer.push_str(&format!(
                    "                DustDdsDynamicType* member_type = {};\n",
                    type_expr
                ));
                self.writer.push_str(&format!(
                    "                DustDdsMemberDescriptor* member = dust_dds_member_descriptor_new(\"{}\", {}, member_type);\n",
                    field_name, member_id
                ));
            } else {
                self.writer.push_str(&format!(
                    "                DustDdsMemberDescriptor* member = dust_dds_member_descriptor_new(\"{}\", {}, {});\n",
                    field_name, member_id, type_expr
                ));
            }
            if *is_key {
                self.writer.push_str(
                    "                dust_dds_member_descriptor_set_is_key(member, true);\n",
                );
            }
            if *is_optional {
                self.writer.push_str(
                    "                dust_dds_member_descriptor_set_is_optional(member, true);\n",
                );
            }
            self.writer.push_str(
                "                dust_dds_dynamic_type_builder_add_member(builder, member);\n",
            );
            self.writer
                .push_str("                dust_dds_member_descriptor_free(member);\n");
            if needs_type_var {
                self.writer
                    .push_str("                dust_dds_dynamic_type_free(member_type);\n");
            }
            self.writer.push_str("            }\n");
        }

        self.writer
            .push_str("            type = dust_dds_dynamic_type_builder_build(builder);\n");
        self.writer.push_str("        }\n");
        self.writer.push_str("        return type;\n");
        self.writer.push_str("    }\n");

        // --- create_sample / create_dynamic_sample / free_sample ---
        let mut create_sample_code = String::new();
        let mut create_dynamic_sample_code = String::new();
        let mut free_sample_code = String::new();

        for (member_id, type_spec, field_name, _is_key, _is_optional) in &members {
            let (leaf_rule, leaf_str) = self.get_type_leaf(type_spec.clone());
            match leaf_rule {
                Rule::boolean_type => {
                    create_sample_code.push_str(&format!(
                        "        dust_dds_dynamic_data_get_boolean_value(src, {}, &sample.{});\n",
                        member_id, field_name
                    ));
                    create_dynamic_sample_code.push_str(&format!(
                        "            dust_dds_dynamic_data_set_boolean_value(sample, {}, src->{});\n",
                        member_id, field_name
                    ));
                }
                Rule::char_type => {
                    create_sample_code.push_str(&format!(
                        "        dust_dds_dynamic_data_get_char8_value(src, {}, &sample.{});\n",
                        member_id, field_name
                    ));
                    create_dynamic_sample_code.push_str(&format!(
                        "            dust_dds_dynamic_data_set_char8_value(sample, {}, src->{});\n",
                        member_id, field_name
                    ));
                }
                Rule::wide_char_type => {
                    create_sample_code.push_str(&format!(
                        "        {{\n            char temp;\n            dust_dds_dynamic_data_get_char8_value(src, {}, &temp);\n            sample.{} = (wchar_t)temp;\n        }}\n",
                        member_id, field_name
                    ));
                    create_dynamic_sample_code.push_str(&format!(
                        "            dust_dds_dynamic_data_set_char8_value(sample, {}, (char)src->{});\n",
                        member_id, field_name
                    ));
                }
                Rule::octet_type | Rule::unsigned_tiny_int => {
                    create_sample_code.push_str(&format!(
                        "        dust_dds_dynamic_data_get_uint8_value(src, {}, &sample.{});\n",
                        member_id, field_name
                    ));
                    create_dynamic_sample_code.push_str(&format!(
                        "            dust_dds_dynamic_data_set_uint8_value(sample, {}, src->{});\n",
                        member_id, field_name
                    ));
                }
                Rule::signed_tiny_int => {
                    create_sample_code.push_str(&format!(
                        "        dust_dds_dynamic_data_get_int8_value(src, {}, &sample.{});\n",
                        member_id, field_name
                    ));
                    create_dynamic_sample_code.push_str(&format!(
                        "            dust_dds_dynamic_data_set_int8_value(sample, {}, src->{});\n",
                        member_id, field_name
                    ));
                }
                Rule::signed_short_int => {
                    create_sample_code.push_str(&format!(
                        "        dust_dds_dynamic_data_get_int16_value(src, {}, &sample.{});\n",
                        member_id, field_name
                    ));
                    create_dynamic_sample_code.push_str(&format!(
                        "            dust_dds_dynamic_data_set_int16_value(sample, {}, src->{});\n",
                        member_id, field_name
                    ));
                }
                Rule::unsigned_short_int => {
                    create_sample_code.push_str(&format!(
                        "        dust_dds_dynamic_data_get_uint16_value(src, {}, &sample.{});\n",
                        member_id, field_name
                    ));
                    create_dynamic_sample_code.push_str(&format!(
                        "            dust_dds_dynamic_data_set_uint16_value(sample, {}, src->{});\n",
                        member_id, field_name
                    ));
                }
                Rule::signed_long_int => {
                    create_sample_code.push_str(&format!(
                        "        dust_dds_dynamic_data_get_int32_value(src, {}, &sample.{});\n",
                        member_id, field_name
                    ));
                    create_dynamic_sample_code.push_str(&format!(
                        "            dust_dds_dynamic_data_set_int32_value(sample, {}, src->{});\n",
                        member_id, field_name
                    ));
                }
                Rule::unsigned_long_int => {
                    create_sample_code.push_str(&format!(
                        "        dust_dds_dynamic_data_get_uint32_value(src, {}, &sample.{});\n",
                        member_id, field_name
                    ));
                    create_dynamic_sample_code.push_str(&format!(
                        "            dust_dds_dynamic_data_set_uint32_value(sample, {}, src->{});\n",
                        member_id, field_name
                    ));
                }
                Rule::signed_longlong_int => {
                    create_sample_code.push_str(&format!(
                        "        dust_dds_dynamic_data_get_int64_value(src, {}, &sample.{});\n",
                        member_id, field_name
                    ));
                    create_dynamic_sample_code.push_str(&format!(
                        "            dust_dds_dynamic_data_set_int64_value(sample, {}, src->{});\n",
                        member_id, field_name
                    ));
                }
                Rule::unsigned_longlong_int => {
                    create_sample_code.push_str(&format!(
                        "        dust_dds_dynamic_data_get_uint64_value(src, {}, &sample.{});\n",
                        member_id, field_name
                    ));
                    create_dynamic_sample_code.push_str(&format!(
                        "            dust_dds_dynamic_data_set_uint64_value(sample, {}, src->{});\n",
                        member_id, field_name
                    ));
                }
                Rule::floating_pt_type => {
                    if leaf_str == "float" {
                        create_sample_code.push_str(&format!(
                            "        dust_dds_dynamic_data_get_float32_value(src, {}, &sample.{});\n",
                            member_id, field_name
                        ));
                        create_dynamic_sample_code.push_str(&format!(
                            "            dust_dds_dynamic_data_set_float32_value(sample, {}, src->{});\n",
                            member_id, field_name
                        ));
                    } else if leaf_str == "double" {
                        create_sample_code.push_str(&format!(
                            "        dust_dds_dynamic_data_get_float64_value(src, {}, &sample.{});\n",
                            member_id, field_name
                        ));
                        create_dynamic_sample_code.push_str(&format!(
                            "            dust_dds_dynamic_data_set_float64_value(sample, {}, src->{});\n",
                            member_id, field_name
                        ));
                    } else {
                        panic!("long double not implemented yet");
                    }
                }
                Rule::string_type => {
                    create_sample_code.push_str(&format!(
                        "        dust_dds_dynamic_data_get_string_value(src, {}, &sample.{});\n",
                        member_id, field_name
                    ));
                    create_dynamic_sample_code.push_str(&format!(
                        "            dust_dds_dynamic_data_set_string_value(sample, {}, src->{});\n",
                        member_id, field_name
                    ));
                    free_sample_code.push_str(&format!(
                        "        dust_dds_string_free(sample->{});\n",
                        field_name
                    ));
                }
                Rule::wide_string_type => {
                    create_sample_code.push_str(&format!(
                        "        {{\n            char* temp = NULL;\n            dust_dds_dynamic_data_get_string_value(src, {}, &temp);\n            if (temp != NULL) {{\n                size_t len = mbstowcs(NULL, temp, 0);\n                if (len != (size_t)-1) {{\n                    sample.{} = malloc((len + 1) * sizeof(wchar_t));\n                    mbstowcs(sample.{}, temp, len + 1);\n                }}\n                dust_dds_string_free(temp);\n            }}\n        }}\n",
                        member_id, field_name, field_name
                    ));
                    create_dynamic_sample_code.push_str(&format!(
                        "            {{\n                if (src->{} != NULL) {{\n                    size_t len = wcstombs(NULL, src->{}, 0);\n                    if (len != (size_t)-1) {{\n                        char* temp = malloc(len + 1);\n                        wcstombs(temp, src->{}, len + 1);\n                        dust_dds_dynamic_data_set_string_value(sample, {}, temp);\n                        free(temp);\n                    }}\n                }}\n            }}\n",
                        field_name, field_name, field_name, member_id
                    ));
                    free_sample_code.push_str(&format!(
                        "        free(sample->{});\n",
                        field_name
                    ));
                }
                _ => {
                    // Custom identifier / nested struct
                    create_sample_code.push_str(&format!(
                        "        {{\n            DustDdsDynamicData* member_data = NULL;\n            dust_dds_dynamic_data_get_complex_value(src, {}, &member_data);\n            if (member_data != NULL) {{\n                sample.{} = {}_create_sample(member_data);\n                dust_dds_dynamic_data_free(member_data);\n            }}\n        }}\n",
                        member_id, field_name, leaf_str
                    ));
                    create_dynamic_sample_code.push_str(&format!(
                        "            {{\n                DustDdsDynamicData* member_data = {}_create_dynamic_sample(&src->{});\n                dust_dds_dynamic_data_set_complex_value(sample, {}, member_data);\n                dust_dds_dynamic_data_free(member_data);\n            }}\n",
                        leaf_str, field_name, member_id
                    ));
                    free_sample_code.push_str(&format!(
                        "        {}_free_sample(&sample->{});\n",
                        leaf_str, field_name
                    ));
                }
            }
        }

        self.writer.push_str(&format!(
            "\n    static inline struct {} {}_create_sample(DustDdsDynamicData* src) {{\n        struct {} sample;\n        memset(&sample, 0, sizeof(sample));\n{}        return sample;\n    }}\n",
            struct_name, struct_name, struct_name, create_sample_code
        ));

        self.writer.push_str(&format!(
            "\n    static inline DustDdsDynamicData* {}_create_dynamic_sample(const struct {}* src) {{\n        DustDdsDynamicData* sample = dust_dds_dynamic_data_create({}_get_type());\n        if (sample != NULL) {{\n{}        }}\n        return sample;\n    }}\n",
            struct_name, struct_name, struct_name, create_dynamic_sample_code
        ));

        self.writer.push_str(&format!(
            "\n    static inline void {}_free_sample(struct {}* sample) {{\n        if (sample != NULL) {{\n{}        }}\n    }}\n",
            struct_name, struct_name, free_sample_code
        ));

        self.writer.push_str(&format!(
            "\n    static inline ReturnCode dust_dds_datawriter_write_{}(DustDdsDataWriter* writer, const struct {}* data) {{\n        if (writer == NULL || data == NULL) {{\n            return RETCODE_BAD_PARAMETER;\n        }}\n        DustDdsDynamicData* sample = {}_create_dynamic_sample(data);\n        if (sample == NULL) {{\n            return RETCODE_ERROR;\n        }}\n        ReturnCode result = dust_dds_datawriter_write(writer, sample);\n        dust_dds_dynamic_data_free(sample);\n        return result;\n    }}\n",
            struct_name, struct_name, struct_name
        ));

        self.writer.push_str(&format!(
            "\n    static inline ReturnCode dust_dds_datareader_read_{}(DustDdsDataReader* reader, struct {}* data_values, int32_t max_samples, int32_t* received_samples) {{\n        if (reader == NULL || data_values == NULL || received_samples == NULL || max_samples <= 0) {{\n            return RETCODE_BAD_PARAMETER;\n        }}\n        DustDdsDynamicData** samples = (DustDdsDynamicData**)calloc(max_samples, sizeof(DustDdsDynamicData*));\n        if (samples == NULL) {{\n            return RETCODE_OUT_OF_RESOURCES;\n        }}\n        ReturnCode result = dust_dds_datareader_read(reader, samples, max_samples, received_samples);\n        if (result == RETCODE_OK) {{\n            for (int32_t i = 0; i < *received_samples; i++) {{\n                if (samples[i] != NULL) {{\n                    data_values[i] = {}_create_sample(samples[i]);\n                    dust_dds_dynamic_data_free(samples[i]);\n                }}\n            }}\n        }}\n        free(samples);\n        return result;\n    }}\n",
            struct_name, struct_name, struct_name
        ));
    }

    fn get_type_leaf(&self, type_spec: IdlPair) -> (Rule, String) {
        let mut current = type_spec;
        loop {
            match current.as_rule() {
                Rule::type_spec
                | Rule::simple_type_spec
                | Rule::base_type_spec
                | Rule::template_type_spec
                | Rule::integer_type
                | Rule::signed_int
                | Rule::unsigned_int => {
                    current = current
                        .into_inner()
                        .next()
                        .expect("Rule must have inner content");
                }
                rule => return (rule, current.as_str().to_string()),
            }
        }
    }

    fn get_dynamic_type_expr(&self, type_spec: IdlPair) -> String {
        let mut current = type_spec;
        loop {
            match current.as_rule() {
                Rule::type_spec
                | Rule::simple_type_spec
                | Rule::base_type_spec
                | Rule::template_type_spec
                | Rule::integer_type
                | Rule::signed_int
                | Rule::unsigned_int => {
                    current = current
                        .into_inner()
                        .next()
                        .expect("Rule must have inner content");
                }
                Rule::boolean_type => {
                    return "dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_BOOLEAN)"
                        .to_string();
                }
                Rule::char_type => {
                    return "dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_CHAR8)".to_string();
                }
                Rule::wide_char_type => {
                    return "dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_CHAR8)".to_string();
                }
                Rule::octet_type => {
                    return "dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_UINT8)".to_string();
                }
                Rule::signed_tiny_int => {
                    return "dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_INT8)".to_string();
                }
                Rule::unsigned_tiny_int => {
                    return "dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_UINT8)".to_string();
                }
                Rule::signed_short_int => {
                    return "dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_INT16)".to_string();
                }
                Rule::unsigned_short_int => {
                    return "dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_UINT16)".to_string();
                }
                Rule::signed_long_int => {
                    return "dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_INT32)".to_string();
                }
                Rule::unsigned_long_int => {
                    return "dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_UINT32)".to_string();
                }
                Rule::signed_longlong_int => {
                    return "dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_INT64)".to_string();
                }
                Rule::unsigned_longlong_int => {
                    return "dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_UINT64)".to_string();
                }
                Rule::floating_pt_type => match current.as_str() {
                    "float" => {
                        return "dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_FLOAT32)"
                            .to_string();
                    }
                    "double" => {
                        return "dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_FLOAT64)"
                            .to_string();
                    }
                    "long double" => {
                        return "dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_FLOAT128)"
                            .to_string();
                    }
                    _ => panic!("Invalid floating point type"),
                },
                Rule::string_type | Rule::wide_string_type => {
                    let bound = current
                        .into_inner()
                        .next()
                        .map(|p| p.as_str().to_string())
                        .unwrap_or_else(|| "4294967295".to_string());
                    return format!("dust_dds_dynamic_type_create_string_type({})", bound);
                }
                _ => {
                    return format!("(DustDdsDynamicType*){}_get_type()", current.as_str());
                }
            }
        }
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

        for declarator in declarators.into_inner() {
            let array_or_simple_declarator = declarator
                .into_inner()
                .next()
                .expect("Must have an element according to the grammar");
            self.writer.push_str("        ");
            self.generate(type_spec.clone());
            self.writer.push(' ');
            match array_or_simple_declarator.as_rule() {
                Rule::simple_declarator => {
                    self.generate(array_or_simple_declarator);
                }
                _ => todo!(),
            }
            self.writer.push_str(";\n");
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
            "float" => self.writer.push_str("float"),
            "double" => self.writer.push_str("double"),
            "long double" => self.writer.push_str("long double"),
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
        self.writer.push_str("int8_t")
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
        self.writer.push_str("int16_t")
    }

    #[inline]
    fn signed_long_int(&mut self, _pair: IdlPair) {
        self.writer.push_str("int32_t")
    }

    #[inline]
    fn signed_longlong_int(&mut self, _pair: IdlPair) {
        self.writer.push_str("int64_t")
    }

    #[inline]
    fn unsigned_tiny_int(&mut self, _pair: IdlPair) {
        self.writer.push_str("uint8_t")
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
        self.writer.push_str("uint16_t")
    }

    #[inline]
    fn unsigned_long_int(&mut self, _pair: IdlPair) {
        self.writer.push_str("uint32_t")
    }

    #[inline]
    fn unsigned_longlong_int(&mut self, _pair: IdlPair) {
        self.writer.push_str("uint64_t")
    }

    #[inline]
    fn octet_type(&mut self, _pair: IdlPair) {
        self.writer.push_str("uint8_t")
    }

    #[inline]
    fn template_type_spec(&mut self, pair: IdlPair) {
        self.generate(
            pair.into_inner()
                .next()
                .expect("Must have an element according to the grammar"),
        )
    }

    #[inline]
    fn string_type(&mut self, _pair: IdlPair) {
        self.writer.push_str("char*")
    }

    #[inline]
    fn wide_string_type(&mut self, _pair: IdlPair) {
        self.writer.push_str("wchar_t*")
    }

    #[inline]
    fn char_type(&mut self, _pair: IdlPair) {
        self.writer.push_str("char")
    }

    #[inline]
    fn wide_char_type(&mut self, _pair: IdlPair) {
        self.writer.push_str("wchar_t")
    }

    #[inline]
    fn boolean(&mut self, _pair: IdlPair) {
        self.writer.push_str("bool")
    }
}
