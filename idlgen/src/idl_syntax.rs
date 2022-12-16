use pest::Span;

use crate::idl_parser;

pub type Rule = idl_parser::Rule;
pub type Token<'i> = pest::Token<'i, Rule>;
pub type Tokens<'i> = pest::iterators::Tokens<'i, Rule>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind<'i> {
    UnexpectedEOS,
    UnexpectedToken(Token<'i>),
    Unsupported(Rule),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error<'i> {
    what: ErrorKind<'i>,
    span: Option<Span<'i>>,
}

pub struct Analysed<'i, T> {
    pub value: T,
    pub span: Option<Span<'i>>,
    pub next: Tokens<'i>,
}
pub type AnalysisResult<'i, T> = Result<Analysed<'i, T>, Error<'i>>;

pub struct AnalyserObject<'i, T> {
    parser: Box<dyn Analyser<'i, T> + 'i>,
}

impl<'i, T> AnalyserObject<'i, T> {
    fn new<P>(parser: P) -> Self
    where
        P: Analyser<'i, T> + 'i,
    {
        Self {
            parser: Box::new(parser),
        }
    }
}

fn extend_span<'i>(span1: Option<Span<'i>>, span2: Option<Span<'i>>) -> Option<Span<'i>> {
    span1
        .zip(span2)
        .map(|(s1, s2)| s1.start_pos().span(&s2.end_pos()))
        .or(span1)
        .or(span2)
}

pub trait Analyser<'i, T> {
    fn analyse(&self, tokens: Tokens<'i>) -> AnalysisResult<'i, T>;

    fn filter<F>(self, f: F) -> AnalyserObject<'i, T>
    where
        Self: Sized + 'i,
        F: Fn(&T) -> bool + 'i,
    {
        AnalyserObject::new(move |mut tokens: Tokens<'i>| {
            let parsed = self.analyse(tokens.clone())?;
            if f(&parsed.value) {
                Ok(parsed)
            } else {
                Err(tokens
                    .next()
                    .map(|token| Error {
                        what: ErrorKind::UnexpectedToken(token),
                        span: parsed.span,
                    })
                    .unwrap_or(Error {
                        what: ErrorKind::UnexpectedEOS,
                        span: parsed.span,
                    }))
            }
        })
    }

    fn map<F, U>(self, f: F) -> AnalyserObject<'i, U>
    where
        Self: Sized + 'i,
        F: Fn(T) -> U + 'i,
    {
        AnalyserObject::new(move |tokens| {
            let parsed = self.analyse(tokens)?;
            Ok(Analysed {
                value: f(parsed.value),
                span: parsed.span,
                next: parsed.next,
            })
        })
    }

    fn zip<P, U>(self, other: P) -> AnalyserObject<'i, (T, U)>
    where
        Self: Sized + 'i,
        P: Analyser<'i, U> + 'i,
    {
        AnalyserObject::new(move |tokens| {
            let parsed1 = self.analyse(tokens)?;
            let parsed2 = other.analyse(parsed1.next)?;

            Ok(Analysed {
                value: (parsed1.value, parsed2.value),
                span: extend_span(parsed1.span, parsed2.span),
                next: parsed2.next,
            })
        })
    }

    fn and_then<F, P, U>(self, f: F) -> AnalyserObject<'i, U>
    where
        Self: Sized + 'i,
        F: Fn(T) -> P + 'i,
        P: Analyser<'i, U>,
    {
        AnalyserObject::new(move |tokens| {
            let parsed = self.analyse(tokens)?;
            let prev_span = parsed.span;
            f(parsed.value)
                .analyse(parsed.next)
                .map(|parsed| Analysed {
                    span: extend_span(prev_span, parsed.span),
                    ..parsed
                })
                .map_err(|err| Error {
                    span: extend_span(prev_span, err.span),
                    ..err
                })
        })
    }

    fn or<P>(self, other: P) -> AnalyserObject<'i, T>
    where
        Self: Sized + 'i,
        P: Analyser<'i, T> + 'i,
    {
        AnalyserObject::new(move |tokens: Tokens<'i>| {
            self.analyse(tokens.clone())
                .or_else(|_| other.analyse(tokens))
        })
    }
}

