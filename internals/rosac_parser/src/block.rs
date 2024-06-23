use rosa_comm::Span;
use rosa_errors::{Diag, Fuzzy};
use rosac_lexer::abs::AbsLexer;
use rosac_lexer::tokens::TokenType::*;

use crate::{derive_loc, expect_token, parse, AstNode, FmtToken, Location, Parser};

#[derive(Debug, Clone)]
pub struct Block<N: AstNode> {
    pub nodes: Vec<N>,
    pub loc: Span,
}

derive_loc!(Block<N> where <N: AstNode>);

impl<N: AstNode<Output = N> + Location> AstNode for Block<N> {
    type Output = Self;

    fn parse<L: AbsLexer>(parser: &mut Parser<'_, L>) -> Fuzzy<Self::Output, Diag> {
        let (_, lf) = expect_token!(parser => [NewLine, ()], [FmtToken::NewLine]);
        let mut loc = lf.clone();
        parser.enter_scope(&lf);
        let first_scope = parser.scopelvl;
        let mut nodes = Vec::new();

        loop {
            let node = parse!(parser => N);
            loc.hi = node.loc().hi;
            nodes.push(node);

            let (_, lf) =
                expect_token!(parser => [NewLine, (); EOF, (), in break], [FmtToken::NewLine]);
            let Some(scope) = parser.scope(&lf) else {
                break;
            };
            if scope < first_scope {
                break;
            }
        }

        Fuzzy::Ok(Block { nodes, loc })
    }
}
