use super::analyser::*;

use crate::idl;
use crate::parser::Rule;

pub fn type_spec<'i>() -> AnalyserObject<'i, idl::Type> {
    within(
        rule(Rule::type_spec),
        match_pair(vec![
            (
                rule(Rule::simple_type_spec),
                simple_type().map(idl::Type::BaseType),
            ),
            (
                rule(Rule::template_type_spec),
                template_type().map(idl::Type::TemplateType),
            ),
        ]),
    )
}

fn simple_type<'i>() -> AnalyserObject<'i, idl::BaseType> {
    let integer_type = match_pair(vec![
        (
            rule(Rule::signed_int),
            match_pair(vec![
                (rule(Rule::signed_short_int), pure(idl::BaseType::Short)),
                (rule(Rule::signed_long_int), pure(idl::BaseType::Long)),
                (
                    rule(Rule::signed_longlong_int),
                    pure(idl::BaseType::LongLong),
                ),
            ]),
        ),
        (
            rule(Rule::unsigned_int),
            match_pair(vec![
                (
                    rule(Rule::unsigned_short_int),
                    pure(idl::BaseType::UnsignedShort),
                ),
                (
                    rule(Rule::unsigned_long_int),
                    pure(idl::BaseType::UnsignedLong),
                ),
                (
                    rule(Rule::unsigned_longlong_int),
                    pure(idl::BaseType::UnsignedLongLong),
                ),
            ]),
        ),
    ]);

    within(
        rule(Rule::base_type_spec),
        match_pair(vec![
            (rule(Rule::integer_type), integer_type),
            (
                rule(Rule::floating_pt_type),
                match_pair(vec![
                    (rule(Rule::float), pure(idl::BaseType::Float)),
                    (rule(Rule::double), pure(idl::BaseType::Double)),
                    (
                        rule(Rule::long_double),
                        fail("long double is not supported"),
                    ),
                ]),
            ),
            (rule(Rule::char_type), pure(idl::BaseType::Char)),
            (rule(Rule::wide_char_type), pure(idl::BaseType::WChar)),
            (rule(Rule::boolean_type), pure(idl::BaseType::Boolean)),
            (rule(Rule::octet_type), pure(idl::BaseType::Octet)),
        ]),
    )
}

fn template_type<'i>() -> AnalyserObject<'i, idl::TemplateType> {
    match_pair(vec![
        (
            rule(Rule::string_type),
            pure(idl::TemplateType::String(None)),
        ),
        (
            rule(Rule::wide_string_type),
            pure(idl::TemplateType::WideString(None)),
        ),
        (
            rule(Rule::sequence_type),
            lazy(|| type_spec().map(|t| idl::TemplateType::Sequence(Box::new(t), None))),
        ),
    ])
}
