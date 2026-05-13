use crate::err::{ExpectedField, UnexpectedAttribute};
use quote::quote;
use syn::{Expr, ExprAssign, LitStr, parse::ParseStream, spanned::Spanned};

pub struct FieldAssignment {
    pub field: LitStr,
    pub value: Expr,
}

impl FieldAssignment {
    pub fn parse(input: &mut ParseStream) -> syn::Result<FieldAssignment> {
        let ExprAssign {
            attrs, left, right, ..
        } = input.parse::<ExprAssign>()?;

        if let Some(attr) = attrs.first() {
            return Err(UnexpectedAttribute::at(attr).into());
        }

        let field_span = left.span();
        let field = match *left {
            Expr::Field(expr) => {
                if let Some(attr) = expr.attrs.first() {
                    return Err(UnexpectedAttribute::at(attr).into());
                }
                quote! { #expr }
            }
            Expr::Array(expr) => return Err(ExpectedField::at(expr).with("array").into()),
            Expr::Assign(expr) => return Err(ExpectedField::at(expr).with("assign").into()),
            Expr::Async(expr) => return Err(ExpectedField::at(expr).with("async").into()),
            Expr::Await(expr) => return Err(ExpectedField::at(expr).with("await").into()),
            Expr::Binary(expr) => return Err(ExpectedField::at(expr).with("binary").into()),
            Expr::Block(expr) => return Err(ExpectedField::at(expr).with("block").into()),
            Expr::Break(expr) => return Err(ExpectedField::at(expr).with("break").into()),
            Expr::Call(expr) => return Err(ExpectedField::at(expr).with("call").into()),
            Expr::Cast(expr) => return Err(ExpectedField::at(expr).with("cast").into()),
            Expr::Closure(expr) => return Err(ExpectedField::at(expr).with("closure").into()),
            Expr::Const(expr) => return Err(ExpectedField::at(expr).with("const").into()),
            Expr::Continue(expr) => return Err(ExpectedField::at(expr).with("continue").into()),
            Expr::ForLoop(expr) => return Err(ExpectedField::at(expr).with("forloop").into()),
            Expr::Group(expr) => return Err(ExpectedField::at(expr).with("group").into()),
            Expr::If(expr) => return Err(ExpectedField::at(expr).with("if").into()),
            Expr::Index(expr) => return Err(ExpectedField::at(expr).with("index").into()),
            Expr::Infer(expr) => return Err(ExpectedField::at(expr).with("infer").into()),
            Expr::Let(expr) => return Err(ExpectedField::at(expr).with("let").into()),
            Expr::Lit(expr) => return Err(ExpectedField::at(expr).with("lit").into()),
            Expr::Loop(expr) => return Err(ExpectedField::at(expr).with("loop").into()),
            Expr::Macro(expr) => return Err(ExpectedField::at(expr).with("macro").into()),
            Expr::Match(expr) => return Err(ExpectedField::at(expr).with("match").into()),
            Expr::MethodCall(expr) => {
                return Err(ExpectedField::at(expr).with("methodcall").into());
            }
            Expr::Paren(expr) => return Err(ExpectedField::at(expr).with("paren").into()),
            Expr::Path(expr) => {
                if let Some(attr) = expr.attrs.first() {
                    return Err(UnexpectedAttribute::at(attr).into());
                }
                quote! { #expr }
            }
            Expr::Range(expr) => return Err(ExpectedField::at(expr).with("range").into()),
            Expr::RawAddr(expr) => return Err(ExpectedField::at(expr).with("rawaddr").into()),
            Expr::Reference(expr) => return Err(ExpectedField::at(expr).with("reference").into()),
            Expr::Repeat(expr) => return Err(ExpectedField::at(expr).with("repeat").into()),
            Expr::Return(expr) => return Err(ExpectedField::at(expr).with("return").into()),
            Expr::Struct(expr) => return Err(ExpectedField::at(expr).with("struct").into()),
            Expr::Try(expr) => return Err(ExpectedField::at(expr).with("try").into()),
            Expr::TryBlock(expr) => return Err(ExpectedField::at(expr).with("tryblock").into()),
            Expr::Tuple(expr) => return Err(ExpectedField::at(expr).with("tuple").into()),
            Expr::Unary(expr) => return Err(ExpectedField::at(expr).with("unary").into()),
            Expr::Unsafe(expr) => return Err(ExpectedField::at(expr).with("unsafe").into()),
            Expr::Verbatim(expr) => return Err(ExpectedField::at(expr).with("verbatim").into()),
            Expr::While(expr) => return Err(ExpectedField::at(expr).with("while").into()),
            Expr::Yield(expr) => return Err(ExpectedField::at(expr).with("yield").into()),
            expr => return Err(ExpectedField::at(expr).with("unknown").into()),
        }
        .to_string();

        Ok(Self {
            field: LitStr::new(&field, field_span),
            value: *right,
        })
    }
}
