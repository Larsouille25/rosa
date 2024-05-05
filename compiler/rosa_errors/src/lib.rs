//! Crate responsible for the error handling in the Rosa compiler.

use std::borrow::Cow;
use std::io::{self, Write};
use std::path::Path;

use rosa_comm::{MultiSpan, Span};

use style::SetStyle;
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

use crate::style::Style;
pub mod style;

#[derive(Clone)]
pub enum Level {
    Error,
    Warning,
    Note,
    Help,
}

impl Level {
    pub fn color(&self) -> ColorSpec {
        let mut spec = ColorSpec::new();
        match self {
            Level::Error => {
                spec.set_fg(Some(Color::Red)).set_intense(true);
                spec.set_bold(true);
            }
            Level::Warning => {
                spec.set_fg(Some(Color::Yellow)).set_intense(true);
                spec.set_bold(true);
            }
            Level::Note => {
                spec.set_fg(Some(Color::Green)).set_intense(true);
            }
            Level::Help => {
                spec.set_fg(Some(Color::Cyan)).set_intense(true);
            }
        }
        spec
    }

    pub fn format(&self, s: &mut StandardStream) -> io::Result<()> {
        s.set_color(&self.color())?;
        match self {
            Level::Error => write!(s, "error"),
            Level::Warning => write!(s, "warning"),
            Level::Note => write!(s, "note"),
            Level::Help => write!(s, "help"),
        }?;
        s.set_no_style()?;
        Ok(())
    }
}

/// `Diag` for `Diagnostic`
#[derive(Clone)]
pub struct Diag<'r> {
    dcx: &'r DiagCtxt<'r>,

    level: Level,
    msg: DiagMessage,
    span: MultiSpan,
}

impl<'r> Diag<'r> {
    pub fn format(&self, s: &mut StandardStream) -> io::Result<()> {
        let (line, col) = self.dcx.line_col(self.span.primary().lo);

        s.set_style(Style::PathLineCol, self.level.clone())?;
        write!(s, "{}:{}:{}: ", self.dcx.filepath.display(), line, col,)?;
        s.set_no_style()?;

        self.level.format(s)?;
        write!(s, ": ")?;
        s.set_style(Style::HeaderMsg, self.level.clone())?;
        write!(s, "{}", self.msg)?;
        s.set_no_style()?;

        // TODO: Implement the rendering of the code pointed by the spans.
        write!(s, "\n")?;
        Ok(())
    }
}

pub type DiagMessage = Cow<'static, str>;

pub struct DiagCtxt<'r> {
    filetext: &'r str,
    filepath: &'r Path,

    diags: Vec<Diag<'r>>,
}

impl<'r> DiagCtxt<'r> {
    pub fn new(filetext: &'r str, filepath: &'r Path) -> Self {
        DiagCtxt {
            filetext,
            filepath,
            diags: vec![],
        }
    }

    pub fn diag(&self, level: Level, msg: impl Into<DiagMessage>, primary_span: Span) -> Diag {
        Diag {
            dcx: self,
            level,
            msg: msg.into(),
            span: MultiSpan::from_span(primary_span),
        }
    }

    pub fn diag_err(&self, msg: impl Into<DiagMessage>, primary_span: Span) -> Diag {
        self.diag(Level::Error, msg, primary_span)
    }

    pub fn diag_warn(&self, msg: impl Into<DiagMessage>, primary_span: Span) -> Diag {
        self.diag(Level::Warning, msg, primary_span)
    }

    pub fn line_col(&self, idx: usize) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;

        for (i, ch) in self.filetext.char_indices() {
            if idx == i {
                break;
            }
            match ch {
                '\n' => {
                    col = 1;
                    line += 1;
                }
                _ => col += 1,
            }
        }

        (line, col)
    }

    pub fn push_diag(&mut self, diag: Diag<'r>) {
        self.diags.push(diag);
    }
}
