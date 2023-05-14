mod date;

pub use date::Date;
use nom::{multi::many0, Finish};

pub fn parse(input: &str) -> Result<BeancountFile, Error<'_>> {
    match beancount_file(Span::new(input)).finish() {
        Ok((_, content)) => Ok(content),
        Err(nom::error::Error { input, .. }) => Err(Error(input)),
    }
}

#[derive(Debug)]
pub struct Error<'a>(Span<'a>);

#[derive(Debug)]
#[non_exhaustive]
pub struct BeancountFile {
    pub directives: Vec<Directive>,
}

#[derive(Debug)]
#[non_exhaustive]
pub struct Directive {
    date: Date,
}

impl Directive {
    #[must_use]
    pub fn date(&self) -> Date {
        self.date
    }
}

type Span<'a> = nom_locate::LocatedSpan<&'a str>;
type IResult<'a, O> = nom::IResult<Span<'a>, O>;

fn beancount_file(input: Span<'_>) -> IResult<'_, BeancountFile> {
    let (input, directives) = many0(directive)(input)?;
    Ok((input, BeancountFile { directives }))
}

fn directive(input: Span<'_>) -> IResult<'_, Directive> {
    let (input, date) = date::parse(input)?;
    Ok((input, Directive { date }))
}
