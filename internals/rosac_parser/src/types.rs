use crate::prelude::*;

#[derive(Debug, Clone)]
pub enum TypeInner {
    // unsigned integers
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    UInt,
    // signed integers
    Int8,
    Int16,
    Int32,
    Int64,
    Int,

    Bool,
    Char,
    // String,

    // TODO: implement parsing for function pointers
    // e.g: `fun (int, bool) -> int` is a fn ptr
    // like `fun ()` is also a fn ptr
    FnPtr {
        args: Vec<Type>,
        ret: Option<Box<Type>>,
    },
}

impl TypeInner {
    pub fn is_primitive_type(ty: &str) -> bool {
        matches!(
            ty,
            "uint8"
                | "uint16"
                | "uint32"
                | "uint64"
                | "uint"
                | "int8"
                | "int16"
                | "int32"
                | "int64"
                | "int"
                | "bool"
                | "char"
        )
    }
}

#[derive(Debug, Clone)]
pub struct Type {
    pub ty: TypeInner,
    pub loc: Span,
}

derive_loc!(Type);

impl AstNode for Type {
    type Output = Self;

    fn parse<L: AbsLexer>(parser: &mut Parser<'_, L>) -> Fuzzy<Self::Output, Diag> {
        match parser.peek_tok() {
            Token {
                tt: Ident(name), ..
            } if TypeInner::is_primitive_type(name) => parse_primitive_type(parser),
            t => {
                let tok = t.clone();
                Fuzzy::Err(
                    parser
                        .dcx()
                        .struct_err(expected_tok_msg(tok.tt, [AstPart::Type]), tok.loc),
                )
            }
        }
    }
}

pub fn parse_primitive_type(parser: &mut Parser<'_, impl AbsLexer>) -> Fuzzy<Type, Diag> {
    let (ty_str, loc) =
        expect_token!(parser => [Ident(ty_str), ty_str.clone()], [FmtToken::Identifier]);

    let ty = match ty_str.as_str() {
        "uint8" => TypeInner::UInt8,
        "uint16" => TypeInner::UInt16,
        "uint32" => TypeInner::UInt32,
        "uint64" => TypeInner::UInt64,
        "uint" => TypeInner::UInt,

        "int8" => TypeInner::Int8,
        "int16" => TypeInner::Int16,
        "int32" => TypeInner::Int32,
        "int64" => TypeInner::Int64,
        "int" => TypeInner::Int,

        "bool" => TypeInner::Bool,
        "char" => TypeInner::Char,
        _ => {
            return Fuzzy::Err(parser.dcx().struct_err(
                expected_tok_msg(FmtToken::NamedIdentifier(ty_str), ["primitive type"]),
                loc,
            ))
        }
    };

    Fuzzy::Ok(Type { ty, loc })
}
