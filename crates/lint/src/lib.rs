use meridian_ast::{Program, Stmt, Expr};
use meridian_diagnostics::{Diagnostic, DiagnosticCategory, Span};
use meridian_semantic::SemanticAnalyzer;
use std::collections::HashSet;

pub struct Linter {
    pub diagnostics: Vec<Diagnostic>,
}

impl Default for Linter {
    fn default() -> Self {
        Self::new()
    }
}

impl Linter {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    pub fn lint_program(&mut self, program: &Program, semantic: &SemanticAnalyzer) {
        for stmt in &program.statements {
            self.lint_stmt(stmt);
        }

        // Unused variables check via SemanticIndex
        let mut used_spans = HashSet::new();
        for def_span in semantic.index.usages.values() {
            used_spans.insert(*def_span);
        }

        for (name, def_span) in &semantic.index.definitions {
            // Ignore variables prefixed with _ or standard globals like stdlib functions
            if !name.starts_with('_') && !used_spans.contains(def_span) {
                // If the name is something we declared, we flag it. 
                // We'll skip flagging functions for now if we can't easily distinguish them, 
                // but since functions are also tracked, this might flag unused functions, which is good!
                self.diagnostics.push(Diagnostic::new(
                    format!("Identifier '{}' is declared but never used", name),
                    "MER0301".to_string(),
                    *def_span,
                    DiagnosticCategory::Lint,
                    Some(format!("If this is intentional, prefix it with an underscore: _{}", name)),
                ));
            }
        }
    }

    #[allow(clippy::collapsible_match)]
    fn lint_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Function { body, .. } => {
                if let Expr::Block(stmts, span) = body {
                    if stmts.is_empty() {
                        self.diagnostics.push(Diagnostic::new(
                            "Empty function body".to_string(),
                            "MER0302".to_string(),
                            *span,
                            DiagnosticCategory::Lint,
                            Some("Consider adding an implementation or removing the function if unused.".to_string()),
                        ));
                    }
                    self.lint_block(stmts);
                }
            }
            Stmt::While { body, .. } => {
                if let Expr::Block(stmts, span) = body {
                    if stmts.is_empty() {
                        self.diagnostics.push(Diagnostic::new(
                            "Empty while loop body".to_string(),
                            "MER0303".to_string(),
                            *span,
                            DiagnosticCategory::Lint,
                            Some("Consider adding logic or removing the loop.".to_string()),
                        ));
                    }
                    self.lint_block(stmts);
                }
            }
            Stmt::For { body, .. } => {
                if let Expr::Block(stmts, span) = body {
                    if stmts.is_empty() {
                        self.diagnostics.push(Diagnostic::new(
                            "Empty for loop body".to_string(),
                            "MER0304".to_string(),
                            *span,
                            DiagnosticCategory::Lint,
                            Some("Consider adding logic or removing the loop.".to_string()),
                        ));
                    }
                    self.lint_block(stmts);
                }
            }
            Stmt::Expr(expr) => self.lint_expr(expr),
            _ => {}
        }
    }

    fn lint_block(&mut self, stmts: &[Stmt]) {
        let mut found_terminator = false;
        for stmt in stmts {
            let span = match stmt {
                Stmt::Import(_, s) => *s,
                Stmt::Let { span, .. } => *span,
                Stmt::Expr(e) => {
                    // Expr has its own span, but we need to fetch it
                    // For now, we can just use a dummy span or implement a span() method on Expr.
                    // Let's implement a helper in Linter
                    self.expr_span(e)
                },
                Stmt::Print(_, s) => *s,
                Stmt::Function { span, .. } => *span,
                Stmt::While { span, .. } => *span,
                Stmt::For { span, .. } => *span,
                Stmt::Break(s) => *s,
                Stmt::Continue(s) => *s,
                Stmt::MacroDef { span, .. } => *span,
                Stmt::ExternBlock { span, .. } => *span,
                Stmt::StructDef { span, .. } => *span,
                Stmt::EnumDef { span, .. } => *span,
                Stmt::TraitDef { span, .. } => *span,
                Stmt::Impl { span, .. } => *span,
                Stmt::Return(_, s) => *s,
            };

            if found_terminator {
                self.diagnostics.push(Diagnostic::new(
                    "Unreachable code detected".to_string(),
                    "MER0305".to_string(),
                    span,
                    DiagnosticCategory::Lint,
                    Some("This statement follows a break, continue, or return, so it will never be executed.".to_string()),
                ));
            }
            
            if matches!(stmt, Stmt::Break(_) | Stmt::Continue(_) | Stmt::Return(_, _)) {
                found_terminator = true;
            }
            self.lint_stmt(stmt);
        }
    }

    fn expr_span(&self, expr: &Expr) -> Span {
        match expr {
            Expr::Number(_, s) => *s,
            Expr::Int(_, s) => *s,
            Expr::String(_, s) => *s,
            Expr::Bool(_, s) => *s,
            Expr::Identifier(_, s) => *s,
            Expr::Unary { span, .. } => *span,
            Expr::Binary { span, .. } => *span,
            Expr::Assign { span, .. } => *span,
            Expr::Call { span, .. } => *span,
            Expr::MethodCall { span, .. } => *span,
            Expr::Index { span, .. } => *span,
            Expr::If { span, .. } => *span,
            Expr::Block(_, s) => *s,
            Expr::Group(_, s) => *s,
            Expr::Range { span, .. } => *span,
            Expr::Borrow { span, .. } => *span,
            Expr::Dereference { span, .. } => *span,
            Expr::AsyncBlock { span, .. } => *span,
            Expr::Await { span, .. } => *span,
            Expr::Spawn { span, .. } => *span,
            Expr::UnsafeBlock { span, .. } => *span,
            Expr::Error(span) => *span,
            Expr::MacroCall { span, .. } => *span,
            Expr::StructInit { span, .. } => *span,
            Expr::FieldAccess { span, .. } => *span,
            Expr::FieldAssign { span, .. } => *span,
            Expr::ArrayInit { span, .. } => *span,
            Expr::Match { span, .. } => *span,
            Expr::EnumInit { span, .. } => *span,
            Expr::Try(_, span) => *span,
        }
    }

    fn lint_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Block(stmts, _) => self.lint_block(stmts),
            Expr::If { then_branch, else_branch, .. } => {
                self.lint_expr(then_branch);
                if let Some(e) = else_branch {
                    self.lint_expr(e);
                }
            }
            _ => {}
        }
    }
}
