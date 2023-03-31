use super::analyser::*;

use crate::idl::{self, Literal, UnaryOperator};
use crate::parser::Rule;

pub fn positive_int_const<'i>() -> AnalyserObject<'i, usize> {
    within(rule(Rule::const_expr), const_expr()).and_then(|l| match l {
        idl::Literal::Integer(i) if i >= 0 => pure(i as usize),
        l => fail(&format!(
            "Unexpected literal {:?} (expected positive integer)",
            l
        )),
    })
}

pub fn const_expr<'i>() -> AnalyserObject<'i, idl::Literal> {
    within(rule(Rule::unary_expr), unary_expr())
}

pub fn primary_expr<'i>() -> AnalyserObject<'i, idl::Literal> {
    within(
        rule(Rule::primary_expr),
        match_rule(vec![
            (Rule::literal, literal()),
            (Rule::const_expr, lazy(const_expr)),
        ]),
    )
}

pub fn unary_expr<'i>() -> AnalyserObject<'i, idl::Literal> {
    maybe(unary_operator())
        .zip(primary_expr())
        .and_then(|(op, l)| match (op, l) {
            (None, l) => pure(l),

            (Some(UnaryOperator::Minus), Literal::Integer(n)) => pure(Literal::Integer(-n)),
            (Some(UnaryOperator::Plus), Literal::Integer(n)) => pure(Literal::Integer(n)),

            (Some(UnaryOperator::Minus), Literal::Float(x)) => pure(Literal::Float(-x)),
            (Some(UnaryOperator::Plus), Literal::Float(x)) => pure(Literal::Float(x)),

            (Some(UnaryOperator::Not), Literal::Bool(b)) => pure(Literal::Bool(!b)),

            (Some(op), l) => fail(&format!("{:?} cannot be applied to {:?}", op, l)),
        })
}

pub fn unary_operator<'i>() -> AnalyserObject<'i, idl::UnaryOperator> {
    use idl::UnaryOperator::*;

    rule(Rule::unary_operator).and_then(|pair| match pair.as_str() {
        "-" => pure(Minus),
        "+" => pure(Plus),
        "~" => pure(Not),
        op => fail(&format!("Expected a unary operator, but got {:?}", op)),
    })
}

pub fn literal<'i>() -> AnalyserObject<'i, idl::Literal> {
    match_rule(vec![
        (Rule::integer_literal, integer_literal()),
        (Rule::floating_pt_literal, fail("Not implemented")),
        (Rule::fixed_pt_literal, fail("Not implemented")),
        (Rule::character_literal, fail("Not implemented")),
        (Rule::wide_character_literal, fail("Not implemented")),
        (Rule::boolean_literal, fail("Not implemented")),
        (Rule::string_literal, fail("Not implemented")),
        (Rule::wide_string_literal, fail("Not implemented")),
    ])
}

pub fn integer_literal<'i>() -> AnalyserObject<'i, idl::Literal> {
    rule(Rule::decimal_integer_literal).and_then(|p| match p.as_str().parse() {
        Ok(n) => pure(idl::Literal::Integer(n)),
        Err(e) => fail(&format!("Failed to parse integer {:?}: {}", p.as_str(), e)),
    })
}
