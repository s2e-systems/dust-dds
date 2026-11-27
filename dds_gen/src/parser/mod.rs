use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parser/idl_v4_grammar.pest"]
pub struct IdlParser;

pub type IdlPair<'i> = pest::iterators::Pair<'i, Rule>;

#[cfg(test)]
mod tests {
    use super::*;
    use pest::Parser;

    fn parse_rules(input: &str) -> Vec<(Rule, String)> {
        IdlParser::parse(Rule::specification, input)
            .unwrap_or_else(|e| panic!("failed to parse `{input}`: {e}"))
            .flatten()
            .map(|p| (p.as_rule(), p.as_str().to_owned()))
            .collect()
    }

    #[test]
    fn parses_long_double_floating_pt_type() {
        let rules = parse_rules("struct S { long double value; };");
        assert!(
            rules
                .iter()
                .any(|(r, s)| *r == Rule::floating_pt_type && s == "long double"),
            "expected floating_pt_type `long double`, got {rules:?}"
        );
    }

    #[test]
    fn parses_wide_literals_without_breaking_identifier_l() {
        let wide_char = parse_rules("const wchar C = L'a';");
        assert!(
            wide_char
                .iter()
                .any(|(r, s)| *r == Rule::wide_character_literal && s == "L'a'"),
            "expected wide_character_literal, got {wide_char:?}"
        );

        let wide_str = parse_rules("const wstring W = L\"hello\";");
        assert!(
            wide_str
                .iter()
                .any(|(r, s)| *r == Rule::wide_string_literal && s == "L\"hello\""),
            "expected wide_string_literal, got {wide_str:?}"
        );

        let ident = parse_rules("const long L = 1; const long X = L;");
        assert!(
            ident
                .iter()
                .any(|(r, s)| *r == Rule::scoped_name && s == "L"),
            "expected scoped_name `L`, got {ident:?}"
        );
    }

    #[test]
    fn parses_adjacent_string_and_wide_string_literals() {
        let narrow = parse_rules("const string S = \"hello \" \"world\";");
        let narrow_parts: Vec<&str> = narrow
            .iter()
            .filter_map(|(r, s)| (*r == Rule::string_literal).then_some(s.as_str()))
            .collect();
        assert_eq!(narrow_parts, vec!["\"hello \"", "\"world\""]);

        let wide = parse_rules("const wstring W = L\"foo\" L\"bar\";");
        let wide_parts: Vec<&str> = wide
            .iter()
            .filter_map(|(r, s)| (*r == Rule::wide_string_literal).then_some(s.as_str()))
            .collect();
        assert_eq!(wide_parts, vec!["L\"foo\"", "L\"bar\""]);
    }
}
