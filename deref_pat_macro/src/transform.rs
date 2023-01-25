use crate::{
    ident::IdentGen,
    util::{create_call, create_if_let, create_path, IntoExpr, IntoPath, IntoStmt},
};
use std::{collections::VecDeque, iter, mem};
use syn::{
    visit_mut::{self, VisitMut},
    Block, Expr, ExprAssign, ExprBlock, ExprLet, ExprTuple, FieldPat, Ident, Pat, PatIdent,
    PatTuple, PatTupleStruct,
};

/// Transforms deref patterns in the [`Expr`].
pub fn transform(mut expr: Expr) -> Expr {
    Transformer::default().visit_expr_mut(&mut expr);
    expr
}

// TODO: handle match expressions?

#[derive(Debug, Default)]
struct Transformer {
    /// Identifier generator.
    idents: IdentGen,

    /// Whether to collect deref patterns & bound identifiers.
    collect: bool,

    /// Collected deref patterns.
    deref_pats: VecDeque<DerefPat>,

    /// Collected bound identifiers.
    bound_idents: Vec<Ident>,
}

impl Transformer {
    fn gen_pat(&self) -> Pat {
        Pat::TupleStruct(PatTupleStruct {
            attrs: vec![],
            path: create_path(["core", "option", "Option", "Some"], true),
            pat: PatTuple {
                attrs: vec![],
                paren_token: Default::default(),
                elems: iter::once(if let [single] = self.bound_idents.as_slice() {
                    Pat::Ident(PatIdent {
                        attrs: vec![],
                        by_ref: None,
                        mutability: None,
                        ident: single.clone(),
                        subpat: None,
                    })
                } else {
                    Pat::Tuple(PatTuple {
                        attrs: vec![],
                        paren_token: Default::default(),
                        elems: self
                            .bound_idents
                            .iter()
                            .map(|ident| {
                                Pat::Ident(PatIdent {
                                    attrs: vec![],
                                    by_ref: None,
                                    mutability: None,
                                    ident: ident.clone(),
                                    subpat: None,
                                })
                            })
                            .collect(),
                    })
                })
                .collect(),
            },
        })
    }

    fn gen_expr(&mut self, pat: Pat, input: Expr) -> Expr {
        let var = Expr::Let(ExprLet {
            attrs: vec![],
            let_token: Default::default(),
            pat: Pat::Ident(PatIdent {
                attrs: vec![],
                by_ref: None,
                mutability: Some(Default::default()),
                ident: IdentGen::prefix("result"),
                subpat: None,
            }),
            eq_token: Default::default(),
            expr: create_path(["core", "option", "Option", "None"], true)
                .into_expr()
                .into(),
        });

        let result = IdentGen::prefix("result").into_path().into_expr();

        let assign = Expr::Assign(ExprAssign {
            attrs: vec![],
            left: IdentGen::prefix("result").into_path().into_expr().into(),
            eq_token: Default::default(),
            right: create_call(
                create_path(["core", "option", "Option", "Some"], true).into_expr(),
                [if self.bound_idents.len() == 1 {
                    IdentGen::prefix(format!("var_{}", self.bound_idents.pop().unwrap()))
                        .into_expr()
                } else {
                    Expr::Tuple(ExprTuple {
                        attrs: vec![],
                        paren_token: Default::default(),
                        elems: self
                            .bound_idents
                            .drain(..)
                            .map(|ident| IdentGen::prefix(format!("var_{}", ident)).into_expr())
                            .collect(),
                    })
                }],
            )
            .into(),
        });

        let mut inner_if_let = self.deref_pats.pop_front().unwrap().into_expr(assign);
        while let Some(pat) = self.deref_pats.pop_front() {
            inner_if_let = pat.into_expr(inner_if_let);
        }

        let top_if_let = create_if_let(pat, input, vec![inner_if_let.into_semi_stmt()], None);

        Expr::Block(ExprBlock {
            attrs: vec![],
            label: None,
            block: Block {
                brace_token: Default::default(),
                stmts: vec![
                    var.into_semi_stmt(),
                    top_if_let.into_semi_stmt(),
                    result.into_expr_stmt(),
                ],
            },
        })
    }
}

impl VisitMut for Transformer {
    fn visit_expr_let_mut(&mut self, expr_let: &mut ExprLet) {
        let saved = self.collect;
        self.collect = true;

        // ensure fresh idents
        self.idents.reset();

        self.visit_pat_mut(&mut expr_let.pat);
        if !self.deref_pats.is_empty() {
            let pat = mem::replace(&mut expr_let.pat, self.gen_pat());
            let input = mem::replace(
                &mut expr_let.expr,
                Expr::Tuple(ExprTuple {
                    attrs: vec![],
                    paren_token: Default::default(),
                    elems: Default::default(),
                })
                .into(),
            );
            expr_let.expr = self.gen_expr(pat, *input).into();
        }

        // ensure no leftover bound idents
        self.bound_idents.clear();

        self.collect = saved;
    }

    fn visit_field_pat_mut(&mut self, field_pat: &mut FieldPat) {
        visit_mut::visit_field_pat_mut(self, field_pat);

        if self.collect {
            if let Some(pos) = field_pat
                .attrs
                .iter()
                .position(|attr| attr.path.is_ident("deref"))
            {
                field_pat.attrs.remove(pos);
                field_pat.colon_token.get_or_insert(Default::default());
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
                self.deref_pats.push_back(DerefPat { ident, pat: *pat });
            }
        }
    }

    fn visit_pat_ident_mut(&mut self, pat_ident: &mut PatIdent) {
        // TODO: better way than just dirty check for ident?
        if self.collect && pat_ident.ident != "None" {
            self.bound_idents.push(pat_ident.ident.clone());
            pat_ident.ident = IdentGen::prefix(format!("var_{}", pat_ident.ident));
        }
        if let Some((_, pat)) = &mut pat_ident.subpat {
            self.visit_pat_mut(pat);
        }
    }
}

// FIXME: do not hardcode crate name
const CRATE_NAME: &str = "deref_pat";

#[derive(Debug)]
struct DerefPat {
    pub ident: Ident,
    pub pat: Pat,
}

impl DerefPat {
    pub fn into_expr(self, inner: Expr) -> Expr {
        create_if_let(
            self.pat,
            create_call(
                create_path([CRATE_NAME, "PatDeref", "pat_deref"], true).into_expr(),
                [self.ident.into_expr()],
            ),
            vec![inner.into_semi_stmt()],
            None,
        )
    }
}
