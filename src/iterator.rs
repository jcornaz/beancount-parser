use std::collections::HashSet;

use nom::{combinator::ParserIterator, Finish};

use crate::{DirectiveContent, Entry, Error, RawEntry, Span, Tag};

type InnerIter<'i, F> = ParserIterator<Span<'i>, nom::error::Error<Span<'i>>, F>;

pub(crate) struct Iter<'i, F> {
    source: &'i str,
    inner: Option<InnerIter<'i, F>>,
    tag_stack: HashSet<Tag>,
}

impl<'i, F> Iter<'i, F> {
    pub(crate) fn new(source: &'i str, value: InnerIter<'i, F>) -> Self {
        Self {
            source,
            inner: Some(value),
            tag_stack: HashSet::new(),
        }
    }
}

impl<'i, D, F> Iterator for Iter<'i, F>
where
    for<'a> &'a mut InnerIter<'i, F>: Iterator<Item = RawEntry<D>>,
{
    type Item = Result<Entry<D>, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        let inner = self.inner.as_mut()?;
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
            Ok((rest, ())) if rest.fragment().is_empty() => None,
            Ok((input, ())) | Err(nom::error::Error { input, .. }) => {
                Some(Err(Error::new(self.source, input)))
            }
        }
    }
}
