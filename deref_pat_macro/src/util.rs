use proc_macro2::Span;
use std::iter;
use syn::{Block, Expr, ExprCall, ExprIf, ExprLet, ExprPath, Ident, Pat, Path, PathSegment, Stmt};

/// Creates an [`Ident`] with mixed site [`Span`].
pub fn create_ident(name: impl AsRef<str>) -> Ident {
    Ident::new(name.as_ref(), Span::mixed_site())
}

/// Creates a [`Path`] with the given segments.
/// `global` prepends `::`.
pub fn create_path<I>(segments: impl IntoIterator<Item = I>, global: bool) -> Path
where
    I: AsRef<str>,
{
    Path {
        leading_colon: if global {
            Some(Default::default())
        } else {
            None
        },
        segments: segments
            .into_iter()
            .map(|item| PathSegment::from(create_ident(item)))
            .collect(),
    }
}

/// Creates an [`ExprIf`] with a [`ExprLet`] as condition.
pub fn create_if_let(pat: Pat, input: Expr, body: Vec<Stmt>, else_branch: Option<Expr>) -> Expr {
    Expr::If(ExprIf {
        attrs: vec![],
        if_token: Default::default(),
        cond: Expr::Let(ExprLet {
            attrs: vec![],
            let_token: Default::default(),
            pat,
            eq_token: Default::default(),
            expr: input.into(),
        })
        .into(),
        then_branch: Block {
            brace_token: Default::default(),
            stmts: body,
        },
        else_branch: else_branch.map(|expr| (Default::default(), expr.into())),
    })
}

/// Creates a [`ExprMethodCall`].
pub fn create_call(func: Expr, args: impl IntoIterator<Item = Expr>) -> Expr {
    Expr::Call(ExprCall {
        attrs: vec![],
        func: func.into(),
        paren_token: Default::default(),
        args: args.into_iter().collect(),
    })
}

/// Helper to convert to an [`Expr`].
pub trait IntoExpr {
    fn into_expr(self) -> Expr;
}

impl IntoExpr for Ident {
    fn into_expr(self) -> Expr {
        self.into_path().into_expr()
    }
}

impl IntoExpr for Path {
    fn into_expr(self) -> Expr {
        Expr::Path(ExprPath {
            attrs: vec![],
            qself: None,
            path: self,
        })
    }
}

pub trait IntoPath {
    fn into_path(self) -> Path;
}

impl IntoPath for Ident {
    fn into_path(self) -> Path {
        Path {
            leading_colon: None,
            segments: iter::once(PathSegment::from(self)).collect(),
        }
    }
}

/// Helpers to convert to an [`Stmt`].
pub trait IntoStmt {
    fn into_semi_stmt(self) -> Stmt;
    fn into_expr_stmt(self) -> Stmt;
}

impl IntoStmt for Expr {
    fn into_semi_stmt(self) -> Stmt {
        Stmt::Semi(self, Default::default())
    }

    fn into_expr_stmt(self) -> Stmt {
        Stmt::Expr(self)
    }
}
