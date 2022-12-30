use pest::Span;

use crate::parser::{Pair, Pairs, Rule};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Error<'i> {
    message: String,
    span: Option<Span<'i>>,
}

impl<'i> Error<'i> {
    pub fn pretty_print(&self) {
        if let Some(span) = self.span {
            let (line, col) = span.start_pos().line_col();
            println!("At line {}, col {}:", line, col);
            for line in span.lines() {
                println!(">    {}", line);
            }
        }
        println!("{}", self.message);
    }
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct Input<'i> {
    pairs: Option<Pairs<'i>>,
}

impl<'i> From<Pairs<'i>> for Input<'i> {
    fn from(pairs: Pairs<'i>) -> Self {
        Input { pairs: Some(pairs) }
    }
}

impl<'i> Iterator for Input<'i> {
    type Item = Pair<'i>;

    fn next(&mut self) -> Option<Self::Item> {
        self.pairs.as_mut().and_then(|it| it.next())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Analysed<'i, T> {
    pub value: T,
    pub span: Option<Span<'i>>,
    pub next: Input<'i>,
}

impl<'i, T> Analysed<'i, T> {
    pub fn pure(value: T) -> Self {
        Analysed {
            value,
            span: None,
            next: Input::default(),
        }
    }
}

fn join_spans<'i>(a: Option<Span<'i>>, b: Option<Span<'i>>) -> Option<Span<'i>> {
    match (a, b) {
        (None, None) => None,
        (a, None) => a,
        (None, b) => b,
        (Some(a), Some(b)) => Some(a.start_pos().span(&b.end_pos())),
    }
}

pub type Result<'i, T> = core::result::Result<Analysed<'i, T>, Error<'i>>;

pub trait Analyser<'i, T> {
    fn analyse(&self, input: Input<'i>) -> Result<'i, T>;

    fn filter<Pred>(self, pred: Pred) -> AnalyserObject<'i, T>
    where
        Self: Sized + 'i,
        Pred: Fn(&T) -> bool + 'i,
    {
        AnalyserObject::new(move |input| {
            let analysed = self.analyse(input)?;
            if !pred(&analysed.value) {
                Err(Error {
                    message: "Predicate failed".to_string(),
                    span: analysed.span,
                })
            } else {
                Ok(analysed)
            }
        })
    }

    fn or<A>(self, other: A) -> AnalyserObject<'i, T>
    where
        Self: Sized + 'i,
        A: Analyser<'i, T> + 'i,
    {
        AnalyserObject::new(move |input| {
            self.analyse(input.clone())
                .or_else(|_| other.analyse(input))
        })
    }

    fn map<Map, U>(self, map: Map) -> AnalyserObject<'i, U>
    where
        Self: Sized + 'i,
        Map: Fn(T) -> U + 'i,
    {
        AnalyserObject::new(move |input| {
            let analysed = self.analyse(input)?;
            Ok(Analysed {
                value: map(analysed.value),
                span: analysed.span,
                next: analysed.next,
            })
        })
    }

    fn zip<A, U>(self, other: A) -> AnalyserObject<'i, (T, U)>
    where
        Self: Sized + 'i,
        A: Analyser<'i, U> + 'i,
    {
        AnalyserObject::new(move |input| {
            let fst = self.analyse(input)?;
            let snd = other.analyse(fst.next)?;

            Ok(Analysed {
                value: (fst.value, snd.value),
                span: join_spans(fst.span, snd.span),
                next: snd.next,
            })
        })
    }

    fn and_then<Bind, A, U>(self, bind: Bind) -> AnalyserObject<'i, U>
    where
        Self: Sized + 'i,
        Bind: Fn(T) -> A + 'i,
        A: Analyser<'i, U> + 'i,
    {
        AnalyserObject::new(move |input| {
            let analysed = self.analyse(input)?;
            bind(analysed.value).analyse(analysed.next)
        })
    }

    fn with_span(self, span: Option<Span<'i>>) -> AnalyserObject<'i, T>
    where
        Self: Sized + 'i,
    {
        AnalyserObject::new(move |input| {
            self.analyse(input)
                .map(|analysed| Analysed { span, ..analysed })
                .map_err(|err| Error { span, ..err })
        })
    }

    fn with_error(self, msg: &str) -> AnalyserObject<'i, T>
    where
        Self: Sized + 'i,
    {
        let msg = msg.to_string();
        AnalyserObject::new(move |input| {
            self.analyse(input).map_err(|err| Error {
                message: msg.clone(),
                ..err
            })
        })
    }

    fn with_prefix<A, U>(self, other: A) -> AnalyserObject<'i, T>
    where
        Self: Sized + 'i,
        T: 'i,
        U: 'i,
        A: Analyser<'i, U> + 'i,
    {
        other.zip(self).map(|(_, x)| x)
    }

    fn with_suffix<A, U>(self, other: A) -> AnalyserObject<'i, T>
    where
        Self: Sized + 'i,
        T: 'i,
        U: 'i,
        A: Analyser<'i, U> + 'i,
    {
        self.zip(other).map(|(x, _)| x)
    }
}

impl<'i, T, F> Analyser<'i, T> for F
where
    F: Fn(Input<'i>) -> Result<'i, T>,
{
    fn analyse(&self, input: Input<'i>) -> Result<'i, T> {
        self(input)
    }
}

pub struct AnalyserObject<'i, T> {
    analyser: Box<dyn Analyser<'i, T> + 'i>,
}

impl<'i, T> AnalyserObject<'i, T> {
    pub fn new<F>(analyse: F) -> Self
    where
        F: Fn(Input<'i>) -> Result<'i, T> + 'i,
    {
        AnalyserObject {
            analyser: Box::new(analyse),
        }
    }
}

impl<'i, T> Analyser<'i, T> for AnalyserObject<'i, T> {
    fn analyse(&self, input: Input<'i>) -> Result<'i, T> {
        self.analyser.analyse(input)
    }
}

pub fn pure<'i, T>(x: T) -> AnalyserObject<'i, T>
where
    T: Clone + 'i,
{
    AnalyserObject::new(move |input| {
        Ok(Analysed {
            value: x.clone(),
            span: None,
            next: input,
        })
    })
}

pub fn fail<'i, T>(msg: &str) -> AnalyserObject<'i, T> {
    let msg = msg.to_string();
    AnalyserObject::new(move |mut pairs| {
        Err(Error {
            message: msg.clone(),
            span: pairs.next().map(|p| p.as_span()),
        })
    })
}

pub fn lazy<'i, LazyA, A, T>(analyser: LazyA) -> AnalyserObject<'i, T>
where
    LazyA: Fn() -> A + 'i,
    A: Analyser<'i, T> + 'i,
{
    AnalyserObject::new(move |input| analyser().analyse(input))
}

pub fn eoi<'i>() -> AnalyserObject<'i, ()> {
    AnalyserObject::new(|input| {
        let not_analysed = input
            .clone()
            .map(|pair| Some(pair.as_span()))
            .fold(None, join_spans);

        if not_analysed.is_some() {
            Err(Error {
                message: "Incomplete analyser didn't analyse all content".to_string(),
                span: not_analysed,
            })
        } else {
            Ok(Analysed {
                value: (),
                span: None,
                next: input,
            })
        }
    })
}

pub fn next_pair<'i>() -> AnalyserObject<'i, Pair<'i>> {
    AnalyserObject::new(|mut pairs| {
        let pair = pairs.next().ok_or(Error {
            message: "Unexpected end of input".to_string(),
            span: None,
        })?;

        let span = pair.as_span();
        Ok(Analysed {
            value: pair,
            span: Some(span),
            next: pairs,
        })
    })
}

pub fn maybe<'i, A, T>(a: A) -> AnalyserObject<'i, Option<T>>
where
    A: Analyser<'i, T> + 'i,
    T: Clone + 'i,
{
    a.map(Some).or(pure(None))
}

pub fn many<'i, A, T>(a: A) -> AnalyserObject<'i, Vec<T>>
where
    A: Analyser<'i, T> + 'i,
{
    AnalyserObject::new(move |input| {
        let mut result = vec![];
        let mut span = None;
        let mut next = input;

        while let Ok(analysed) = a.analyse(next.clone()) {
            result.push(analysed.value);
            span = join_spans(span, analysed.span);
            next = analysed.next;
        }

        Ok(Analysed {
            value: result,
            span,
            next,
        })
    })
}

pub fn rule<'i>(rule: Rule) -> AnalyserObject<'i, Pair<'i>> {
    next_pair()
        .filter(move |pair| pair.as_rule() == rule)
        .or(next_pair()
            .with_error(&format!(
                "Expected rule: {:?}, but reached end of input",
                rule
            ))
            .and_then(move |pair| {
                fail(&format!(
                    "Expected rule: {:?} but got: {:?}",
                    rule,
                    pair.as_rule()
                ))
            }))
}

pub fn try_within_incomplete<'i, APair, A, T>(
    pair_analyser: APair,
    analyser: A,
) -> AnalyserObject<'i, Result<'i, T>>
where
    APair: Analyser<'i, Pair<'i>> + 'i,
    A: Analyser<'i, T> + 'i,
    T: Clone,
{
    pair_analyser.map(move |pair| analyser.analyse(pair.into_inner().into()))
}

pub fn try_within<'i, APair, A, T>(
    pair_analyser: APair,
    analyser: A,
) -> AnalyserObject<'i, Result<'i, T>>
where
    APair: Analyser<'i, Pair<'i>> + 'i,
    A: Analyser<'i, T> + 'i,
    T: Clone + 'i,
{
    try_within_incomplete(pair_analyser, analyser.with_suffix(eoi()))
}

