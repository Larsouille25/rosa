//! Lexing of Rosa source code into Tokens.

use std::str::{CharIndices, FromStr};
use std::{iter::Peekable, path::Path};

use crate::tokens::{Token, TokenType};

use crate::tokens::TokenType::*;
use crate::tokens::{Keyword, Punctuation};
use rosa_comm::{BytePos, Span};
use rosa_errors::Diag;
use rosa_errors::{DiagCtxt, Fuzzy};

pub mod abs;
pub mod literals;
pub mod tokens;

pub struct LexrFile<'r> {
    filepath: &'r Path,
    filetext: &'r str,
    /// Index of the last `pop`ed char, starting from 1.
    idx: BytePos,

    iter: Peekable<CharIndices<'r>>,
}

impl<'r> LexrFile<'r> {
    pub fn new(filepath: &'r Path, filetext: &'r str) -> LexrFile<'r> {
        LexrFile {
            filepath,
            filetext,
            idx: 0.into(),
            iter: filetext.char_indices().peekable(),
        }
    }

    pub fn pop(&mut self) -> Option<char> {
        let (i, ch) = self.iter.next()?;
        self.idx = i.into();
        Some(ch)
    }

    pub fn peek(&mut self) -> Option<char> {
        Some(self.iter.peek()?.1)
    }

    pub fn filepath(&self) -> &'r Path {
        self.filepath
    }

    pub fn filetext(&self) -> &'r str {
        self.filetext
    }

    /// NOTE: This function can slow the lexing, it shouldn't be called too
    /// often.
    pub fn reset(&mut self) {
        self.iter = self.filetext.char_indices().peekable();
        self.idx = BytePos::ZERO;
    }

    /// Returns the true length, the count of how many Unicode characters there is
    /// in the source code file.
    pub fn length(&self) -> usize {
        // NOTE: This function is slow because it creates a new iterator each
        // time it's called, if it's called to much time, we should consider
        // storing the lenght of the file in a field and compute it only once.
        self.filetext.chars().count()
    }

    /// Resets the iterator and put the iterator to the new index. The index
    /// starts from 1.
    ///
    /// NOTE: This function can slow the lexing, it shouldn't be called too
    /// often.
    pub fn reset_to(&mut self, new_idx: usize) -> Option<()> {
        if new_idx > self.length() {
            return None;
        }
        self.reset();
        // TODO: use `advance_by` method on the iterator when it will be
        // stabilized
        for _ in 0..new_idx {
            if let Some((i, _)) = self.iter.next() {
                self.idx = i.into();
            } else {
                unreachable!("Should've been caught before.")
            }
        }

        Some(())
    }

    pub fn relative_reset(&mut self, offset: isize) -> Option<()> {
        let new_idx = ((<BytePos as Into<usize>>::into(self.idx) as isize + offset).try_into()
            as Result<usize, _>)
            .ok()?;
        self.reset_to(new_idx)
    }
}

pub struct Lexer<'r> {
    file: LexrFile<'r>,
    prev_idx: BytePos,
    idx: BytePos,
    dcx: &'r DiagCtxt<'r>,
}