impl<'i, F, T> Analyser<'i, T> for F
where
    F: Fn(Tokens<'i>) -> AnalysisResult<'i, T> + 'i,
{
    fn analyse(&self, tokens: Tokens<'i>) -> AnalysisResult<'i, T> {
        self(tokens)
    }
}

impl<'i, T> Analyser<'i, T> for AnalyserObject<'i, T> {
    fn analyse(&self, tokens: Tokens<'i>) -> AnalysisResult<'i, T> {
        self.parser.analyse(tokens)
    }
}

fn pure<'i, T>(value: T) -> impl Analyser<'i, T>
where
    T: Clone + 'i,
{
    move |tokens| {
        Ok(Analysed {
            value: value.clone(),
            span: None,
            next: tokens,
        })
    }
}

fn fail<T>(err: ErrorKind) -> impl Analyser<T> {
    move |_| {
        Err(Error {
            what: err.clone(),
            span: None,
        })
    }
}

pub fn token<'i>() -> impl Analyser<'i, Token<'i>> {
    move |mut tokens: Tokens<'i>| {
        let token = tokens.next().ok_or(Error {
            what: ErrorKind::UnexpectedEOS,
            span: None,
        })?;
        let pos = match token {
            pest::Token::Start { pos, .. } => pos,
            pest::Token::End { pos, .. } => pos,
        };
        Ok(Analysed {
            value: token,
            span: Some(pos.span(&pos)),
            next: tokens,
        })
    }
}

fn start<'i>(expected_rule: Rule) -> impl Analyser<'i, ()> {
    token()
        .filter(
            move |token| matches!(token, pest::Token::Start { rule, .. } if rule == &expected_rule),
        )
        .map(|_| ())
}

fn end<'i>(expected_rule: Rule) -> impl Analyser<'i, ()> {
    token()
        .filter(
            move |token| matches!(token, pest::Token::End { rule, .. } if rule == &expected_rule),
        )
        .map(|_| ())
}

fn within<'i, P, T>(expected_rule: Rule, parser: P) -> impl Analyser<'i, T>
where
    P: Analyser<'i, T> + 'i,
    T: 'i,
{
    start(expected_rule)
        .zip(parser)
        .zip(end(expected_rule))
        .map(|((_, value), _)| value)
}

