use crate::{transaction, Directive, Error};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::{
        complete::{line_ending, not_line_ending},
        streaming::space1,
    },
    combinator::{map, opt, value},
    sequence::{preceded, tuple},
    IResult,
};

use crate::directive::directive;

/// Parser of a beancount document
///
/// It is an iterator over the beancount directives.
///
/// See the crate documentation for usage example.
#[allow(missing_debug_implementations)]
pub struct Parser<'a> {
    rest: &'a str,
    tags: Vec<&'a str>,
    line: u64,
}

impl<'a> Parser<'a> {
    /// Create a new parser from the beancount string to parse
    #[must_use]
    pub fn new(content: &'a str) -> Self {
        Self {
            rest: content,
            tags: Vec::new(),
            line: 1,
        }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<Directive<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.rest.is_empty() {
            if let Ok((rest, chunk)) = chunk(self.rest) {
                self.line += 1;
                self.rest = rest;
                match chunk {
                    Chunk::Directive(mut directive) => {
                        if let Directive::Transaction(trx) = &mut directive {
                            self.line += trx.postings().len() as u64;
                            trx.append_tags(&self.tags);
                        }
                        return Some(Ok(directive));
                    }
                    Chunk::PushTag(tag) => self.tags.push(tag),
                    Chunk::PopTag(tag) => self.tags.retain(|&t| t != tag),
                    Chunk::Comment => (),
                }
            } else {
                self.rest = "";
                return Some(Err(Error::from_parsing(self.line)));
            }
        }
        None
    }
}

fn chunk(input: &str) -> IResult<&str, Chunk<'_>> {
    alt((
        map(directive, Chunk::Directive),
        map(pushtag, Chunk::PushTag),
        map(poptag, Chunk::PopTag),
        value(Chunk::Comment, tuple((not_line_ending, opt(line_ending)))),
    ))(input)
}

fn pushtag(input: &str) -> IResult<&str, &str> {
    preceded(tuple((tag("pushtag"), space1)), transaction::tag)(input)
}

fn poptag(input: &str) -> IResult<&str, &str> {
    preceded(tuple((tag("poptag"), space1)), transaction::tag)(input)
}

#[derive(Debug, Clone)]
enum Chunk<'a> {
    Directive(Directive<'a>),
    Comment,
    PushTag(&'a str),
    PopTag(&'a str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pushtag() {
        let input = "pushtag #test";
        let (_, chunk) = chunk(input).expect("should successfully parse the input");
        assert!(matches!(chunk, Chunk::PushTag("test")));
    }

    #[test]
    fn poptag() {
        let input = "poptag #test";
        let (_, chunk) = chunk(input).expect("should successfully parse the input");
        assert!(matches!(chunk, Chunk::PopTag("test")));
    }
}