impl<'r> Lexer<'r> {
    pub fn new(filepath: &'r Path, filetext: &'r str, dcx: &'r DiagCtxt<'r>) -> Lexer<'r> {
        Lexer {
            file: LexrFile::new(filepath, filetext),
            prev_idx: 0.into(),
            idx: 0.into(),
            dcx,
        }
    }

    /// Advance the iterator and the index (self.idx)
    pub fn pop(&mut self) -> Option<char> {
        self.idx += 1;
        self.file.pop()
    }

    pub fn expect(&mut self, expected: char) {
        let popped = self.pop().unwrap();
        assert_eq!(popped, expected, "Expected to be the same")
    }

    #[inline]
    pub fn peek(&mut self) -> Option<char> {
        self.file.peek()
    }

    /// Current location
    pub fn current_span(&self) -> Span {
        Span {
            lo: self.prev_idx,
            hi: self.idx,
        }
    }

    /// Current location but used when we know we are at the end of file.
    pub fn current_span_end(&self) -> Span {
        Span {
            lo: self.prev_idx,
            hi: self.idx - 1.into(),
        }
    }

    pub fn window(&self, size: usize) -> Option<&str> {
        self.file
            .filetext
            .get(self.prev_idx.0 as usize..self.prev_idx.0 as usize + size)
    }

    pub fn lex(&mut self) -> Fuzzy<Token, Diag> {
        self.skip_useless_whitespace();
        self.skip_comments();

        self.prev_idx = self.idx;

        let tt = 'm: {
            match self.pop() {
                Some(c @ ('A'..='Z' | 'a'..='z' | '_' | '0'..='9')) => {
                    return self.lex_word(c);
                }
                Some('\n') => NewLine,
                Some('"') => return self.lex_str(),
                Some('\'') => return self.lex_char(),
                Some(c) => {
                    if let Some(punct) = self.could_make_punct(c) {
                        // pop the lenght of the punctuation.
                        for _ in 0..punct.size() - 1 {
                            self.pop();
                        }
                        break 'm TokenType::Punct(punct);
                    }
                    let err = self
                        .dcx
                        .struct_err(format!("unknown start of token {c:?}"), self.current_span());
                    return Fuzzy::Err(err);
                }
                None => {
                    let len = self.file.length();
                    return Fuzzy::Ok(Token {
                        tt: EOF,
                        loc: Span::new(len - 1, len),
                    });
                }
            }
        };

        Fuzzy::Ok(Token {
            tt,
            loc: self.current_span(),
        })
    }

    pub fn make_word(&mut self, c: char) -> (String, bool) {
        let mut word = String::from(c);
        let mut numeric = c.is_numeric();

        while let Some(c) = self.peek() {
            match c {
                'A'..='Z' | 'a'..='z' => {
                    word.push(c);
                    numeric = false;
                }
                '0'..='9' | '_' => {
                    word.push(c);
                }
                _ => break,
            }
            self.pop();
        }

        (word, numeric)
    }

    pub fn lex_word(&mut self, c: char) -> Fuzzy<Token, Diag> {
        let (word, numeric) = self.make_word(c);

        let tt = if numeric {
            return self.lex_int(word);
        } else {
            self.lex_keyword(word)
        };

        Fuzzy::Ok(Token {
            tt,
            loc: self.current_span(),
        })
    }

    pub fn lex_keyword(&self, word: String) -> TokenType {
        if let Ok(kw) = Keyword::from_str(&word) {
            TokenType::KW(kw)
        } else {
            TokenType::Ident(word)
        }
    }

    pub fn skip_useless_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            match c {
                ' ' => {
                    self.pop();
                }
                '\u{000B}'..='\u{000D}'
                | '\u{0085}'
                | '\u{00A0}'
                | '\u{1680}'
                | '\u{2000}'..='\u{200A}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202F}'
                | '\u{205F}'
                | '\u{3000}' => {
                    self.pop();
                }
                _ => break,
            }
        }
    }

    pub fn could_make_punct(&mut self, c: char) -> Option<Punctuation> {
        use Punctuation::*;
        Some(match c {
            // single char punctuation
            '(' => RParen,
            ')' => LParen,
            '[' => RBracket,
            ']' => LBracket,
            '{' => RBrace,
            '}' => LBrace,
            ':' => Colon,
            ';' => Semi,
            ',' => Comma,
            '@' => At,
            '*' => Asterisk,
            '^' => Caret,
            '.' => Dot,
            '-' => Minus,
            '%' => Percent,
            '+' => Plus,
            '/' => Slash,

            // ambigious
            '!' => match self.peek() {
                Some('=') => ExclamationmarkEqual,
                _ => Exclamationmark,
            },
            '=' => match self.peek() {
                Some('=') => Equal2,
                _ => Equal,
            },
            '<' => match self.peek() {
                Some('<') => LArrow2,
                Some('=') => LArrowEqual,
                _ => LArrow,
            },
            '>' => match self.peek() {
                Some('>') => RArrow2,
                Some('=') => RArrowEqual,
                _ => RArrow,
            },

            _ => return None,
        })
    }

    pub fn skip_comments(&mut self) {
        if let Some('#') = self.peek() {
            while self.pop() != Some('\n') {}

            // we are on a new line, skip use less whitespaces
            self.skip_useless_whitespace();

            // maybe skip a comment
            self.skip_comments();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const UNIT_TEST_PATH: &str = "<unit test>";
    const TEXT1: &str = "Rosa 🌹";

    macro_rules! unit_test_path {
        ($path_str:ident) => {
            Path::new($path_str.into())
        };
        () => {
            unit_test_path!(UNIT_TEST_PATH)
        };
    }

    #[test]
    fn lexr_file_peek_pop() {
        let mut lfile = LexrFile::new(unit_test_path!(), TEXT1);
        assert_eq!(lfile.peek(), Some('R'));
        assert_eq!(lfile.pop(), Some('R'));

        assert_eq!(lfile.peek(), Some('o'));
        assert_eq!(lfile.pop(), Some('o'));

        assert_eq!(lfile.peek(), Some('s'));
        assert_eq!(lfile.pop(), Some('s'));

        assert_eq!(lfile.peek(), Some('a'));
        assert_eq!(lfile.pop(), Some('a'));

        assert_eq!(lfile.peek(), Some(' '));
        assert_eq!(lfile.pop(), Some(' '));

        assert_eq!(lfile.peek(), Some('🌹'));
        assert_eq!(lfile.pop(), Some('🌹'));

        assert_eq!(lfile.pop(), None);
        assert_eq!(lfile.pop(), None);
    }

    #[test]
    fn lexr_file_reset() {
        let mut lfile = LexrFile::new(unit_test_path!(), TEXT1);
        assert_eq!(lfile.pop(), Some('R'));
        assert_eq!(lfile.pop(), Some('o'));
        assert_eq!(lfile.pop(), Some('s'));
        assert_eq!(lfile.pop(), Some('a'));
        lfile.reset_to(3);
        assert_eq!(lfile.peek(), Some('a'));
        assert_eq!(lfile.pop(), Some('a'));

        lfile.reset_to(6);
        assert_eq!(lfile.pop(), None);
        assert_eq!(lfile.pop(), None);
    }

    #[test]
    fn lexer_peek_pop() {
        let dcx = DiagCtxt::new(TEXT1, unit_test_path!());
        let mut lexer = Lexer::new(unit_test_path!(), TEXT1, &dcx);

        assert_eq!(lexer.pop(), Some('R'));

        assert_eq!(lexer.peek(), Some('o'));
        assert_eq!(lexer.peek(), Some('o'));
        assert_eq!(lexer.pop(), Some('o'));

        assert_eq!(lexer.pop(), Some('s'));

        assert_eq!(lexer.pop(), Some('a'));

        assert_eq!(lexer.current_span(), Span::new(0, 4));

        assert_eq!(lexer.pop(), Some(' '));
        assert_eq!(lexer.pop(), Some('🌹'));
        assert_eq!(lexer.pop(), None);
    }

    #[test]
    fn lexer_identifier_and_keywords() {
        let text = "abc fun return val var type true false";
        let dcx = DiagCtxt::new(text, unit_test_path!());
        let mut lexer = Lexer::new(unit_test_path!(), text, &dcx);
        assert_eq!(lexer.lex().unwrap().tt, TokenType::Ident("abc".to_string()));
        assert_eq!(lexer.lex().unwrap().tt, TokenType::KW(Keyword::Fun));
        assert_eq!(lexer.lex().unwrap().tt, TokenType::KW(Keyword::Return));
        assert_eq!(lexer.lex().unwrap().tt, TokenType::KW(Keyword::Val));
        assert_eq!(lexer.lex().unwrap().tt, TokenType::KW(Keyword::Var));
        assert_eq!(lexer.lex().unwrap().tt, TokenType::KW(Keyword::Type));
        assert_eq!(lexer.lex().unwrap().tt, TokenType::KW(Keyword::True));
        assert_eq!(lexer.lex().unwrap().tt, TokenType::KW(Keyword::False));
        assert_eq!(lexer.lex().unwrap().tt, TokenType::EOF);
    }

    #[test]
    #[should_panic]
    fn lexer_too_large_int() {
        let text = u128::MAX.to_string();
        let dcx = DiagCtxt::new(&text, unit_test_path!());
        let mut lexer = Lexer::new(unit_test_path!(), &text, &dcx);
        // Should panic because we unwrap an Fuzzy::Err due to the
        // source code containing a number too large to fit in the int literal
        lexer.lex().unwrap();
    }
}
