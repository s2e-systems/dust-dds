use pest_derive::Parser;
use pest::Parser as _; // for `parse`

#[derive(Parser)]
#[grammar = "parser/idl_v4_grammar.pest"]
pub struct IdlParser;

pub type IdlPair<'i> = pest::iterators::Pair<'i, Rule>;

#[derive(Debug, Clone)]
pub struct Annotation {
    pub name: String,
    pub parameters: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Member {
    pub name: String,
    pub idl_type: String,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub annotations: Vec<Annotation>,
    pub members: Vec<Member>,
}

pub fn parse_idl(source: &str) -> Result<Vec<StructDef>, pest::error::Error<Rule>> {
    println!("Unwrapping parse file");
    let file = IdlParser::parse(Rule::specification, source)?
        .next()
        .unwrap();

    println!("\n== Top-Level Parse: ==");
    println!("Rule: {:?}", file.as_rule());

    let mut structs = Vec::new();


    Ok(structs)
}





fn parse_struct(pair: IdlPair) -> StructDef {
    let mut annotations = Vec::new();
    let mut name = String::new();
    let mut members = Vec::new();

    for inner in pair.into_inner() {
        println!("Parsing rule struct: {:?}", inner.as_rule());
        match inner.as_rule() {
            Rule::annotation_appl => {
                annotations.push(parse_annotation(inner));
            }
            Rule::identifier => {
                name = inner.as_str().to_string();
            }
            Rule::member => {
                members.push(parse_member(inner));
            }
            _ => {}
        }
    }

    StructDef {
        name,
        annotations,
        members,
    }
}

fn parse_member(pair: IdlPair) -> Member {
    let mut annotations = Vec::new();
    let mut idl_type = String::new();
    let mut name = String::new();

    for inner in pair.into_inner() {
        println!("Parsing rule member: {:?}", inner.as_rule());

        match inner.as_rule() {
            Rule::annotation_appl => {
                annotations.push(parse_annotation(inner));
            }
            Rule::type_spec => {
                println!("type_spec: '{}'", inner.as_str());
                idl_type = inner.as_str().to_string();
            }
            Rule::declarators => {
                for child in inner.clone().into_inner() {
                    println!("Declarator child rule: {:?}, text: '{}'", child.as_rule(), child.as_str());
                }
                name = inner.into_inner().next().unwrap().as_str().to_string();
            }
            _ => {}
        }
    }

    Member {
        name,
        idl_type,
        annotations,
    }
}

fn parse_annotation(pair: IdlPair) -> Annotation {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().trim().to_string();
    let mut parameters = Vec::new();

    if let Some(param_group) = inner.next() {
        println!("Parsing rule annotation: {:?}", param_group.as_rule());
        match param_group.as_rule() {
            Rule::annotation_appl_params => {
                for p in param_group.into_inner() {
                    parameters.push(p.as_str().trim_matches('"').to_string());
                }
            }
            Rule::annotation_appl_param => {
                parameters.push(param_group.as_str().trim_matches('"').to_string());
            }
            _ => {
                parameters.push(param_group.as_str().to_string());
            }
        }
    }

    Annotation { name, parameters }
}
