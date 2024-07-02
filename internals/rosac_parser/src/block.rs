use rosa_comm::Span;
use rosa_errors::{Diag, Fuzzy};
use rosac_lexer::tokens::TokenType::*;
use rosac_lexer::{abs::AbsLexer, tokens::Token};

use crate::{derive_loc, expected_tok_msg, parse, AstNode, Location, Parser};

#[derive(Debug, Clone)]
pub struct Block<N: AstNode> {
    pub content: Vec<N>,
    pub loc: Span,
}

derive_loc!(Block<N> where <N: AstNode>);

impl<N: AstNode<Output = N> + Location> AstNode for Block<N> {
    type Output = Self;

    fn parse<L: AbsLexer>(parser: &mut Parser<'_, L>) -> Fuzzy<Self::Output, Diag> {
        let mut content = Vec::new();

        match parser.try_peek_tok() {
            Some(Token { tt: NewLine, .. }) => {}
            _ => {
                let elem = parse!(parser => N);
                return Fuzzy::Ok(Block {
                    loc: elem.loc(),
                    content: vec![elem],
                });
            }
        }

        let Some((gap, til_next)) = parser.compute_indent() else {
            let loc = parser
                .try_peek_tok()
                .map(|t| t.loc.clone())
                .unwrap_or_default();

            return Fuzzy::Err(
                parser
                    .dcx()
                    .struct_err(expected_tok_msg("block", [EOF]), loc),
            );
        };

        if let Some(lvl) = parser.last_indent() {
            if lvl == gap {
                let loc = parser
                    .try_peek_tok()
                    .map(|t| t.loc.clone())
                    .unwrap_or_default();

                return Fuzzy::Err(parser.dcx().struct_err("a block may not be empty", loc));
            }
        }

        for _ in 0..til_next {
            parser.consume_tok();
        }
        parser.indent(gap);

        loop {
            content.push(parse!(parser => N));

            // we compute the indent level here and how many new lines we need
            // to consume
            let Some((gap, til_next)) = parser.compute_indent() else {
                let loc = parser
                    .try_peek_tok()
                    .map(|t| t.loc.clone())
                    .unwrap_or_default();

                return Fuzzy::Err(
                    parser
                        .dcx()
                        .struct_err(expected_tok_msg("block", [EOF]), loc),
                );
            };

            // if the indent level don't match we break.
            if gap != parser.last_indent().unwrap() {
                break;
            }

            // here we consume the new lines tokens
            for _ in 0..til_next {
                parser.consume_tok();
            }
        }

        parser.dedent();
        // Here, we unwrap because we know for sure we have at one thing
        let loc = Span::from_ends(
            content.first().unwrap().loc(),
            content.last().unwrap().loc(),
        );

        Fuzzy::Ok(Block { content, loc })
    }
}
