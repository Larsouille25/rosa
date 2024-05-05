//! Common utilities and data structures used in the compiler.

#[derive(Clone)]
pub struct Span {
    /// start index of the span, starting from zero.
    pub lo: usize,
    /// end index of the span, starting from zero.
    pub hi: usize,
}

#[derive(Clone)]
pub struct MultiSpan {
    pub(crate) primary_spans: Vec<Span>,
    // TODO: Implement this part of the MultiSpan type, for it's not one of the
    // most important thing to worry about.
    // pub(crate) span_labels: Vec<(DiagSpan, DiagMessage)>,
}

impl MultiSpan {
    pub fn from_span(primary_span: Span) -> MultiSpan {
        MultiSpan {
            primary_spans: vec![primary_span],
        }
    }

    pub fn primary(&self) -> &Span {
        self.primary_spans
            .get(0)
            .expect("There is no span in the primary spans vector.")
    }
}
