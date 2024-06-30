//! Module responsible for parsing declarations like function, types, imports..

use rosa_comm::Span;
use rosa_errors::{Diag, Fuzzy};
use rosac_lexer::tokens::{Punctuation, Token, TokenType::*};
use rosac_lexer::{abs::AbsLexer, tokens::Keyword};

use crate::types::Type;
use crate::{block::Block, derive_loc, expect_token, stmt::Statement, AstNode, Parser};
use crate::{expected_tok_msg, parse, AstPart, FmtToken};

#[derive(Debug, Clone)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Debug, Clone)]
pub struct Declaration {
    pub vis: Visibility,
    pub decl: DeclarationInner,
    pub loc: Span,
}

derive_loc!(Declaration);

impl AstNode for Declaration {
    type Output = Self;

    fn parse<L: AbsLexer>(parser: &mut Parser<'_, L>) -> Fuzzy<Self::Output, Diag> {
        let (vis, vis_loc) = expect_token!(
            parser => [KW(Keyword::Pub), Visibility::Public]
            else { (Visibility::Private, Span::ZERO) }
        );

        let (decl, decl_loc) = match parser.peek_tok() {
            Token {
                tt: KW(Keyword::Fun),
                ..
            } => parse!(@fn parser => parse_fun_decl),
            t => {
                let t = t.clone();
                return Fuzzy::Err(
                    parser
                        .dcx()
                        .struct_err(expected_tok_msg(t.tt, [AstPart::Declaration]), t.loc),
                );
            }
        };

        let loc = if vis_loc == Span::ZERO {
            decl_loc
        } else {
            Span::new(vis_loc.lo, decl_loc.hi)
        };
        Fuzzy::Ok(Declaration { vis, decl, loc })
    }
}

#[derive(Debug, Clone)]
pub enum DeclarationInner {
    Function {
        name: String,
        args: Vec<(String, Type)>,
        ret: Option<Type>,
        block: Block<Statement>,
    },
}

pub fn parse_fun_decl(
    parser: &mut Parser<'_, impl AbsLexer>,
) -> Fuzzy<(DeclarationInner, Span), Diag> {
    let mut loc = Span::default();
    let (_, Span { lo, .. }) =
        expect_token!(parser => [KW(Keyword::Fun), ()], [FmtToken::KW(Keyword::Fun)]);
    loc.lo = lo;

    let (name, _) = expect_token!(parser => [Ident(name), name.clone()], [FmtToken::Identifier]);

    expect_token!(parser => [Punct(Punctuation::LParen), ()], [FmtToken::Punct(Punctuation::LParen)]);

    let mut args = Vec::new();
    loop {
        if let Some(Token {
            tt: Punct(Punctuation::RParen),
            ..
        }) = parser.try_peek_tok()
        {
            break;
        }

        let (name, _) =
            expect_token!(parser => [Ident(name), name.clone()], [FmtToken::Identifier]);

        expect_token!(parser => [Punct(Punctuation::Colon), ()], [FmtToken::Punct(Punctuation::Colon)]);

        let ty = parse!(parser => Type);

        args.push((name, ty));
        expect_token!(
            parser => [
                Punct(Punctuation::Comma), (); Punct(Punctuation::RParen), (), in break
            ],
            [FmtToken::Punct(Punctuation::Comma), FmtToken::Punct(Punctuation::RParen)]
        );
    }

    expect_token!(parser => [Punct(Punctuation::RParen), ()], [FmtToken::Punct(Punctuation::RParen)]);

    let ret = if let Some(Token {
        tt: Punct(Punctuation::ThinRArrow),
        ..
    }) = parser.try_peek_tok()
    {
        expect_token!(parser => [Punct(Punctuation::ThinRArrow), ()], [FmtToken::Punct(Punctuation::ThinRArrow)]);
        Some(parse!(parser => Type))
    } else {
        None
    };

    expect_token!(parser => [Punct(Punctuation::Equal), ()], [FmtToken::Punct(Punctuation::Equal)]);

    let block = parse!(parser => Block<Statement>);

    if let Some(node) = block.content.last() {
        loc.hi = node.loc.hi;
    }
    Fuzzy::Ok((
        DeclarationInner::Function {
            name,
            args,
            ret,
            block,
        },
        loc,
    ))
}
