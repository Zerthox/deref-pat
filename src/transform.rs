#![allow(dead_code)]

use crate::{
    ident::IdentGen,
    util::{create_ident, create_path},
};
use std::mem;
use syn::{
    visit_mut::{self, VisitMut},
    Block, Expr, ExprBlock, ExprIf, ExprLet, ExprPath, ExprTuple, FieldPat, Ident, Pat, PatIdent,
    PatTuple, PatTupleStruct, Stmt,
};

/// Transforms deref patterns in the [`Expr`].
pub fn transform(mut expr: Expr) -> Expr {
    Transformer::default().visit_expr_mut(&mut expr);
    expr
}

#[derive(Debug, Default)]
struct Transformer {
    idents: IdentGen,
    collect: bool,
    deref_pats: Vec<DerefPat>,
    bound_idents: Vec<Ident>,
}

impl Transformer {
    const RESULT: &str = "deref_pat_result";

    fn gen_pat(&mut self) -> Pat {
        Pat::TupleStruct(PatTupleStruct {
            attrs: vec![],
            path: create_path(["core", "option", "Option", "Some"], true),
            pat: PatTuple {
                attrs: vec![],
                paren_token: Default::default(),
                elems: self
                    .bound_idents
                    .drain(..)
                    .map(|ident| {
                        Pat::Ident(PatIdent {
                            attrs: vec![],
                            by_ref: None,
                            mutability: None,
                            ident,
                            subpat: None,
                        })
                    })
                    .collect(),
            },
        })
    }

    fn gen_expr(&mut self, _input: Expr) -> Expr {
        let var = Expr::Let(ExprLet {
            attrs: vec![],
            let_token: Default::default(),
            pat: Pat::Ident(PatIdent {
                attrs: vec![],
                by_ref: None,
                mutability: Some(Default::default()),
                ident: create_ident(Self::RESULT),
                subpat: None,
            }),
            eq_token: Default::default(),
            expr: Expr::Path(ExprPath {
                attrs: vec![],
                qself: None,
                path: create_path(["core", "option", "Option", "None"], true),
            })
            .into(),
        });

        let result = Expr::Path(ExprPath {
            attrs: vec![],
            qself: None,
            path: create_path([Self::RESULT], false),
        });

        Expr::Block(ExprBlock {
            attrs: vec![],
            label: None,
            block: Block {
                brace_token: Default::default(),
                stmts: vec![Stmt::Semi(var, Default::default()), Stmt::Expr(result)],
            },
        })
    }
}

impl VisitMut for Transformer {
    fn visit_expr_if_mut(&mut self, if_expr: &mut ExprIf) {
        if let Expr::Let(let_guard) = if_expr.cond.as_mut() {
            let saved = self.collect;
            self.collect = true;

            // ensure fresh idents
            self.idents.reset();

            self.visit_pat_mut(&mut let_guard.pat);
            if !self.deref_pats.is_empty() {
                let_guard.pat = self.gen_pat();
                let input = mem::replace(
                    &mut let_guard.expr,
                    Expr::Tuple(ExprTuple {
                        attrs: vec![],
                        paren_token: Default::default(),
                        elems: Default::default(),
                    })
                    .into(),
                );
                let_guard.expr = self.gen_expr(*input).into();
            }

            self.collect = saved;
        } else {
            self.visit_expr_mut(&mut if_expr.cond);
        }

        self.visit_block_mut(&mut if_expr.then_branch);
        if let Some((_, else_branch)) = &mut if_expr.else_branch {
            self.visit_expr_mut(else_branch);
        }
    }

    fn visit_field_pat_mut(&mut self, field_pat: &mut FieldPat) {
        if self.collect {
            if let Some(pos) = field_pat
                .attrs
                .iter()
                .position(|attr| attr.path.is_ident("deref"))
            {
                field_pat.attrs.remove(pos);
                let ident = self.idents.next();
                let pat = mem::replace(
                    &mut field_pat.pat,
                    Pat::Ident(PatIdent {
                        attrs: vec![],
                        by_ref: None,
                        mutability: None,
                        ident: ident.clone(),
                        subpat: None,
                    })
                    .into(),
                );
                self.deref_pats.push(DerefPat { ident, pat: *pat });
            }
        }
        visit_mut::visit_field_pat_mut(self, field_pat);
    }

    fn visit_pat_ident_mut(&mut self, pat_ident: &mut PatIdent) {
        if self.collect {
            self.bound_idents.push(pat_ident.ident.clone());
            if let Some((_, pat)) = &mut pat_ident.subpat {
                self.visit_pat_mut(pat);
            }
        } else {
            visit_mut::visit_pat_ident_mut(self, pat_ident);
        }
    }
}

#[derive(Debug)]
struct DerefPat {
    pub ident: Ident,
    pub pat: Pat,
}
