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
            used_spans.insert(def_span.clone());
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
                    def_span.clone(),
                    DiagnosticCategory::Lint,
                    Some(format!("If this is intentional, prefix it with an underscore: _{}", name)),
                ));
            }
        }
    }

    fn lint_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Function { body, .. } => {
                if let Expr::Block(stmts, span) = body {
                    if stmts.is_empty() {
                        self.diagnostics.push(Diagnostic::new(
                            "Empty function body".to_string(),
                            "MER0302".to_string(),
                            span.clone(),
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
                            span.clone(),
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
                            span.clone(),
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
                Stmt::Import(_, s) => s.clone(),
                Stmt::Let { span, .. } => span.clone(),
                Stmt::Expr(e) => {
                    // Expr has its own span, but we need to fetch it
                    // For now, we can just use a dummy span or implement a span() method on Expr.
                    // Let's implement a helper in Linter
                    self.expr_span(e)
                },
                Stmt::Print(_, s) => s.clone(),
                Stmt::Function { span, .. } => span.clone(),
                Stmt::While { span, .. } => span.clone(),
                Stmt::For { span, .. } => span.clone(),
                Stmt::Break(s) => s.clone(),
                Stmt::Continue(s) => s.clone(),
                Stmt::MacroDef { span, .. } => span.clone(),
                Stmt::ExternBlock { span, .. } => span.clone(),
                Stmt::StructDef { span, .. } => span.clone(),
                Stmt::EnumDef { span, .. } => span.clone(),
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
            
            if matches!(stmt, Stmt::Break(_) | Stmt::Continue(_)) {
                found_terminator = true;
            }
            self.lint_stmt(stmt);
        }
    }

    fn expr_span(&self, expr: &Expr) -> Span {
        match expr {
            Expr::Number(_, s) => s.clone(),
            Expr::String(_, s) => s.clone(),
            Expr::Bool(_, s) => s.clone(),
            Expr::Identifier(_, s) => s.clone(),
            Expr::Binary { span, .. } => span.clone(),
            Expr::Assign { span, .. } => span.clone(),
            Expr::Call { span, .. } => span.clone(),
            Expr::MethodCall { span, .. } => span.clone(),
            Expr::Index { span, .. } => span.clone(),
            Expr::If { span, .. } => span.clone(),
            Expr::Block(_, s) => s.clone(),
            Expr::Group(_, s) => s.clone(),
            Expr::Range { span, .. } => span.clone(),
            Expr::Borrow { span, .. } => span.clone(),
            Expr::Dereference { span, .. } => span.clone(),
            Expr::AsyncBlock { span, .. } => span.clone(),
            Expr::Await { span, .. } => span.clone(),
            Expr::Spawn { span, .. } => span.clone(),
            Expr::UnsafeBlock { span, .. } => span.clone(),
            Expr::Error(span) => span.clone(),
            Expr::MacroCall { span, .. } => span.clone(),
            Expr::StructInit { span, .. } => span.clone(),
            Expr::FieldAccess { span, .. } => span.clone(),
            Expr::FieldAssign { span, .. } => span.clone(),
            Expr::ArrayInit { span, .. } => span.clone(),
            Expr::Int(_, span) => span.clone(),
            Expr::Match { span, .. } => span.clone(),
            Expr::EnumInit { span, .. } => span.clone(),
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