fn many<'i, P, T>(parser: P) -> impl Analyser<'i, Vec<T>>
where
    P: Analyser<'i, T> + 'i,
{
    move |tokens: Tokens<'i>| {
        let mut list = vec![];
        let mut span = None;
        let mut next = tokens;

        while let Ok(parsed) = parser.analyse(next.clone()) {
            list.push(parsed.value);
            span = extend_span(span, parsed.span);
            next = parsed.next;
        }

        Ok(Analysed {
            value: list,
            span,
            next,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseType {
    Float,
    Double,

    Short,
    Long,
    LongLong,

    UnsignedShort,
    UnsignedLong,
    UnsignedLongLong,

    Char,
    WChar,
    Boolean,
    Octet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    BaseType(BaseType),
    // ScopedName(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructMember {
    pub datatype: Type,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Struct {
    pub name: String,
    pub members: Vec<StructMember>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Enum {
    pub name: String,
    pub variants: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Definition {
    Struct(Struct),
    Module(Module),
    Enum(Enum),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub name: String,
    pub definitions: Vec<Definition>,
}

pub fn integer_type<'i>() -> impl Analyser<'i, BaseType> {
    within(
        Rule::signed_int,
        within(Rule::signed_short_int, pure(BaseType::Short))
            .or(within(Rule::signed_long_int, pure(BaseType::Long)))
            .or(within(Rule::signed_longlong_int, pure(BaseType::LongLong))),
    )
    .or(within(
        Rule::unsigned_int,
        within(Rule::unsigned_short_int, pure(BaseType::UnsignedShort))
            .or(within(
                Rule::unsigned_long_int,
                pure(BaseType::UnsignedLong),
            ))
            .or(within(
                Rule::unsigned_longlong_int,
                pure(BaseType::UnsignedLongLong),
            )),
    ))
}

pub fn base_type<'i>() -> impl Analyser<'i, BaseType> {
    within(
        Rule::base_type_spec,
        within(Rule::integer_type, integer_type())
            .or(within(
                Rule::floating_pt_type,
                within(Rule::float, pure(BaseType::Float))
                    .or(within(Rule::double, pure(BaseType::Double)))
                    .or(within(
                        Rule::long_double,
                        fail(ErrorKind::Unsupported(Rule::long_double)),
                    )),
            ))
            .or(within(Rule::char_type, pure(BaseType::Char)))
            .or(within(Rule::wide_char_type, pure(BaseType::WChar)))
            .or(within(Rule::boolean_type, pure(BaseType::Boolean)))
            .or(within(Rule::octet_type, pure(BaseType::Octet))),
    )
}

fn type_spec<'i>() -> impl Analyser<'i, Type> {
    within(
        Rule::type_spec,
        within(Rule::simple_type_spec, base_type().map(Type::BaseType)),
    )
}

pub fn identifier<'i>() -> impl Analyser<'i, String> {
    move |tokens| {
        let parsed_identifier = within(Rule::identifier, pure(())).analyse(tokens)?;
        Ok(Analysed {
            value: parsed_identifier
                .span
                .ok_or(Error {
                    what: ErrorKind::UnexpectedEOS,
                    span: None,
                })?
                .as_str()
                .to_string(),
            span: parsed_identifier.span,
            next: parsed_identifier.next,
        })
    }
}

fn declarators<'i>() -> impl Analyser<'i, String> {
    within(
        Rule::declarators,
        within(
            Rule::declarator,
            within(Rule::simple_declarator, identifier()),
        ),
    )
}

fn struct_member<'i>() -> impl Analyser<'i, StructMember> {
    within(
        Rule::member,
        type_spec()
            .zip(declarators())
            .map(|(datatype, name)| StructMember { datatype, name }),
    )
}

pub fn struct_dcl<'i>() -> impl Analyser<'i, Struct> {
    within(
        Rule::struct_dcl,
        within(
            Rule::struct_def,
            identifier()
                .zip(many(struct_member()))
                .map(|(name, members)| Struct { name, members }),
        ),
    )
}

pub fn enum_dcl<'i>() -> impl Analyser<'i, Enum> {
    within(
        Rule::enum_dcl,
        identifier()
            .zip(many(within(Rule::enumerator, identifier())))
            .map(|(name, variants)| Enum { name, variants }),
    )
}

pub fn definition<'i>() -> impl Analyser<'i, Definition> {
    let struct_dcl = within(Rule::type_dcl, within(Rule::constr_type_dcl, struct_dcl()));
    let enum_dcl = within(Rule::type_dcl, within(Rule::constr_type_dcl, enum_dcl()));

    within(
        Rule::definition,
        struct_dcl
            .map(Definition::Struct)
            .or(enum_dcl.map(Definition::Enum))
            .or(module().map(Definition::Module)),
    )
}

pub fn module<'i>() -> impl Analyser<'i, Module> {
    within(
        Rule::module_dcl,
        identifier().and_then(|name| {
            many(definition()).map(move |definitions| Module {
                name: name.clone(),
                definitions,
            })
        }),
    )
}

pub fn specification<'i>() -> impl Analyser<'i, Vec<Definition>> {
    within(
        Rule::specification,
        many(definition())
            .zip(within(Rule::EOI, pure(())))
            .map(|(definitions, _)| definitions),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use pest::Parser;
    use crate::idl_parser;

    #[test]
    fn test_analyse_type_spec() {
        let short = idl_parser::IdlParser::parse(idl_parser::Rule::type_spec, "short").unwrap();
        assert_eq!(
            type_spec().analyse(short.tokens()).unwrap().value,
            Type::BaseType(BaseType::Short)
        );

        let wchar = idl_parser::IdlParser::parse(idl_parser::Rule::type_spec, "wchar").unwrap();
        assert_eq!(
            type_spec().analyse(wchar.tokens()).unwrap().value,
            Type::BaseType(BaseType::WChar)
        );

        let double = idl_parser::IdlParser::parse(idl_parser::Rule::type_spec, "double").unwrap();
        assert_eq!(
            type_spec().analyse(double.tokens()).unwrap().value,
            Type::BaseType(BaseType::Double)
        );

        let unsigned_long_long =
            idl_parser::IdlParser::parse(idl_parser::Rule::type_spec, "unsigned long long")
                .unwrap();
        assert_eq!(
            type_spec()
                .analyse(unsigned_long_long.tokens())
                .unwrap()
                .value,
            Type::BaseType(BaseType::UnsignedLongLong)
        );
    }

    #[test]
    fn test_analyse_struct_member() {
        let member = idl_parser::IdlParser::parse(idl_parser::Rule::member, "octet x;").unwrap();
        assert_eq!(
            struct_member().analyse(member.tokens()).unwrap().value,
            StructMember {
                name: "x".to_string(),
                datatype: Type::BaseType(BaseType::Octet)
            },
        );

        let member =
            idl_parser::IdlParser::parse(idl_parser::Rule::member, "double the_variable;").unwrap();
        assert_eq!(
            struct_member().analyse(member.tokens()).unwrap().value,
            StructMember {
                name: "the_variable".to_string(),
                datatype: Type::BaseType(BaseType::Double)
            },
        );

        let member =
            idl_parser::IdlParser::parse(idl_parser::Rule::member, "unsigned long long toto;")
                .unwrap();
        assert_eq!(
            struct_member().analyse(member.tokens()).unwrap().value,
            StructMember {
                name: "toto".to_string(),
                datatype: Type::BaseType(BaseType::UnsignedLongLong)
            },
        );
    }

    #[test]
    fn test_analyse_struct_definition() {
        let struct_def =
            idl_parser::IdlParser::parse(idl_parser::Rule::definition, "struct Toto {};").unwrap();
        assert_eq!(
            definition().analyse(struct_def.tokens()).unwrap().value,
            Definition::Struct(Struct {
                name: "Toto".to_string(),
                members: vec![]
            })
        );

        let struct_def = idl_parser::IdlParser::parse(
            idl_parser::Rule::definition,
            "struct Titi { short a; char b; unsigned long long c; };",
        )
        .unwrap();
        assert_eq!(
            definition().analyse(struct_def.tokens()).unwrap().value,
            Definition::Struct(Struct {
                name: "Titi".to_string(),
                members: vec![
                    StructMember {
                        datatype: Type::BaseType(BaseType::Short),
                        name: "a".to_string()
                    },
                    StructMember {
                        datatype: Type::BaseType(BaseType::Char),
                        name: "b".to_string()
                    },
                    StructMember {
                        datatype: Type::BaseType(BaseType::UnsignedLongLong),
                        name: "c".to_string()
                    },
                ]
            })
        );
    }

    #[test]
    fn test_analyse_enum_definition() {
        let enum_def = idl_parser::IdlParser::parse(
            idl_parser::Rule::definition,
            "enum Suits { Spades, Hearts, Diamonds, Clubs };",
        )
        .unwrap();
        assert_eq!(
            definition().analyse(enum_def.tokens()).unwrap().value,
            Definition::Enum(Enum {
                name: "Suits".to_string(),
                variants: vec![
                    "Spades".to_string(),
                    "Hearts".to_string(),
                    "Diamonds".to_string(),
                    "Clubs".to_string(),
                ]
            })
        );
    }

    #[test]
    fn test_analyse_module_definition() {
        let module_src = [
            "module M {",
            "  struct A { char a; };",
            "  module N {",
            "    enum B { C, D };",
            "  };",
            "};",
        ]
        .concat();

        let enum_def =
            idl_parser::IdlParser::parse(idl_parser::Rule::definition, &module_src).unwrap();
        assert_eq!(
            definition().analyse(enum_def.tokens()).unwrap().value,
            Definition::Module(Module {
                name: "M".to_string(),
                definitions: vec![
                    Definition::Struct(Struct {
                        name: "A".to_string(),
                        members: vec![StructMember {
                            name: "a".to_string(),
                            datatype: Type::BaseType(BaseType::Char)
                        }]
                    }),
                    Definition::Module(Module {
                        name: "N".to_string(),
                        definitions: vec![Definition::Enum(Enum {
                            name: "B".to_string(),
                            variants: vec!["C".to_string(), "D".to_string()]
                        })]
                    })
                ]
            })
        );
    }
}
