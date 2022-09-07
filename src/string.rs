use nom::{
    branch::alt,
    bytes::complete::{escaped_transform, tag, take_till1},
    character::complete::char,
    combinator::{map, value},
    sequence::delimited,
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
}
