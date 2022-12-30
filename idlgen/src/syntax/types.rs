use super::analyser::*;
use super::constants::*;

use crate::idl;
use crate::parser::Rule;

pub fn type_spec<'i>() -> AnalyserObject<'i, idl::Type> {
    within(
        rule(Rule::type_spec),
        match_rule(vec![
            (
                Rule::simple_type_spec,
                simple_type().map(idl::Type::BaseType),
            ),
            (
                Rule::template_type_spec,
                template_type().map(idl::Type::TemplateType),
            ),
        ]),
    )
}

fn simple_type<'i>() -> AnalyserObject<'i, idl::BaseType> {
    let integer_type = match_rule(vec![
        (
            Rule::signed_int,
            match_rule(vec![
                (Rule::signed_short_int, pure(idl::BaseType::Short)),
                (Rule::signed_long_int, pure(idl::BaseType::Long)),
                (Rule::signed_longlong_int, pure(idl::BaseType::LongLong)),
            ]),
        ),
        (
            Rule::unsigned_int,
            match_rule(vec![
                (Rule::unsigned_short_int, pure(idl::BaseType::UnsignedShort)),
                (Rule::unsigned_long_int, pure(idl::BaseType::UnsignedLong)),
                (
                    Rule::unsigned_longlong_int,
                    pure(idl::BaseType::UnsignedLongLong),
                ),
            ]),
        ),
    ]);

    within(
        rule(Rule::base_type_spec),
        match_rule(vec![
            (Rule::integer_type, integer_type),
            (
                Rule::floating_pt_type,
                match_rule(vec![
                    (Rule::float, pure(idl::BaseType::Float)),
                    (Rule::double, pure(idl::BaseType::Double)),
                    (Rule::long_double, fail("long double is not supported")),
                ]),
            ),
            (Rule::char_type, pure(idl::BaseType::Char)),
            (Rule::wide_char_type, pure(idl::BaseType::WChar)),
            (Rule::boolean_type, pure(idl::BaseType::Boolean)),
            (Rule::octet_type, pure(idl::BaseType::Octet)),
        ]),
    )
}

fn template_type<'i>() -> AnalyserObject<'i, idl::TemplateType> {
    match_rule(vec![
        (
            Rule::string_type,
            maybe_within(rule(Rule::positive_int_const), positive_int_const())
                .map(idl::TemplateType::String),
        ),
        (
            Rule::wide_string_type,
            maybe_within(rule(Rule::positive_int_const), positive_int_const())
                .map(idl::TemplateType::WideString),
        ),
        (
            Rule::sequence_type,
            lazy(|| {
                type_spec()
                    .zip(maybe_within(
                        rule(Rule::positive_int_const),
                        positive_int_const(),
                    ))
                    .map(|(t, size)| idl::TemplateType::Sequence(Box::new(t), size))
            }),
        ),
    ])
}
