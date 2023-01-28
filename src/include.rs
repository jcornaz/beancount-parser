use nom::{
    bytes::complete::tag,
    character::complete::space1,
    combinator::map,
    sequence::{preceded, tuple},
    IResult,
};

use crate::string::string;

/// Include directive
#[derive(Clone, Debug)]
pub struct Include {
    path: String,
}

impl Include {
    /// Path to include
    #[must_use]
    pub fn path(&self) -> &str {
        self.path.as_ref()
    }
}

pub(crate) fn include(input: &str) -> IResult<&str, Include> {
    map(preceded(tuple((tag("include"), space1)), string), |path| {
        Include { path }
    })(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    use nom::combinator::all_consuming;

    #[test]
    fn valid_include_directive() {
        let (_, inc) = include(r#"include "abc.beancount""#).unwrap();
        assert_eq!(inc.path(), "abc.beancount");
    }

    #[rstest]
    fn invalid(#[values("include", r#"include "a" "b""#)] input: &str) {
        assert!(matches!(
            all_consuming(include)(input),
            Err(nom::Err::Error(_))
        ));
    }
}
