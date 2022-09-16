use nom::{
    branch::alt,
    bytes::complete::{escaped_transform, tag, take_till1, take_while1},
    character::complete::{char, not_line_ending},
    combinator::{map, value},
    sequence::{delimited, preceded},
    IResult,
};

pub(crate) fn string(input: &str) -> IResult<&str, String> {
    delimited(
        char('"'),
        alt((
            escaped_transform(
                take_till1(|c| c == '\\' || c == '"'),
                '\\',
                alt((
                    tag("\\"),
                    tag("\""),
                    value("\n", tag("n")),
                    value("\t", tag("t")),
                )),
            ),
            map(tag(""), String::from),
        )),
        char('"'),
    )(input)
}

pub(crate) fn comment(input: &str) -> IResult<&str, &str> {
    preceded(take_while1(|c| c == ';'), map(not_line_ending, str::trim))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(r#""hello world""#, "hello world")]
    #[case(r#""hello \"world\"""#, "hello \"world\"")]
    #[case(r#""hello \"\"""#, "hello \"\"")]
    #[case(r#""hello\nworld""#, "hello\nworld")]
    #[case(r#""hello\tworld""#, "hello\tworld")]
    #[case(r#""hello\\world""#, "hello\\world")]
    #[case(r#""""#, "")]
    fn parse_string(#[case] input: &str, #[case] expected: &str) {
        let (rest, actual) = string(input).expect("should succesfully parse input");
        assert_eq!(&actual, expected);
        assert!(rest.is_empty());
    }

    #[rstest]
    fn simple_comment(#[values("; This is a comment", ";;; This is a comment")] input: &str) {
        let (_, comment) = comment(input).expect("should successfully parse input");
        assert_eq!(comment, "This is a comment");
    }

    #[test]
    fn comment_ends_at_end_of_line() {
        let input = "; This is a comment \n This is not a comment";
        let (rest, comment) = comment(input).expect("should successfully parse input");
        assert_eq!(comment, "This is a comment");
        assert_eq!(rest, "\n This is not a comment");
    }
}