pub fn extract<'i, A, T>(analyser: A) -> AnalyserObject<'i, T>
where
    A: Analyser<'i, Result<'i, T>> + 'i,
    T: Clone + 'i,
{
    analyser.and_then(move |result| match result {
        Ok(analysed) => pure(analysed.value).with_span(analysed.span),
        Err(e) => fail(&e.message).with_span(e.span),
    })
}

pub fn extract_vec<'i, A, T>(analyser: A) -> AnalyserObject<'i, Vec<T>>
where
    A: Analyser<'i, Vec<Result<'i, T>>> + 'i,
    T: Clone + 'i,
{
    analyser.and_then(move |results| {
        match results
            .into_iter()
            .collect::<core::result::Result<Vec<Analysed<T>>, Error>>()
        {
            Ok(analysed) => pure(analysed.iter().map(|a| a.value.clone()).collect()).with_span(
                analysed
                    .iter()
                    .fold(None, |span, a| join_spans(span, a.span)),
            ),
            Err(e) => fail(&e.message).with_span(e.span),
        }
    })
}

pub fn within<'i, APair, A, T>(pair_analyser: APair, analyser: A) -> AnalyserObject<'i, T>
where
    APair: Analyser<'i, Pair<'i>> + 'i,
    A: Analyser<'i, T> + 'i,
    T: Clone + 'i,
{
    extract(try_within(pair_analyser, analyser))
}

