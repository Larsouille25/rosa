//! Common utilities and data structures used in the compiler.

use std::{
    collections::HashMap,
    ops::{Add, Range},
};

/// A type used to store the offset in byte. It's an alias of u32 because,
/// there is a lot of them in the AST.
#[derive(Clone, Copy)]
pub struct BytePos(pub u32);

impl Add for BytePos {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        BytePos(self.0 + rhs.0)
    }
}

impl From<BytePos> for usize {
    fn from(val: BytePos) -> Self {
        val.0 as Self
    }
}

impl From<usize> for BytePos {
    fn from(val: usize) -> Self {
        BytePos(val as u32)
    }
}

impl BytePos {
    pub const ZERO: BytePos = BytePos(0);
}

#[derive(Clone)]
pub struct Span {
    /// start index of the span, starting from zero.
    pub lo: BytePos,
    /// end index of the span, starting from zero.
    pub hi: BytePos,
}

#[derive(Clone)]
pub struct MultiSpan {
    pub(crate) primary_spans: Vec<Span>,
    // TODO: Implement this part of the MultiSpan type, for it's not one of the
    // most important thing to worry about.
    // pub(crate) span_labels: Vec<(DiagSpan, DiagMessage)>,
}

impl MultiSpan {
    pub fn from_spans(primary_spans: Vec<Span>) -> MultiSpan {
        MultiSpan { primary_spans }
    }

    pub fn primary(&self) -> &Span {
        self.primary_spans
            .first()
            .expect("There is no span in the primary spans vector.")
    }

    pub fn primaries(&self) -> &Vec<Span> {
        &self.primary_spans
    }
}

#[derive(Debug)]
pub struct LineCol {
    /// Line number, starting from one.
    pub line: u32,
    /// Column number, starting from one.
    pub col: u32,
}

pub type FullLinePos = Range<LineCol>;

#[derive(Debug, Default)]
pub struct LinesData {
    /// key = line number
    /// value = a list of spans inside that line.
    p: HashMap<u32, Vec<Range<u32>>>,
}

impl LinesData {
    pub fn new() -> LinesData {
        LinesData { p: HashMap::new() }
    }

    pub fn insert_inline(&mut self, line: u32, span: Range<u32>) -> Option<()> {
        let spans = self.p.get_mut(&line)?;
        spans.push(span);

        Some(())
    }

    pub fn push(&mut self, line: u32, poses: Vec<Range<u32>>) -> Option<()> {
        self.p.insert(line, poses)?;
        Some(())
    }

    pub fn lines(&self) -> Vec<u32> {
        let mut v: Vec<u32> = self.p.keys().copied().collect();
        v.sort();
        v
    }

    pub fn contains_line(&self, line: u32) -> bool {
        self.p.contains_key(&line)
    }

    pub fn get(&self, line: u32) -> Vec<Range<u32>> {
        let mut v: Vec<_> = self
            .p
            .get(&line)
            .expect("This line does not exist.")
            .clone();
        v.sort_by(|a, b| a.clone().cmp(b.clone()));
        v
    }

    pub fn push_or_append(&mut self, line: u32, r: Range<u32>) -> Option<()> {
        if self.contains_line(line) {
            self.insert_inline(line, r)?;
        } else {
            self.push(line, vec![r]);
        }
        Some(())
    }
}
