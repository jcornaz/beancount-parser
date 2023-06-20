use nom::{
    branch::alt,
    bytes::complete::{tag, take_till1, take_while1},
    character::complete::{char, not_line_ending},
    combinator::{map, value},
    sequence::preceded,
    Parser,
};

use crate::{IResult, Span};

pub(crate) fn string(input: Span<'_>) -> IResult<'_, String> {
    let (mut input, _) = char('"')(input)?;
    let mut string = String::new();
    while !input.fragment().starts_with('"') && !input.fragment().is_empty() {
        let (rest, s) = alt((
            take_till1(|c| c == '\\' || c == '"').map(|s: Span<'_>| *s.fragment()),
            preceded(
                char('\\'),
                alt((
                    tag("\\").map(|s: Span<'_>| *s.fragment()),
                    tag("\"").map(|s: Span<'_>| *s.fragment()),
                    value("\n", tag("n")),
                    value("\t", tag("t")),
                )),
            ),
        ))(input)?;
        string.push_str(s);
        input = rest;
    }
    let (input, _) = char('"')(input)?;
    Ok((input, string))
}

pub(crate) fn comment(input: Span<'_>) -> IResult<'_, &str> {
    preceded(
        take_while1(|c| c == ';'),
        map(not_line_ending, |s: Span<'_>| s.fragment().trim()),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest]
    #[case(r#""hello world""#, "hello world")]
    #[case(r#""hello \"world\"""#, "hello \"world\"")]
    #[case(r#""hello \"\"""#, "hello \"\"")]
    #[case(r#""hello\nworld""#, "hello\nworld")]
    #[case(r#""hello\tworld""#, "hello\tworld")]
    #[case(r#""hello\\world""#, "hello\\world")]
    #[case(r#""""#, "")]
    fn parse_string(#[case] input: &str, #[case] expected: &str) {
        let (rest, actual) = string(Span::new(input)).expect("should successfully parse input");
        assert_eq!(&actual, expected);
        assert!(rest.is_empty());
    }

    #[rstest]
    fn simple_comment(#[values("; This is a comment", ";;; This is a comment")] input: &str) {
        let (_, comment) = comment(Span::new(input)).expect("should successfully parse input");
        assert_eq!(comment, "This is a comment");
    }

    #[test]
    fn comment_ends_at_end_of_line() {
        let input = "; This is a comment \n This is not a comment";
        let (rest, comment) = comment(Span::new(input)).expect("should successfully parse input");
        assert_eq!(comment, "This is a comment");
        assert_eq!(*rest.fragment(), "\n This is not a comment");
    }
}
