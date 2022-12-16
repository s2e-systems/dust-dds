use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar/idl_v4.pest"]
pub struct IdlParser;