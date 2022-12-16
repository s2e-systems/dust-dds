use std::{
    fs::File,
    io::{BufRead, BufReader},
    process::Command,
};

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

        let cmd_result = Command::new("cargo")
            .args(["run", idl_path.to_str().unwrap()])
            .output()
            .expect("(;_;) Couldn't run command");

        let expected_lines = BufReader::new(rs_file).lines();
        let result_lines = std::str::from_utf8(&cmd_result.stdout)
            .expect("(;_;) Program produced invalid UTF-8")
            .lines();

        for (result_line, expected_line) in result_lines.zip(expected_lines) {
            assert_eq!(result_line, expected_line.unwrap())
        }
    }
}
