//! Source code span handling for error reporting

use std::fmt;
use std::ops::Range;

/// Represents a location in the source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    /// Start byte offset (inclusive)
    pub start: usize,
    /// End byte offset (exclusive)
    pub end: usize,
}

impl Span {
    /// Create a new span from start and end byte offsets
    pub fn new(start: usize, end: usize) -> Self {
        assert!(start <= end, "Span start must be <= end");
        Span { start, end }
    }
    
    /// Create a span that covers both this span and another span
    pub fn to(&self, other: Span) -> Self {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
    
    /// Check if a position is within this span
    pub fn contains(&self, pos: usize) -> bool {
        self.start <= pos && pos < self.end
    }
}

impl From<Range<usize>> for Span {
    fn from(range: Range<usize>) -> Self {
        Span::new(range.start, range.end)
    }
}

impl From<&Range<usize>> for Span {
    fn from(range: &Range<usize>) -> Self {
        Span::new(range.start, range.end)
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

/// A value with an associated source code span
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Spanned<T> {
    /// The value
    pub value: T,
    /// The source code span
    pub span: Span,
}

impl<T> Spanned<T> {
    /// Create a new spanned value
    pub fn new(value: T, span: Span) -> Self {
        Spanned { value, span }
    }
    
    /// Map the inner value while preserving the span
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Spanned<U> {
        Spanned {
            value: f(self.value),
            span: self.span,
        }
    }
}

impl<T: Default> Default for Spanned<T> {
    fn default() -> Self {
        Spanned {
            value: T::default(),
            span: Span::new(0, 0),
        }
    }
}

impl<T: fmt::Display> fmt::Display for Spanned<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}
