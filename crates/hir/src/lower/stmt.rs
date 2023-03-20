use fe_parser2::ast::{self, prelude::*};

use crate::{
    hir_def::{stmt::*, ArithBinOp, Expr, Pat, PathId, TypeId},
    span::{AugAssignDesugared, HirOriginKind},
};

use super::body::BodyCtxt;

impl Stmt {
    pub(super) fn push_to_body(ctxt: &mut BodyCtxt<'_>, ast: ast::Stmt) -> StmtId {
        let (stmt, origin_kind) = match ast.kind() {
            ast::StmtKind::Let(let_) => {
                let pat = Pat::push_to_body_opt(ctxt, let_.pat());
                let ty = let_
                    .type_annotation()
                    .map(|ty| TypeId::from_ast(ctxt.db, ctxt.fid, ty));
                let init = let_
                    .initializer()
                    .map(|init| Expr::push_to_body(ctxt, init));
                (Stmt::Let(pat, ty, init), HirOriginKind::raw(&ast))
            }
            ast::StmtKind::Assign(assign) => {
                let lhs = assign
                    .pat()
                    .map(|pat| Pat::push_to_body(ctxt, pat))
                    .unwrap_or_else(|| ctxt.push_missing_pat());

                let rhs = assign
                    .expr()
                    .map(|expr| Expr::push_to_body(ctxt, expr))
                    .unwrap_or_else(|| ctxt.push_missing_expr());
                (Stmt::Assign(lhs, rhs), HirOriginKind::raw(&ast))
            }

            ast::StmtKind::AugAssign(aug_assign) => desugar_aug_assign(ctxt, &aug_assign),

            ast::StmtKind::For(for_) => {
                let bind = Pat::push_to_body_opt(ctxt, for_.pat());
                let iter = Expr::push_to_body_opt(ctxt, for_.iterable());
                let body = Expr::push_to_body_opt(
                    ctxt,
                    for_.body()
                        .map(|body| ast::Expr::cast(body.syntax().clone()))
                        .flatten(),
                );

                (Stmt::For(bind, iter, body), HirOriginKind::raw(&ast))
            }

            ast::StmtKind::While(while_) => {
                let cond = Expr::push_to_body_opt(ctxt, while_.cond());
                let body = Expr::push_to_body_opt(
                    ctxt,
                    while_
                        .body()
                        .map(|body| ast::Expr::cast(body.syntax().clone()))
                        .flatten(),
                );

                (Stmt::While(cond, body), HirOriginKind::raw(&ast))
            }

            ast::StmtKind::Continue(_) => (Stmt::Continue, HirOriginKind::raw(&ast)),

            ast::StmtKind::Break(_) => (Stmt::Break, HirOriginKind::raw(&ast)),

            ast::StmtKind::Return(ret) => {
                let expr = ret
                    .has_value()
                    .then(|| Expr::push_to_body_opt(ctxt, ret.expr()));
                (Stmt::Return(expr), HirOriginKind::raw(&ast))
            }

            ast::StmtKind::Expr(expr) => {
                let expr = Expr::push_to_body_opt(ctxt, expr.expr());
                (Stmt::Expr(expr), HirOriginKind::raw(&ast))
            }
        };

        ctxt.push_stmt(stmt, origin_kind)
    }
}

fn desugar_aug_assign(
    ctxt: &mut BodyCtxt<'_>,
    ast: &ast::AugAssignStmt,
) -> (Stmt, HirOriginKind<ast::Stmt>) {
    let lhs_ident = ast.ident();
    let path = lhs_ident
        .clone()
        .map(|ident| PathId::from_ident(ctxt.db, ident));

    let lhs_origin: AugAssignDesugared = lhs_ident.clone().unwrap().text_range().into();
    let lhs_pat = if let Some(path) = path {
        ctxt.push_pat(
            Pat::Path(Some(path).into()),
            HirOriginKind::desugared(lhs_origin.clone()),
        )
    } else {
        ctxt.push_missing_pat()
    };

    let binop_lhs = if let Some(path) = path {
        ctxt.push_expr(
            Expr::Path(Some(path).into()),
            HirOriginKind::desugared(lhs_origin.clone()),
        )
    } else {
        ctxt.push_missing_expr()
    };

    let binop_rhs = ast
        .expr()
        .map(|expr| Expr::push_to_body(ctxt, expr))
        .unwrap_or_else(|| ctxt.push_missing_expr());

    let binop = ast.op().map(|op| ArithBinOp::from_ast(op).into()).into();
    let expr = ctxt.push_expr(
        Expr::Bin(binop_lhs, binop_rhs, binop),
        HirOriginKind::desugared(AugAssignDesugared::stmt(ast)),
    );

    (
        Stmt::Assign(lhs_pat, expr),
        HirOriginKind::desugared(AugAssignDesugared::stmt(ast)),
    )
}
