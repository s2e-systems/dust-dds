use std::{
    iter::{self, from_fn, Peekable},
    str::Chars,
};

struct Lexer<'a> {
    cursor: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        let cursor = input.trim().chars().peekable();
        Self { cursor }
    }
}

enum Token {
    Identifier(String),
    Symbol(char),
    Whitespace,
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(c) = self.cursor.next() {
            if c.is_ascii_alphanumeric() || c == '_' {
                Some(Token::Identifier(
                    iter::once(c)
                        .chain(from_fn(|| {
                            self.cursor
                                .by_ref()
                                .next_if(|s| s.is_ascii_alphanumeric() || s == &'_')
                        }))
                        .collect(),
                ))
            } else if c.is_whitespace() {
                // Consume all consecutive whitespaces
                while let Some(s) = self.cursor.peek() {
                    if s.is_whitespace() {
                        self.cursor.next();
                    } else {
                        break;
                    }
                }

                Some(Token::Whitespace)
            } else {
                Some(Token::Symbol(c))
            }
        } else {
            None
        }
    }
}

pub fn preprocess_input(input: &str) -> Result<String, String> {
    let mut output = String::new();

    for line in input.lines() {
        let trimmed_line = line.trim();
        if trimmed_line.starts_with('#') {
            todo!()
        } else {
            let lexer = Lexer::new(trimmed_line);
            for token in lexer {
                match token {
                    Token::Identifier(i) => output.push_str(&i),
                    Token::Symbol(c) => output.push(c),
                    Token::Whitespace => output.push(' '),
                }
            }
            output.push('\n');
        }
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preprocessor_uniformizes_whitespaces() {
        let input = r#"
        This is a   TEST.
        This is another     test
"#;
        let expected = r#"
This is a TEST.
This is another test
"#;
        let result = preprocess_input(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn preprocessor_replaces_define() {
        let input = r#"
        #define TEST special_test

        This is a   TEST.
        This is another     test
"#;
        let expected = r#"
This is a special_test.
This is another test
"#;
        let result = preprocess_input(input).unwrap();
        assert_eq!(result, expected);
    }
}
