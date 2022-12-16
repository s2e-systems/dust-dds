pub mod idl_parser;
pub mod idl_syntax;
pub mod rust_mapping;

// fn main() {
//     let args: Vec<_> = std::env::args().collect();
//     let idl_path = args.get(1).expect("(;_;) No idl source given");
//     let idl_src = std::fs::read_to_string(idl_path).expect("(;_;) Couldn't read IDL source file!");

//     let result = idl_parser::IdlParser::parse(idl_parser::Rule::specification, &idl_src)
//         .expect("Couldn't parse IDL file");

//     let spec = idl_syntax::specification()
//         .analyse(result.tokens())
//         .expect("Couldn't analyse IDL syntax");

//     for def in spec.value {
//         for line in rust_mapping::definition(def) {
//             println!("{}", line)
//         }
//     }
// }
