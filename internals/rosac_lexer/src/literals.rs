//! Module responsible for the lexing of the literals in the source file,
//! like integer, float, string and char literals

use rosa_comm::Span;
use rosa_errors::{Diag, Fuzzy};

use crate::tokens::{Token, TokenType};

impl<'r> super::Lexer<'r> {
    pub fn lex_int(&mut self, num: String) -> Fuzzy<Token, Diag> {
        match self.make_int(&num, 10) {
            Ok(lit) => Fuzzy::Ok(Token {
                tt: TokenType::Int(lit),
                loc: self.current_span(),
            }),
            Err(diag) => Fuzzy::Err(diag),
        }
    }

    pub fn make_int(&mut self, num: &str, radix: u8) -> Result<u64, Diag> {
        match parse_u64(num, radix) {
            Ok(number) => Ok(number),
            Err(ParseUIntError::IntegerOverflow) => Err(self
                .dcx
                .struct_err("integer literal is too large", self.current_span())),
            Err(ParseUIntError::DigitOutOfRange(loc)) => Err(self.dcx.struct_err(
                format!(
                    "digit out of radix {:?}",
                    &num[loc.clone().range_usize()].chars().next().unwrap()
                ),
                loc.offset(self.prev_idx),
            )),
            Err(ParseUIntError::InvalidCharacter(loc)) => Err(self.dcx.struct_err(
                format!(
                    "invalid character in literal, {:?} {loc:?}",
                    &num[loc.clone().range_usize()].chars().next().unwrap()
                ),
                loc.offset(self.idx - 2.into()),
            )),
            Err(ParseUIntError::InvalidRadix) => {
                Err(self.dcx.struct_err("invalid radix", self.current_span()))
            }
        }
    }

    /// Lexes a string literal
    pub fn lex_str(&mut self) -> Fuzzy<Token, Diag> {
        let mut str = String::new();
        let mut diags = Vec::new();

        loop {
            match self.peek() {
                Some('"') => {
                    self.expect('"');
                    break;
                }
                Some('\\') => {
                    self.expect('\\');

                    let es = match self.pop() {
                        Some(es) => es,
                        None => continue,
                    };

                    if es == '"' {
                        str.push(es);
                        continue;
                    }

                    match self.make_escape_sequence(es) {
                        Ok(res) => str.push(res),
                        Err(diag) => diags.push(diag),
                    }
                }
                Some(c) => {
                    str.push(c);
                    self.expect(c);
                }
                _ => {
                    return Fuzzy::Err(
                        self.dcx
                            .struct_err("unterminated string literal", self.current_span_end()),
                    )
                }
            }
        }

        let tok = Token {
            tt: TokenType::Str(str),
            loc: self.current_span(),
        };
        if diags.is_empty() {
            Fuzzy::Ok(tok)
        } else {
            Fuzzy::Fuzzy(tok, diags)
        }
    }

    pub fn make_escape_sequence(&mut self, es: char) -> Result<char, Diag> {
        Ok(match es {
            '0' => '\0',
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            'x' => self.make_hex_es()?,
            'u' => {
                // TODO: implement the lexing of unicode es
                return Err(self.dcx.struct_err(
                    "unicode escape sequence are not yet supported",
                    Span::new(self.idx - 2.into(), self.idx),
                ));
            }
            _ => {
                return Err(self.dcx.struct_err(
                    format!("unknown escape sequence: '\\{es}'"),
                    Span::new(self.idx - 2.into(), self.idx),
                ))
            }
        })
    }

    pub fn make_hex_es(&mut self) -> Result<char, Diag> {
        let mut str = String::with_capacity(2);
        for _ in 0..2 {
            str.push(self.pop().ok_or_else(|| {
                self.dcx
                    .struct_err("unterminated string literal", self.current_span())
            })?);
        }

        Ok(self.make_int(&str, 16)? as u8 as char)
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseUIntError {
    InvalidRadix,
    InvalidCharacter(Span),
    DigitOutOfRange(Span),
    IntegerOverflow,
}

/// Parse a number passed as input into a u64 using the radix.
///
/// # Note
///
/// The radix is 'inclusive' if you want to parse a number as a decimal, then
/// `radix = 10` and if you want to parse a number as hexadecimal `radix = 16`
/// etc etc...
pub fn parse_u64(input: &str, radix: u8) -> Result<u64, ParseUIntError> {
    if !(2..=36).contains(&radix) {
        return Err(ParseUIntError::InvalidRadix);
    }

    let mut result: u64 = 0;

    for (i, c) in input.char_indices().peekable() {
        let digit = match c {
            '0'..='9' => (c as u8 - b'0') as u32,
            'a'..='z' => (c as u8 - b'a' + 10) as u32,
            'A'..='Z' => (c as u8 - b'A' + 10) as u32,
            '_' => continue,
            _ => {
                return Err(ParseUIntError::InvalidCharacter(Span::new(i, i + 1)));
            }
        };

        if digit >= radix.into() {
            return Err(ParseUIntError::DigitOutOfRange(Span::new(i, i + 1)));
        }

        result = match result
            .checked_mul(radix as u64)
            .and_then(|r| r.checked_add(digit as u64))
        {
            Some(val) => val,
            None => return Err(ParseUIntError::IntegerOverflow),
        };
    }

    Ok(result)
}
