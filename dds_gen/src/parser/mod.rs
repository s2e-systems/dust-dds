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
    let file = IdlParser::parse(Rule::specification, source)?
        .next()
        .unwrap();

    println!("\n== Top-Level Parse: ==");
    println!("Rule: {:?}", file.as_rule());

    let mut structs = Vec::new();

    for definition in file.into_inner() {
        println!("\n-- Top-Level Child: {:?}", definition.as_rule());

        if definition.as_rule() != Rule::definition {
            println!("Skipped top-level rule: {:?}", definition.as_rule());
            continue;
        }

        let mut inner_rules = definition.into_inner().peekable();

        // Collect top-level annotations (e.g., @derive)
        let mut struct_annotations = Vec::new();
        while let Some(ann) = inner_rules.peek() {
            if ann.as_rule() == Rule::annotation_appl {
                println!("Found annotation: {:?}", ann.as_str());
                struct_annotations.push(parse_annotation(inner_rules.next().unwrap()));
            } else {
                break;
            }
        }

        // After annotations, the next rule should be type_dcl (or other definitions)
        if let Some(type_dcl) = inner_rules.next() {
            println!("Next rule after annotations: {:?}", type_dcl.as_rule());

            if type_dcl.as_rule() == Rule::type_dcl {
                // Drill down into constr_type_dcl
                for constr_type in type_dcl.into_inner() {
                    println!("Inside type_dcl: {:?}", constr_type.as_rule());

                    if constr_type.as_rule() == Rule::constr_type_dcl {
                        // Drill down into struct_dcl
                        for struct_dcl in constr_type.into_inner() {
                            println!("Inside constr_type_dcl: {:?}", struct_dcl.as_rule());

                            if struct_dcl.as_rule() == Rule::struct_dcl {
                                // Drill down into struct_def
                                for struct_def in struct_dcl.into_inner() {
                                    println!("Inside struct_dcl: {:?}", struct_def.as_rule());

                                    if struct_def.as_rule() == Rule::struct_def {
                                        let mut parsed_struct = parse_struct(struct_def.clone());
                                        // Merge annotations collected outside struct_def, if any
                                        parsed_struct.annotations.extend(struct_annotations.clone());

                                        println!("Parsed struct: {:?}", parsed_struct.name);
                                        structs.push(parsed_struct);
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                println!("Expected type_dcl but found: {:?}", type_dcl.as_rule());
            }
        } else {
            println!("No type_dcl found inside definition");
        }
    }

    println!("\n== Finished Parsing. Structs Parsed: {} ==\n", structs.len());

    Ok(structs)
}





fn parse_struct(pair: IdlPair) -> StructDef {
    let mut annotations = Vec::new();
    let mut name = String::new();
    let mut members = Vec::new();

    for inner in pair.into_inner() {
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
        match inner.as_rule() {
            Rule::annotation_appl => {
                annotations.push(parse_annotation(inner));
            }
            Rule::type_spec => {
                idl_type = inner.as_str().to_string();
            }
            Rule::declarators => {
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