pub fn maybe_within<'i, APair, A, T>(
    pair_analyser: APair,
    analyser: A,
) -> AnalyserObject<'i, Option<T>>
where
    APair: Analyser<'i, Pair<'i>> + 'i,
    A: Analyser<'i, T> + 'i,
    T: Clone + 'i,
{
    extract(try_within(pair_analyser, analyser.map(Some)).or(pure(Ok(Analysed::pure(None)))))
}

pub fn many_within<'i, APair, A, T>(pair_analyser: APair, analyser: A) -> AnalyserObject<'i, Vec<T>>
where
    APair: Analyser<'i, Pair<'i>> + 'i,
    A: Analyser<'i, T> + 'i,
    T: Clone + 'i,
{
    extract_vec(many(try_within(pair_analyser, analyser)))
}

pub fn match_rule<'i, T>(cases: Vec<(Rule, AnalyserObject<'i, T>)>) -> AnalyserObject<'i, T>
where
    T: Clone + 'i,
{
    let rules: Vec<Rule> = cases.iter().map(|&(r, _)| r).collect();
    let rules2 = rules.clone();

    let match_fail = next_pair()
        .and_then(move |p| {
            fail(&format!(
                "Expected one of {:?} but got {:?} instead",
                rules,
                p.as_rule()
            ))
        })
        .or(fail(&format!(
            "Expected one of {:?} but reached end of input",
            rules2
        )));

    extract(
        cases
            .into_iter()
            .map(|(rule_name, analyser)| try_within(rule(rule_name), analyser))
            .chain([match_fail])
            .fold(fail("unreachable"), |a, b| a.or(b)),
    )
}
