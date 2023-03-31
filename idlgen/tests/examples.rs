use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use dust_idlgen::compile_idl;

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
        let compiled_idl = compile_idl(&idl_src).expect("Failed to parse IDL file");

        for (expected_line, result_line) in
            expected_lines.zip(compiled_idl.lines().chain(std::iter::repeat("")))
        {
            assert_eq!(result_line, expected_line.unwrap())
        }
    }
}
