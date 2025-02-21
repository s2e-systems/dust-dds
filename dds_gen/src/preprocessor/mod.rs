use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

pub struct Preprocessor {
    output: String,
}

impl Preprocessor {
    pub fn parse(idl_filepath: &Path) -> io::Result<String> {
        let mut preprocessor = Preprocessor {
            output: String::new(),
        };

        preprocessor.parse_file(idl_filepath)?;

        Ok(preprocessor.output)
    }

    fn parse_file(&mut self, idl_filepath: &Path) -> io::Result<()> {
        let idl_file = File::open(idl_filepath)?;
        let mut idl_file_reader = BufReader::new(idl_file);

        let mut line_buffer = String::new();
        loop {
            line_buffer.clear();
            let line_read = idl_file_reader.read_line(&mut line_buffer)?;
            if line_read == 0 {
                break;
            }

            let mut token_iter = line_buffer.split_whitespace();
            if let Some(first_token) = token_iter.next() {
                if first_token == "#include" {
                    let include_file_token = token_iter.next().ok_or(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Missing file argument for preprocessor directive #include",
                    ))?;
                    self.include_file(idl_filepath, include_file_token)?;
                    continue;
                } else if first_token.starts_with('#') {
                    return Err(std::io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Unknown preprocessor directive {first_token}"),
                    ));
                }
            }

            self.output.push_str(&line_buffer);
            self.output.push('\n');
        }

        Ok(())
    }

    fn include_file(&mut self, idl_filepath: &Path, include_file_token: &str) -> io::Result<()> {
        if include_file_token.len() > 2
            && ((include_file_token.starts_with('"') && include_file_token.ends_with('"'))
                || (include_file_token.starts_with('<') && include_file_token.ends_with('>')))
        {
            let include_filepath = idl_filepath
                .parent()
                .ok_or(io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to get parent path of IDL file",
                ))?
                .join(&include_file_token[1..include_file_token.len() - 1]);
            self.parse_file(&include_filepath)
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid #include filename enclosing characters. Expected \"\" or <>",
            ));
        }
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
            "struct SimpleStruct {\r\n\n    long i;\r\n\n};\nstruct SimpleStruct {\r\n\n    long i;\r\n\n};\n\r\n\nstruct OtherStruct {\r\n\n    long i;\r\n\n};\n";
        let output = Preprocessor::parse(idl_file).unwrap();

        assert_eq!(output, expected);
    }
}
