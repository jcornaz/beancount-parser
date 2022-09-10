use nom::{
    branch::alt,
    bytes::complete::{escaped_transform, tag, take_till1, take_while1},
    character::complete::{char, digit1, line_ending, not_line_ending},
    combinator::{map, not, opt, recognize, value},
    sequence::{delimited, preceded, tuple},
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

pub(crate) fn comment_line(input: &str) -> IResult<&str, &str> {
    let date_like = tuple((digit1, char('-'), digit1, char('-'), digit1));
    recognize(tuple((not(date_like), not_line_ending, opt(line_ending))))(input)
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

    #[rstest]
    #[case("* Banking", "")]
    #[case("* Banking\n2022-01-01", "2022-01-01")]
    #[case("\n", "")]
    #[case("\ntest", "test")]
    #[case("test", "")]
    fn recognize_comment_line(#[case] input: &str, #[case] expected_rest: &str) {
        let (rest, _) =
            comment_line(input).expect("should succesfully parse the input as a comment line");
        assert_eq!(rest, expected_rest);
    }

    #[rstest]
    fn recognize_a_non_comment_line(#[values("2022-01-01", "0000-00-00")] input: &str) {
        let result = comment_line(input);
        assert!(result.is_err(), "{:?}", result);
    }
}
