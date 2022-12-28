use super::analyser::*;
use super::types::*;

use crate::idl;
use crate::parser::Rule;

pub fn definition<'i>() -> AnalyserObject<'i, idl::Definition> {
    match_pair(vec![
        (
            rule(Rule::type_dcl),
            within(
                rule(Rule::constr_type_dcl),
                match_pair(vec![
                    (
                        rule(Rule::struct_dcl),
                        struct_dcl().map(idl::Definition::Struct),
                    ),
                    (rule(Rule::enum_dcl), enum_dcl().map(idl::Definition::Enum)),
                ]),
            ),
        ),
        (
            rule(Rule::module_dcl),
            module_dcl().map(idl::Definition::Module),
        ),
    ])
}

pub fn module_dcl<'i>() -> AnalyserObject<'i, idl::Module> {
    identifier().and_then(|name| {
        many_within(rule(Rule::definition), definition()).map(move |definitions| idl::Module {
            name: name.clone(),
            definitions,
        })
    })
}

fn struct_dcl<'i>() -> AnalyserObject<'i, idl::Struct> {
    within(
        rule(Rule::struct_def),
        identifier()
            .zip(many_within(rule(Rule::member), struct_member()))
            .map(|(name, members)| idl::Struct { name, members }),
    )
}

fn enum_dcl<'i>() -> AnalyserObject<'i, idl::Enum> {
    identifier()
        .zip(many_within(rule(Rule::enumerator), identifier()))
        .map(|(name, variants)| idl::Enum { name, variants })
}

fn struct_member<'i>() -> AnalyserObject<'i, idl::StructMember> {
    type_spec()
        .zip(declarators())
        .map(|(datatype, name)| idl::StructMember { datatype, name })
}

fn declarators<'i>() -> AnalyserObject<'i, String> {
    within(
        rule(Rule::declarators),
        within(
            rule(Rule::declarator),
            within(rule(Rule::simple_declarator), identifier()),
        ),
    )
}

fn identifier<'i>() -> AnalyserObject<'i, String> {
    rule(Rule::identifier).map(|pair| pair.as_str().to_string())
}
