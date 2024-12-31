use std::ops::Range;

use serde::{Deserialize, Serialize};

use super::src::SourceId;

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    src: SourceId,
    range: Range<usize>,
}

pub type Spanned<T> = (T, Span);

impl Span {
    #[cfg(test)]
    pub fn empty() -> Self {
        Self {
            src: SourceId::empty(),
            range: 0..0,
        }
    }

    pub fn repl(range: Range<usize>) -> Self {
        use chumsky::span::Span;
        Self::new(SourceId::new("[repl]"), range)
    }

    pub fn src(&self) -> SourceId {
        self.src
    }

    pub fn range(&self) -> Range<usize> {
        use chumsky::span::Span;
        self.start()..self.end()
    }

    pub fn union(self, other: &Span) -> Self {
        use chumsky::span::Span;

        assert_eq!(
            self.src, other.src,
            "attempted to union spans with different sources"
        );
        Self {
            range: self.start().min(other.start())..self.end().max(other.end()),
            ..self
        }
    }
}

impl chumsky::span::Span for Span {
    type Context = SourceId;

    type Offset = usize;

    fn new(context: Self::Context, range: std::ops::Range<Self::Offset>) -> Self {
        Self {
            range,
            src: context,
        }
    }

    fn context(&self) -> Self::Context {
        self.src
    }

    fn start(&self) -> Self::Offset {
        self.range.start()
    }

    fn end(&self) -> Self::Offset {
        self.range.end()
    }
}

impl ariadne::Span for Span {
    type SourceId = SourceId;

    fn source(&self) -> &Self::SourceId {
        &self.src
    }

    fn start(&self) -> usize {
        chumsky::span::Span::start(self)
    }

    fn end(&self) -> usize {
        chumsky::span::Span::end(self)
    }
}
