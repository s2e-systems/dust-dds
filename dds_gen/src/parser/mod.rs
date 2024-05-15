use pest_derive::Parser;

#[derive(Parser)]
#[allow(clippy::empty_docs)]
#[grammar = "parser/idl_v4_grammar.pest"]
pub struct IdlParser;

pub type IdlPair<'i> = pest::iterators::Pair<'i, Rule>;
