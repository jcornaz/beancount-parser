mod account;
mod date;
mod open;

pub use date::Date;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::space1,
    combinator::map,
    multi::many0,
    sequence::{preceded, tuple},
    Finish,
};

pub fn parse(input: &str) -> Result<BeancountFile<'_>, Error<'_>> {
    match beancount_file(Span::new(input)).finish() {
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
    Open(open::Open<'a>),
}

type Span<'a> = nom_locate::LocatedSpan<&'a str>;
type IResult<'a, O> = nom::IResult<Span<'a>, O>;

fn beancount_file(input: Span<'_>) -> IResult<'_, BeancountFile<'_>> {
    let (input, directives) = many0(directive)(input)?;
    Ok((input, BeancountFile { directives }))
}

fn directive(input: Span<'_>) -> IResult<'_, Directive<'_>> {
    let (input, date) = date::parse(input)?;
    let (input, content) = directive_content(input)?;
    Ok((input, Directive { date, content }))
}

fn directive_content(input: Span<'_>) -> IResult<'_, DirectiveContent<'_>> {
    let (input, _) = space1(input)?;
    alt((map(
        preceded(tuple((tag("open"), space1)), open::parse),
        DirectiveContent::Open,
    ),))(input)
}
