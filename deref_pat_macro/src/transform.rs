use crate::{
    ident::IdentGen,
    util::{create_call, create_ident, create_if_let, create_path, ToExpr, ToStmt},
};
use std::{collections::VecDeque, iter, mem};
use syn::{
    visit_mut::{self, VisitMut},
    Block, Expr, ExprAssign, ExprBlock, ExprIf, ExprLet, ExprStruct, ExprTuple, FieldPat,
    FieldValue, Ident, Item, ItemUse, Member, Pat, PatIdent, PatTuple, PatTupleStruct, Stmt,
    UseName, UsePath, UseTree, Visibility,
};

/// Transforms deref patterns in the [`Expr`].
pub fn transform(mut expr: Expr) -> Expr {
    Transformer::default().visit_expr_mut(&mut expr);
    expr
}

// TODO: handle match expressions?

#[derive(Debug, Default)]
struct Transformer {
    idents: IdentGen,
    collect: bool,
    deref_pats: VecDeque<DerefPat>,
    bound_idents: Vec<Ident>,
}

impl Transformer {
    // FIXME: do not hardcode crate name
    const CRATE_NAME: &str = "deref_pat";

    const RESULT: &str = concat!("_", env!("CARGO_PKG_NAME"), "_result");

    fn create_import(crate_name: impl AsRef<str>) -> Stmt {
        Stmt::Item(Item::Use(ItemUse {
            attrs: vec![],
            vis: Visibility::Inherited,
            use_token: Default::default(),
            leading_colon: Some(Default::default()),
            tree: UseTree::Path(UsePath {
                ident: create_ident(crate_name),
                colon2_token: Default::default(),
                tree: UseTree::Name(UseName {
                    ident: create_ident("PatDeref"),
                })
                .into(),
            }),
            semi_token: Default::default(),
        }))
    }

    fn gen_pat(&self) -> Pat {
        Pat::TupleStruct(PatTupleStruct {
            attrs: vec![],
            path: create_path(["core", "option", "Option", "Some"], true),
            pat: PatTuple {
                attrs: vec![],
                paren_token: Default::default(),
                elems: iter::once(Pat::Tuple(PatTuple {
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
                }))
                .collect(),
            },
        })
    }

    fn gen_expr(&mut self, pat: Pat, input: Expr) -> Expr {
        let import = Self::create_import(Self::CRATE_NAME);

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
            expr: create_path(["core", "option", "Option", "None"], true)
                .to_expr()
                .into(),
        });

        let result = create_path([Self::RESULT], false).to_expr();

        let assign = Expr::Assign(ExprAssign {
            attrs: vec![],
            left: create_path([Self::RESULT], false).to_expr().into(),
            eq_token: Default::default(),
            right: Expr::Struct(ExprStruct {
                attrs: vec![],
                path: create_path(["core", "option", "Option", "Some"], true),
                brace_token: Default::default(),
                fields: iter::once(FieldValue {
                    attrs: vec![],
                    member: Member::Unnamed(0.into()),
                    colon_token: Some(Default::default()),
                    expr: Expr::Tuple(ExprTuple {
                        attrs: vec![],
                        paren_token: Default::default(),
                        elems: self
                            .bound_idents
                            .drain(..)
                            .map(|ident| ident.to_expr())
                            .collect(),
                    }),
                })
                .collect(),
                dot2_token: None,
                rest: None,
            })
            .into(),
        });

        let mut inner_if_let = self.deref_pats.pop_front().unwrap().to_expr(assign);
        while let Some(pat) = self.deref_pats.pop_front() {
            inner_if_let = pat.to_expr(inner_if_let);
        }

        let top_if_let = create_if_let(pat, input, vec![inner_if_let.to_semi_stmt()], None);

        Expr::Block(ExprBlock {
            attrs: vec![],
            label: None,
            block: Block {
                brace_token: Default::default(),
                stmts: vec![
                    import,
                    var.to_semi_stmt(),
                    top_if_let.to_semi_stmt(),
                    result.to_expr_stmt(),
                ],
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
                let pat = mem::replace(&mut let_guard.pat, self.gen_pat());
                let input = mem::replace(
                    &mut let_guard.expr,
                    Expr::Tuple(ExprTuple {
                        attrs: vec![],
                        paren_token: Default::default(),
                        elems: Default::default(),
                    })
                    .into(),
                );
                let_guard.expr = self.gen_expr(pat, *input).into();
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
        visit_mut::visit_field_pat_mut(self, field_pat);

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
                self.deref_pats.push_back(DerefPat { ident, pat: *pat });
            }
        }
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

impl DerefPat {
    pub fn to_expr(self, inner: Expr) -> Expr {
        create_if_let(
            self.pat,
            create_call(
                create_path(["PatDeref", "pat_deref"], false).to_expr(),
                [self.ident.to_expr()],
            ),
            vec![inner.to_semi_stmt()],
            None,
        )
    }
}
