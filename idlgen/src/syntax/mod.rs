mod analyser;
mod constants;
mod definition;
mod types;

use analyser::*;
use definition::definition;

use crate::idl;
use crate::parser::Rule;

pub use analyser::Analyser;

pub fn specification<'i>() -> AnalyserObject<'i, Vec<idl::Definition>> {
    within(
        rule(Rule::specification),
        many_within(rule(Rule::definition), definition()).with_suffix(rule(Rule::EOI)),
    )
    .with_suffix(eoi())
}