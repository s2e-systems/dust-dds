use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read},
    path::Path,
};

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "preprocessor/preprocessor_grammar.pest"]
pub struct IdlPreprocessorParser;

pub struct Preprocessor<'a> {
    idl_file_path: &'a Path,
    output: String,
    define_list: HashMap<String, String>,
}

impl<'a> Preprocessor<'a> {
    pub fn parse(idl_filepath: &'a Path) -> io::Result<String> {
        let mut preprocessor = Preprocessor {
            idl_file_path: idl_filepath,
            output: String::new(),
            define_list: HashMap::new(),
        };

        preprocessor.parse_file(idl_filepath)?;

        Ok(preprocessor.output)
    }

    fn parse_file(&mut self, idl_filepath: &Path) -> io::Result<()> {
        let mut idl_file = File::open(idl_filepath)?;
        let mut idl_file_contents = String::new();
        idl_file.read_to_string(&mut idl_file_contents)?;
        // Make sure file always ends in a newline for correct parsing
        idl_file_contents.push('\n');

        let mut parsed_idl = IdlPreprocessorParser::parse(Rule::file, &idl_file_contents)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

        if let Some(p) = parsed_idl.next() {
            self.generate_preprocessed_idl(p)?;
        }

        Ok(())
    }

    fn generate_preprocessed_idl(
        &mut self,
        pair: pest::iterators::Pair<'_, Rule>,
    ) -> io::Result<()> {
        match pair.as_rule() {
            Rule::file => {
                for l in pair.into_inner() {
                    self.generate_preprocessed_idl(l)?;
                }
            }
            Rule::line => {
                if let Some(p) = pair.into_inner().next() {
                    self.generate_preprocessed_idl(p)?;
                }
            }
            Rule::include_file => {
                let include_filename = pair.as_str();
                let include_filepath = self
                    .idl_file_path
                    .parent()
                    .ok_or(io::Error::new(
                        io::ErrorKind::Other,
                        "Failed to get parent path of IDL file",
                    ))?
                    .join(&include_filename[1..include_filename.len() - 1]);

                self.parse_file(&include_filepath)?;
            }
            Rule::define_directive => {
                let mut define_pairs = pair.into_inner();
                let define_identifier = define_pairs
                    .next()
                    .expect("#define identifier must exist according to grammar")
                    .as_str()
                    .to_string();
                let define_value = define_pairs
                    .next()
                    .map(|r| r.as_str().to_string())
                    .unwrap_or_default();
                self.define_list.insert(define_identifier, define_value);
            }
            Rule::ifdef_directive => {
                let mut ifdef_pairs = pair.into_inner();
                let ifdef_identifier = ifdef_pairs
                    .next()
                    .expect("#ifdef identifier must exist according to grammar")
                    .as_str();
                if self.define_list.contains_key(ifdef_identifier) {
                    for ifdef_line in ifdef_pairs {
                        self.generate_preprocessed_idl(ifdef_line)?;
                    }
                }
            }
            Rule::ifndef_directive => {
                let mut ifdef_pairs = pair.into_inner();
                let ifdef_identifier = ifdef_pairs
                    .next()
                    .expect("#ifndef identifier must exist according to grammar")
                    .as_str();
                if !self.define_list.contains_key(ifdef_identifier) {
                    for ifdef_line in ifdef_pairs {
                        self.generate_preprocessed_idl(ifdef_line)?;
                    }
                }
            }
            Rule::other_line => {
                let mut line = pair.as_str().to_string();
                // Replace all occurences of the define macro by their values. The order of this
                // iteration is not guaranteed
                for (define_macro, define_value) in self.define_list.iter() {
                    line = line.replace(define_macro.as_str(), define_value);
                }
                self.output.push_str(&line);
                self.output.push('\n');
            }
            Rule::directive
            | Rule::include_directive
            | Rule::quoted_string
            | Rule::angle_bracketed_string
            | Rule::identifier
            | Rule::value
            | Rule::WHITESPACE
            | Rule::NEWLINE
            | Rule::EOI => (),
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
        let expected = "struct SimpleStruct {\nboolean a;\nchar b;\nlong i;\n};\n";
        let output = Preprocessor::parse(idl_file).unwrap();

        assert_eq!(output, expected);
    }

    #[test]
    fn preprocessor_file_with_include() {
        let idl_file = Path::new("src/preprocessor/test_resources/file_with_include.idl");
        let expected =
            "struct SimpleStruct {\nlong i;\n};\nstruct SimpleStruct {\nlong i;\n};\n\nstruct OtherStruct {\nlong i;\n};\n";
        let output = Preprocessor::parse(idl_file).unwrap();

        assert_eq!(output, expected);
    }

    #[test]
    fn preprocessor_file_with_define() {
        let idl_file = Path::new("src/preprocessor/test_resources/file_with_define.idl");
        let expected = "\nstruct SimpleStruct {\nboolean a;\nchar b;\nlong i;\n};\n";
        let output = Preprocessor::parse(idl_file).unwrap();

        assert_eq!(output, expected);
    }

    #[test]
    fn preprocessor_file_with_ifdef() {
        let idl_file = Path::new("src/preprocessor/test_resources/file_with_ifdef.idl");
        let expected = "\nstruct SimpleStruct {\nboolean a;\nchar b;\nlong i;\n};\n\n";
        let output = Preprocessor::parse(idl_file).unwrap();

        assert_eq!(output, expected);
    }

    #[test]
    fn preprocessor_file_with_ifdef_not_defined() {
        let idl_file = Path::new("src/preprocessor/test_resources/file_with_ifdef_not_defined.idl");
        let expected = "";
        let output = Preprocessor::parse(idl_file).unwrap();

        assert_eq!(output, expected);
    }

    #[test]
    fn preprocessor_file_with_ifndef() {
        let idl_file = Path::new("src/preprocessor/test_resources/file_with_ifndef.idl");
        let expected = "struct SimpleStruct {\nboolean a;\nchar b;\nlong i;\n};\n\n";
        let output = Preprocessor::parse(idl_file).unwrap();

        assert_eq!(output, expected);
    }

    #[test]
    fn preprocessor_file_with_ifndef_defined() {
        let idl_file = Path::new("src/preprocessor/test_resources/file_with_ifndef_defined.idl");
        let expected = "";
        let output = Preprocessor::parse(idl_file).unwrap();

        assert_eq!(output, expected);
    }
}
