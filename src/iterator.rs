use std::collections::HashSet;

use nom::{combinator::ParserIterator, Finish};

use crate::{DirectiveContent, Entry, Error, RawEntry, Span, Tag};

pub(crate) struct Iter<I, E, F> {
    inner: Option<ParserIterator<I, E, F>>,
    tag_stack: HashSet<Tag>,
}

impl<I, E, F> Iter<I, E, F> {
    pub(crate) fn new(inner: ParserIterator<I, E, F>) -> Self {
        Self {
            inner: Some(inner),
            tag_stack: HashSet::new(),
        }
    }
}

impl<'i, D, F> Iterator for Iter<Span<'i>, nom::error::Error<Span<'i>>, F>
where
    ParserIterator<Span<'i>, nom::error::Error<Span<'i>>, F>: Iterator<Item = RawEntry<D>>,
{
    type Item = Result<Entry<D>, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        let Some(inner) = &mut self.inner else { return None };
        for entry in inner {
            match entry {
                RawEntry::Directive(mut d) => {
                    if let DirectiveContent::Transaction(trx) = &mut d.content {
                        trx.tags.extend(self.tag_stack.iter().cloned());
                    }
                    return Some(Ok(Entry::Directive(d)));
                }
                RawEntry::Option(o) => {
                    return Some(Ok(Entry::Option(o)));
                }
                RawEntry::Include(path) => {
                    return Some(Ok(Entry::Include(path)));
                }
                RawEntry::PushTag(tag) => {
                    self.tag_stack.insert(tag);
                }
                RawEntry::PopTag(tag) => {
                    self.tag_stack.remove(&tag);
                }
                RawEntry::Comment => (),
            }
        }
        match self.inner.take().unwrap().finish().finish() {
            Ok((rest, _)) if rest.fragment().is_empty() => None,
            Ok((input, _)) | Err(nom::error::Error { input, .. }) => Some(Err(Error::new(input))),
        }
    }
}
