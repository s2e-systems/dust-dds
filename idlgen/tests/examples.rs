use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use dust_idlgen::{
    parser,
    syntax::{self, Analyser},
    mappings::rust,
};
use pest::Parser;

#[test]
fn verify_examples() {
    let idl_files = std::fs::read_dir("tests/examples")
        .expect("(;_;) examples folder not found; this test should be run in the crate directory")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.file_type().ok()?.is_file() && entry.path().extension()? == "idl" {
                Some(entry.path())
            } else {
                None
            }
        });

    for idl_path in idl_files {
        let rs_path = idl_path.with_extension("rs");

        let rs_file = File::open(&rs_path).expect(&format!(
            "(;_;) Couldn't find corresponding file {:?} to example {:?}",
            rs_path, idl_path
        ));
        let expected_lines = BufReader::new(rs_file).lines();

        let idl_src = std::fs::read_to_string(idl_path.clone())
            .expect("(;_;) Couldn't read IDL source file!");
        let parsed_idl = parser::IdlParser::parse(parser::Rule::specification, &idl_src)
            .expect(&format!("(;_;) Parse error in {:?}", idl_path));
        let idl_spec = syntax::specification()
            .analyse(parsed_idl)
            .expect(&format!("(;_;) Syntax error in {:?}", idl_path));
        let result_lines = idl_spec
            .value
            .into_iter()
            .flat_map(rust::definition);

        for (expected_line, result_line) in
            expected_lines.zip(result_lines.chain(std::iter::repeat("".to_string())))
        {
            assert_eq!(result_line, expected_line.unwrap())
        }
    }
}
