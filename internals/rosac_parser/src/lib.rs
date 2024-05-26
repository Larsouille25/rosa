use std::marker::PhantomData;

use rosa_errors::DiagCtxt;
use rosac_lexer::{
    abs::{AbsLexer, BufferedLexer},
    Lexer,
};

pub struct Parser<'r, L: AbsLexer = BufferedLexer<'r>> {
    lexer: L,
    // used to be able to make the L type default to BufferedLexer.
    _marker: PhantomData<&'r ()>,
}
