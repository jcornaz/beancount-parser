mod account;
mod currency;
mod date;
mod transaction;

pub use date::Date;
use nom::{
    branch::alt,
    bytes::{complete::tag, streaming::take_till},
    character::complete::{char, line_ending, not_line_ending, space0, space1},
    combinator::{all_consuming, cut, eof, iterator, map, opt},
    sequence::{delimited, preceded},
    Finish, Parser,
};
pub use transaction::Flag;

pub fn parse(input: &str) -> Result<BeancountFile<'_>, Error<'_>> {
    match all_consuming(beancount_file)(Span::new(input)).finish() {
        Ok((_, content)) => Ok(content),
        Err(nom::error::Error { input, .. }) => Err(Error(input)),
    }
}

#[derive(Debug)]
pub struct Error<'a>(Span<'a>);

#[derive(Debug)]
#[non_exhaustive]
pub struct BeancountFile<'a> {
    pub directives: Vec<Directive<'a>>,
}

#[derive(Debug)]
#[non_exhaustive]
pub struct Directive<'a> {
    pub date: Date,
    pub content: DirectiveContent<'a>,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum DirectiveContent<'a> {
    Transaction(transaction::Transaction<'a>),
    Open(account::Open<'a>),
    Close(account::Close<'a>),
}

type Span<'a> = nom_locate::LocatedSpan<&'a str>;
type IResult<'a, O> = nom::IResult<Span<'a>, O>;

fn beancount_file(input: Span<'_>) -> IResult<'_, BeancountFile<'_>> {
    let mut iter = iterator(input, alt((directive.map(Some), line.map(|_| None))));
    let directives = iter.flatten().collect();
    let (input, _) = iter.finish()?;
    Ok((input, BeancountFile { directives }))
}

fn directive(input: Span<'_>) -> IResult<'_, Directive<'_>> {
    let (input, date) = date::parse(input)?;
    let (input, content) = cut(directive_content)(input)?;
    Ok((input, Directive { date, content }))
}

fn directive_content(input: Span<'_>) -> IResult<'_, DirectiveContent<'_>> {
    let (input, _) = space1(input)?;
    let (input, content) = alt((
        map(transaction::parse, DirectiveContent::Transaction),
        map(
            preceded(tag("open"), cut(preceded(space1, account::open))),
            DirectiveContent::Open,
        ),
        map(
            preceded(tag("close"), cut(preceded(space1, account::close))),
            DirectiveContent::Close,
        ),
    ))(input)?;
    let (input, _) = end_of_line(input)?;
    Ok((input, content))
}

fn end_of_line(input: Span<'_>) -> IResult<'_, ()> {
    let (input, _) = space0(input)?;
    let (input, _) = opt(comment)(input)?;
    let (input, _) = alt((line_ending, eof))(input)?;
    Ok((input, ()))
}

fn comment(input: Span<'_>) -> IResult<'_, ()> {
    let (input, _) = char(';')(input)?;
    let (input, _) = not_line_ending(input)?;
    Ok((input, ()))
}

fn line(input: Span<'_>) -> IResult<'_, ()> {
    let (input, _) = not_line_ending(input)?;
    let (input, _) = line_ending(input)?;
    Ok((input, ()))
}

fn string(input: Span<'_>) -> IResult<'_, &str> {
    map(
        delimited(char('"'), take_till(|c: char| c == '"'), char('"')),
        |s: Span<'_>| *s.fragment(),
    )(input)
}
