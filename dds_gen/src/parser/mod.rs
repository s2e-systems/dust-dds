use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parser/idl_v4_grammar.pest"]
pub struct IdlParser;

pub type IdlPair<'i> = pest::iterators::Pair<'i, Rule>;
