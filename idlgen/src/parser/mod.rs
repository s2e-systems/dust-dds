use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parser/idl_v4_grammar.pest"]
pub struct IdlParser;

pub type Pair<'i> = pest::iterators::Pair<'i, Rule>;
pub type Pairs<'i> = pest::iterators::Pairs<'i, Rule>;