//! Module responsible for the lexing of the literals in the source file,
//! like integer, float, string and char literals

use rosa_comm::Span;
use rosa_errors::{Diag, RosaRes};

use crate::tokens::{Token, TokenType};

impl<'r> super::Lexer<'r> {
    pub(crate) fn lex_int(&mut self, num: String) -> RosaRes<Token, Diag> {
        match parse_u64(&num, 10) {
            Ok(lit) => RosaRes::Good(Token {
                tt: TokenType::Int(lit),
                loc: self.current_span(),
            }),
            Err(ParseUIntError::IntegerOverflow) => RosaRes::Unrecovered(
                self.dcx
                    .struct_err("integer literal is too large", self.current_span()),
            ),
            Err(ParseUIntError::DigitOutOfRange(loc)) => RosaRes::Unrecovered(self.dcx.struct_err(
                format!(
                    "digit out of radix {:?}",
                    &num[loc.clone().range_usize()].chars().next().unwrap()
                ),
                loc.offset(self.prev_idx),
            )),
            Err(ParseUIntError::InvalidCharacter(loc)) => {
                RosaRes::Unrecovered(self.dcx.struct_err(
                    format!(
                        "invalid character in literal, {:?}",
                        &num[loc.clone().range_usize()].chars().next().unwrap()
                    ),
                    loc.offset(self.prev_idx),
                ))
            }
            Err(ParseUIntError::InvalidRadix) => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum ParseUIntError {
    InvalidRadix,
    InvalidCharacter(Span),
    DigitOutOfRange(Span),
    IntegerOverflow,
}

pub(crate) fn parse_u64(input: &str, radix: u8) -> Result<u64, ParseUIntError> {
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
            _ => return Err(ParseUIntError::InvalidCharacter(Span::new(i, i + 1))),
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
