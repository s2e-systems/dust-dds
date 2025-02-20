use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

pub struct Preprocessor;

impl Preprocessor {
    pub fn parse(idl_filepath: &Path) -> io::Result<String> {
        let mut output = String::new();

        Self::parse_file(idl_filepath, &mut output)?;

        Ok(output)
    }

    fn parse_file(idl_filepath: &Path, output: &mut String) -> io::Result<()> {
        let idl_file = File::open(idl_filepath)?;
        let mut idl_file_reader = BufReader::new(idl_file);

        let mut line_buffer = String::new();
        loop {
            line_buffer.clear();
            let line_read = idl_file_reader.read_line(&mut line_buffer)?;
            if line_read == 0 {
                break;
            }

            output.push_str(&line_buffer);
            output.push('\n');
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preprocessor_makes_no_changes() {
        let idl_file = Path::new("src/preprocessor/test_resources/simple_struct.idl");
        let expected =
            "struct SimpleStruct {\r\n\n    boolean a;\r\n\n    char b;\r\n\n    long i;\r\n\n};\n";
        let output = Preprocessor::parse(idl_file).unwrap();

        assert_eq!(output, expected);
    }

    #[test]
    fn preprocessor_file_with_include() {
        let idl_file = Path::new("src/preprocessor/test_resources/file_with_include.idl");
        let expected =
            "struct SimpleStruct {\r\n\n    long i;\r\n\n};\nstruct SimpleStruct {\r\n\n    long i;\r\n\n};\nstruct OtherStruct {\r\n\n    long i;\r\n\n};\n";
        let output = Preprocessor::parse(idl_file).unwrap();

        assert_eq!(output, expected);
    }
}
