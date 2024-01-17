use nom::character::complete::space1;

use crate::{string_escapable, IResult, Span};

/// An event
///
/// # Example
/// ```
/// # use beancount_parser::{BeancountFile, DirectiveContent};
/// let input = r#"2023-05-31 event "Location" "Switzerland""#;
/// let beancount: BeancountFile<f64> = input.parse().unwrap();
/// let DirectiveContent::Event(ref event) = beancount.directives[0].content else { unreachable!() };
/// assert_eq!(event.name, "Location");
/// assert_eq!(event.value, "Switzerland");
/// ```
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Event {
    /// Name of the event
    pub name: String,
    /// Value of the event
    pub value: String,
}

pub(super) fn parse(input: Span<'_>) -> IResult<'_, Event> {
    let (input, name) = string_escapable(input)?;
    let (input, _) = space1(input)?;
    let (input, value) = string_escapable(input)?;
    Ok((input, Event { name, value }))
}
