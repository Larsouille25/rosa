//! Crate responsible for the error handling in the Rosa compiler.

use std::io::{self, Write};
use std::path::Path;
use std::{borrow::Cow, fmt};

use rosa_comm::{MultiSpan, Span};

use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

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
            }
            Level::Warning => {
                spec.set_fg(Some(Color::Yellow)).set_intense(true);
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

    fn format(&self, s: &mut StandardStream) -> Result<(), io::Error> {
        s.set_color(&self.color());
        match self {
            Level::Error => write!(s, "error"),
            Level::Warning => write!(s, "warning"),
            Level::Note => write!(s, "note"),
            Level::Help => write!(s, "help"),
        }?;
        s.set_color(&Style::NoStyle.color_spec(self.clone()));
        Ok(())
    }
}

pub enum Style {
    HeaderMsg,
    Level(Level),
    PathLineCol,
    UnderlinePrimary,
    UnderlineSecondary,
    LabelPrimary,
    LabelSecondary,
    NoStyle,
}

impl Style {
    pub fn color_spec(&self, level: Level) -> ColorSpec {
        let mut spec = ColorSpec::new();
        match self {
            Style::HeaderMsg => {
                spec.set_bold(true);
            }
            Style::Level(lvl) => {
                spec = lvl.color();
                spec.set_bold(true);
            }
            Style::PathLineCol => {
                spec.set_bold(true);
            }
            Style::UnderlinePrimary | Style::LabelPrimary => {
                spec = level.color();
                spec.set_bold(true);
            }
            Style::UnderlineSecondary | Style::LabelSecondary => {
                spec.set_bold(true).set_intense(true);
                spec.set_fg(Some(Color::Blue));
            }
            Style::NoStyle => {}
        }
        spec
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
    pub fn format(&self, s: &mut StandardStream) -> Result<(), io::Error> {
        let (line, col) = self.dcx.line_col(self.span.primary().lo);
        write!(s, "{}:{}:{}: ", self.dcx.filepath.display(), line, col,)?;
        self.level.format(s)?;
        write!(s, ": {}", self.msg)?;
        Ok(())
    }
}

pub type DiagMessage = Cow<'static, str>;

pub struct DiagCtxt<'r> {
    filetext: &'r str,
    filepath: &'r Path,
    stream: &'r mut StandardStream,

    diags: Vec<Diag<'r>>,
}

impl<'r> DiagCtxt<'r> {
    pub fn new(filetext: &'r str, filepath: &'r Path, stream: &'r mut StandardStream) -> Self {
        DiagCtxt {
            filetext,
            filepath,
            stream,
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

    fn set_style(&mut self, style: Style, level: Level) -> Result<(), io::Error> {
        self.stream.set_color(&style.color_spec(level))
    }

    pub fn render_diag(&mut self, diag: &Diag) -> Result<(), io::Error> {
        diag.format(self.stream)
    }
}
