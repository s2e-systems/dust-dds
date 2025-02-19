use std::fmt::Write;

pub struct Preprocessor;

impl Preprocessor {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&mut self, idl_source: &str) -> Result<String, String> {
        let mut output = String::new();
        for line in idl_source.lines() {
            output.write_str(line).map_err(|e| e.to_string())?;
        }
        Ok(idl_source.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preprocessor_makes_no_changes() {
        let input = r#"
        struct A {
            i32 a
        };"#;

        let mut preprocessor = Preprocessor::new();

        let output = preprocessor.parse(input).unwrap();
        assert_eq!(input, output);
    }
}
