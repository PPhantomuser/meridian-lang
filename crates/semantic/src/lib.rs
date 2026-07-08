use std::collections::HashMap;
use meridian_ast::{BinaryOperator, Expr, Program, Stmt, Type};
use meridian_diagnostics::{Diagnostic, DiagnosticCategory, Span};

#[derive(Clone, Default)]
pub struct SemanticIndex {
    pub usages: HashMap<Span, Span>,
    pub definitions: HashMap<String, Span>,
}

#[derive(Clone, PartialEq, Eq)]
pub enum BorrowKind {
    Shared,
    Exclusive,
}

#[derive(Clone)]
struct SymbolInfo {
    ty: Type,
    mutable: bool,
    span: Span,
    borrows: Vec<BorrowKind>,
}

#[derive(Clone)]
struct FunctionSignature {
    parameters: Vec<Type>,
    return_type: Type,
    span: Span,
    is_extern: bool,
}

#[derive(Clone)]
struct MacroDefinition {
    parameters: Vec<meridian_ast::Parameter>,
    body: Expr,
}

pub struct SemanticAnalyzer {
    pub diagnostics: Vec<Diagnostic>,
    scopes: Vec<HashMap<String, SymbolInfo>>,
    functions: HashMap<String, FunctionSignature>,
    macros: HashMap<String, MacroDefinition>,
    current_function_return_type: Option<Type>,
    in_loop_depth: usize,
    macro_expansion_depth: usize,
    in_unsafe_block: bool,
    pub index: SemanticIndex,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        let mut analyzer = Self {
            diagnostics: Vec::new(),
            scopes: vec![HashMap::new()],
            functions: HashMap::new(),
            macros: HashMap::new(),
            current_function_return_type: None,
            in_loop_depth: 0,
            macro_expansion_depth: 0,
            in_unsafe_block: false,
            index: SemanticIndex::default(),
        };
        // Register standard library
        let native_funcs = vec![
            ("read_file", vec![Type::String], Type::String),
            ("hashmap_new", vec![], Type::Unknown),
            ("hashmap_insert", vec![Type::Unknown, Type::String, Type::Unknown], Type::Unknown),
            ("hashmap_get", vec![Type::Unknown, Type::String], Type::Unknown),
            ("hashset_new", vec![], Type::Unknown),
            ("hashset_insert", vec![Type::Unknown, Type::String], Type::Unknown),
            ("hashset_contains", vec![Type::Unknown, Type::String], Type::Bool),
            ("vecdeque_new", vec![], Type::Unknown),
            ("vecdeque_push_back", vec![Type::Unknown, Type::Unknown], Type::Unknown),
            ("vecdeque_pop_front", vec![Type::Unknown], Type::Unknown),
            ("tcp_bind", vec![Type::String], Type::Unknown),
            ("tcp_accept", vec![Type::Unknown], Type::Unknown),
            ("tcp_read", vec![Type::Unknown], Type::String),
            ("tcp_write", vec![Type::Unknown, Type::String], Type::Bool),
            ("file_write", vec![Type::String, Type::String], Type::Bool),
            ("file_append", vec![Type::String, Type::String], Type::Bool),
            ("file_delete", vec![Type::String], Type::Bool),
            ("file_exists", vec![Type::String], Type::Bool),
            ("process_output", vec![Type::String, Type::Unknown], Type::String),
            ("dlopen", vec![Type::String], Type::Unknown),
            ("dlsym", vec![Type::Unknown, Type::String, Type::String], Type::Unknown),
            ("dlcall", vec![Type::Unknown, Type::Unknown], Type::Unknown),
        ];

        for (name, params, ret) in native_funcs {
            analyzer.register_native_function(name, params, ret);
        }
        analyzer
    }

    pub fn register_native_function(&mut self, name: &str, params: Vec<Type>, ret: Type) {
        self.functions.insert(name.to_string(), FunctionSignature {
            parameters: params,
            return_type: ret,
            span: Span::new(0, 0),
            is_extern: false,
        });
    }

    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare_variable(&mut self, name: String, ty: Type, mutable: bool, span: Span) {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(&name) {
                self.diagnostics.push(Diagnostic::new(
                    format!("Variable '{}' is already declared in this scope", name),
                    "MER0103".to_string(),
                    span,
                    DiagnosticCategory::Semantic,
                    Some("Use a different name or mutate the existing variable.".to_string()),
                ));
            } else {
                self.index.definitions.insert(name.clone(), span);
                scope.insert(name, SymbolInfo { ty, mutable, span, borrows: Vec::new() });
            }
        }
    }

    fn borrow_variable(&mut self, name: &str, kind: BorrowKind, span: Span) -> bool {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(info) = scope.get_mut(name) {
                if kind == BorrowKind::Exclusive {
                    if !info.borrows.is_empty() {
                        self.diagnostics.push(Diagnostic::new(
                            format!("Cannot borrow '{}' mutably because it is already borrowed", name),
                            "MER0120".to_string(),
                            span,
                            DiagnosticCategory::Semantic,
                            None,
                        ));
                        return false;
                    }
                    if !info.mutable {
                        self.diagnostics.push(Diagnostic::new(
                            format!("Cannot borrow immutable variable '{}' mutably", name),
                            "MER0121".to_string(),
                            span,
                            DiagnosticCategory::Semantic,
                            Some("Declare the variable as 'mut'.".to_string()),
                        ));
                        return false;
                    }
                } else {
                    if info.borrows.contains(&BorrowKind::Exclusive) {
                        self.diagnostics.push(Diagnostic::new(
                            format!("Cannot borrow '{}' immutably because it is already borrowed mutably", name),
                            "MER0122".to_string(),
                            span,
                            DiagnosticCategory::Semantic,
                            None,
                        ));
                        return false;
                    }
                }
                info.borrows.push(kind);
                return true;
            }
        }
        false
    }

    fn lookup_variable(&self, name: &str) -> Option<SymbolInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Some(info.clone());
            }
        }
        None
    }

    pub fn analyze_program(&mut self, program: &Program) {
        // Pass 1: Hoist functions
        for stmt in &program.statements {
            if let Stmt::Function { name, parameters, return_type, span, .. } = stmt {
                if self.functions.contains_key(name) {
                    self.diagnostics.push(Diagnostic::new(
                        format!("Function '{}' is already defined", name),
                        "MER0109".to_string(),
                        *span,
                        DiagnosticCategory::Semantic,
                        None,
                    ));
                } else {
                    let param_types = parameters.iter().map(|p| p.ty.clone()).collect();
                    self.functions.insert(name.clone(), FunctionSignature {
                        parameters: param_types,
                        return_type: return_type.clone(),
                        span: *span,
                        is_extern: false,
                    });
                    self.index.definitions.insert(name.clone(), *span);
                }
            } else if let Stmt::ExternBlock { functions, .. } = stmt {
                for func in functions {
                    if let Stmt::Function { name, parameters, return_type, span, .. } = func {
                        if self.functions.contains_key(name) {
                            self.diagnostics.push(Diagnostic::new(
                                format!("Function '{}' is already defined", name),
                                "MER0109".to_string(),
                                *span,
                                DiagnosticCategory::Semantic,
                                None,
                            ));
                        } else {
                            let param_types = parameters.iter().map(|p| p.ty.clone()).collect();
                            self.functions.insert(name.clone(), FunctionSignature {
                                parameters: param_types,
                                return_type: return_type.clone(),
                                span: *span,
                                is_extern: true,
                            });
                            self.index.definitions.insert(name.clone(), *span);
                        }
                    }
                }
            }
        }

        // Pass 2: Analyze everything
        for stmt in &program.statements {
            self.analyze_statement(stmt);
        }
    }

    fn analyze_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let { name, mutable, type_annotation, initializer, span } => {
                let init_type = self.analyze_expression(initializer);
                
                let final_type = if let Some(annotated_type) = type_annotation {
                    if init_type != Type::Unknown && init_type != Type::Error && *annotated_type != init_type {
                        self.diagnostics.push(Diagnostic::new(
                            format!("Type mismatch: expected {:?}, found {:?}", annotated_type, init_type),
                            "MER0102".to_string(),
                            initializer.span(),
                            DiagnosticCategory::Type,
                            Some("Ensure the initializer matches the declared type.".to_string()),
                        ));
                    }
                    annotated_type.clone()
                } else {
                    init_type
                };

                self.declare_variable(name.clone(), final_type, *mutable, *span);
            }
            Stmt::Print(expr, _) => {
                self.analyze_expression(expr);
            }
            Stmt::Expr(expr) => {
                self.analyze_expression(expr);
            }
            Stmt::Function { name, parameters, return_type, is_async, body, span, doc_comment: _, attributes: _ } => {
                let mut ref_param_count = 0;
                for param in parameters {
                    if let Type::Reference(_, _) = param.ty {
                        ref_param_count += 1;
                    }
                }

                let returns_ref = matches!(return_type, Type::Reference(_, _));
                if returns_ref && ref_param_count != 1 {
                    self.diagnostics.push(Diagnostic::new(
                        format!("Function '{}' returns a reference but takes {} reference parameters. Lifetime elision fails.", name, ref_param_count),
                        "MER0123".to_string(),
                        *span,
                        DiagnosticCategory::Semantic,
                        Some("Wrap the return type in Rc<T> or Arc<T> since explicit lifetimes are not supported.".to_string()),
                    ));
                }

                let sig = FunctionSignature {
                    parameters: parameters.iter().map(|p| p.ty.clone()).collect(),
                    return_type: return_type.clone(),
                    span: *span,
                    is_extern: false,
                };
                self.functions.insert(name.clone(), sig);

                self.enter_scope();
                for param in parameters {
                    self.declare_variable(param.name.clone(), param.ty.clone(), false, param.span);
                }

                self.current_function_return_type = Some(return_type.clone());
                
                let actual_return_type = self.analyze_expression(body);
                
                let expected_ty = if *is_async {
                    if let Type::Future(inner) = return_type {
                        *inner.clone()
                    } else {
                        self.diagnostics.push(Diagnostic::new(
                            format!("Async function `{}` must return Future<T>", name),
                            "MER0130".to_string(),
                            *span,
                            DiagnosticCategory::Semantic,
                            None,
                        ));
                        Type::Error
                    }
                } else {
                    return_type.clone()
                };

                if actual_return_type != expected_ty && actual_return_type != Type::Error && actual_return_type != Type::Unknown {
                    self.diagnostics.push(Diagnostic::new(
                        format!(
                            "Function `{}` expected to return type `{:?}`, but found `{:?}`",
                            name, expected_ty, actual_return_type
                        ),
                        "MER0111".to_string(),
                        body.span(),
                        DiagnosticCategory::Semantic,
                        None,
                    ));
                }
                
                self.exit_scope();
            }
            Stmt::While { condition, body, .. } => {
                let cond_ty = self.analyze_expression(condition);
                if cond_ty != Type::Bool && cond_ty != Type::Error && cond_ty != Type::Unknown {
                    self.diagnostics.push(Diagnostic::new(
                        format!("'while' condition must be a Bool, found {:?}", cond_ty),
                        "MER0113".to_string(),
                        condition.span(),
                        DiagnosticCategory::Type,
                        None,
                    ));
                }
                
                self.in_loop_depth += 1;
                self.analyze_expression(body);
                self.in_loop_depth -= 1;
            }
            Stmt::For { iterator, iterable, body, span } => {
                let iter_ty = self.analyze_expression(iterable);
                let yielded_ty = if iter_ty == Type::Number || iter_ty == Type::Unknown || iter_ty == Type::Error {
                    Type::Number
                } else {
                    self.diagnostics.push(Diagnostic::new(
                        format!("Cannot iterate over type {:?}", iter_ty),
                        "MER0114".to_string(),
                        iterable.span(),
                        DiagnosticCategory::Type,
                        None,
                    ));
                    Type::Error
                };
                
                self.enter_scope();
                self.declare_variable(iterator.clone(), yielded_ty, false, *span);
                self.in_loop_depth += 1;
                self.analyze_expression(body);
                self.in_loop_depth -= 1;
                self.exit_scope();
            }
            Stmt::Break(span) => {
                if self.in_loop_depth == 0 {
                    self.diagnostics.push(Diagnostic::new(
                        "'break' outside of loop".to_string(),
                        "MER0115".to_string(),
                        *span,
                        DiagnosticCategory::Semantic,
                        None,
                    ));
                }
            }
            Stmt::Continue(span) => {
                if self.in_loop_depth == 0 {
                    self.diagnostics.push(Diagnostic::new(
                        "'continue' outside of loop".to_string(),
                        "MER0116".to_string(),
                        *span,
                        DiagnosticCategory::Semantic,
                        None,
                    ));
                }
            }
            Stmt::MacroDef { name, parameters, body, span } => {
                self.macros.insert(name.clone(), MacroDefinition {
                    parameters: parameters.clone(),
                    body: body.clone(),
                });
                self.index.definitions.insert(name.clone(), *span);
            }
            Stmt::ExternBlock { .. } => {}
            Stmt::Import(_, _) => {}
        }
    }

    fn analyze_expression(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Number(_, _) => Type::Number,
            Expr::String(_, _) => Type::String,
            Expr::Bool(_, _) => Type::Bool,
            Expr::Identifier(name, span) => {
                if let Some(info) = self.lookup_variable(name) {
                    self.index.usages.insert(*span, info.span);
                    info.ty
                } else {
                    self.diagnostics.push(Diagnostic::new(
                        format!("Cannot find value '{}' in this scope", name),
                        "MER0100".to_string(),
                        *span,
                        DiagnosticCategory::Semantic,
                        Some("Declare the variable before using it.".to_string()),
                    ));
                    Type::Error
                }
            }
            Expr::Binary { left, operator, right, span } => {
                let left_ty = self.analyze_expression(left);
                let right_ty = self.analyze_expression(right);

                if left_ty == Type::Error || right_ty == Type::Error {
                    return Type::Error;
                }
                
                if left_ty == Type::Unknown || right_ty == Type::Unknown {
                    return Type::Unknown;
                }

                if left_ty != right_ty {
                    self.diagnostics.push(Diagnostic::new(
                        format!("Type mismatch in binary operation: {:?} and {:?}", left_ty, right_ty),
                        "MER0102".to_string(),
                        *span,
                        DiagnosticCategory::Type,
                        None,
                    ));
                    return Type::Error;
                }

                match operator {
                    BinaryOperator::Add | BinaryOperator::Subtract | BinaryOperator::Multiply | BinaryOperator::Divide => {
                        if left_ty == Type::Number {
                            Type::Number
                        } else if left_ty == Type::String && *operator == BinaryOperator::Add {
                            Type::String
                        } else {
                            self.diagnostics.push(Diagnostic::new(
                                format!("Cannot apply operator {:?} to type {:?}", operator, left_ty),
                                "MER0104".to_string(),
                                *span,
                                DiagnosticCategory::Type,
                                None,
                            ));
                            Type::Error
                        }
                    }
                    BinaryOperator::Equal | BinaryOperator::NotEqual => {
                        Type::Bool
                    }
                    BinaryOperator::LessThan | BinaryOperator::LessThanEqual | BinaryOperator::GreaterThan | BinaryOperator::GreaterThanEqual => {
                        if left_ty == Type::Number {
                            Type::Bool
                        } else {
                            self.diagnostics.push(Diagnostic::new(
                                format!("Cannot apply operator {:?} to type {:?}", operator, left_ty),
                                "MER0104".to_string(),
                                *span,
                                DiagnosticCategory::Type,
                                None,
                            ));
                            Type::Error
                        }
                    }
                }
            }
            Expr::If { condition, then_branch, else_branch, span } => {
                let cond_ty = self.analyze_expression(condition);
                if cond_ty != Type::Bool && cond_ty != Type::Error && cond_ty != Type::Unknown {
                    self.diagnostics.push(Diagnostic::new(
                        format!("'if' condition must be a Bool, found {:?}", cond_ty),
                        "MER0105".to_string(),
                        condition.span(),
                        DiagnosticCategory::Type,
                        None,
                    ));
                }

                let then_ty = self.analyze_expression(then_branch);
                if let Some(else_expr) = else_branch {
                    let else_ty = self.analyze_expression(else_expr);
                    if then_ty != Type::Error && else_ty != Type::Error && then_ty != else_ty {
                        self.diagnostics.push(Diagnostic::new(
                            format!("'if' and 'else' branches have incompatible types: {:?} and {:?}", then_ty, else_ty),
                            "MER0106".to_string(),
                            *span,
                            DiagnosticCategory::Type,
                            None,
                        ));
                        Type::Error
                    } else {
                        then_ty
                    }
                } else {
                    if then_ty != Type::Unit && then_ty != Type::Error {
                        self.diagnostics.push(Diagnostic::new(
                            format!("'if' expression missing an 'else' must return Unit, found {:?}", then_ty),
                            "MER0107".to_string(),
                            *span,
                            DiagnosticCategory::Type,
                            None,
                        ));
                    }
                    Type::Unit
                }
            }
            Expr::Block(statements, _) => {
                self.enter_scope();
                let mut block_type = Type::Unit;
                
                for (i, stmt) in statements.iter().enumerate() {
                    if i == statements.len() - 1 {
                        if let Stmt::Expr(e) = stmt {
                            block_type = self.analyze_expression(e);
                        } else {
                            self.analyze_statement(stmt);
                        }
                    } else {
                        self.analyze_statement(stmt);
                    }
                }
                
                self.exit_scope();
                block_type
            }
            Expr::Assign { target, value, span } => {
                let val_ty = self.analyze_expression(value);
                
                if let Expr::Identifier(name, _) = &**target {
                    if let Some(info) = self.lookup_variable(name) {
                        if !info.mutable {
                            self.diagnostics.push(Diagnostic::new(
                                format!("Cannot assign twice to immutable variable '{}'", name),
                                "MER0101".to_string(),
                                *span,
                                DiagnosticCategory::Semantic,
                                Some("Declare the variable with 'mut' to allow assignment.".to_string()),
                            ));
                        } else if info.ty != val_ty && val_ty != Type::Error && info.ty != Type::Unknown {
                            self.diagnostics.push(Diagnostic::new(
                                format!("Type mismatch: cannot assign type {:?} to variable of type {:?}", val_ty, info.ty),
                                "MER0102".to_string(),
                                *span,
                                DiagnosticCategory::Type,
                                None,
                            ));
                        }
                    } else {
                        self.diagnostics.push(Diagnostic::new(
                            format!("Cannot find value '{}' in this scope", name),
                            "MER0100".to_string(),
                            *span,
                            DiagnosticCategory::Semantic,
                            Some("Declare the variable before assigning to it.".to_string()),
                        ));
                    }
                } else {
                    self.diagnostics.push(Diagnostic::new(
                        "Invalid left-hand side of assignment".to_string(),
                        "MER0108".to_string(),
                        target.span(),
                        DiagnosticCategory::Syntax,
                        Some("The target of an assignment must be a variable identifier.".to_string()),
                    ));
                }
                
                val_ty
            }
            Expr::Call { callee, arguments, span } => {
                if let Expr::Identifier(name, callee_span) = &**callee {
                    if let Some(signature) = self.functions.get(name).cloned() {
                        self.index.usages.insert(*callee_span, signature.span);
                        if signature.is_extern && !self.in_unsafe_block {
                            self.diagnostics.push(Diagnostic::new(
                                format!("Call to external function '{}' is unsafe and requires an unsafe block", name),
                                "MER0150".to_string(),
                                *span,
                                DiagnosticCategory::Semantic,
                                Some("Wrap the call in an unsafe { ... } block.".to_string()),
                            ));
                        }
                        if self.macro_expansion_depth > 0 {
                            let unsafe_funcs = [
                                "process_output", "process_spawn", "tcp_bind", "tcp_accept", "tcp_read", "tcp_write",
                                "file_write", "file_append", "file_delete", "file_exists", "read_file"
                            ];
                            if unsafe_funcs.contains(&name.as_str()) {
                                self.diagnostics.push(Diagnostic::new(
                                    format!("Macro expansion resulted in a call to unsafe system function '{}'", name),
                                    "MER0151".to_string(),
                                    *span,
                                    DiagnosticCategory::Semantic,
                                    Some("System calls within macros are restricted by the Safe Harbor policies.".to_string()),
                                ));
                            }
                        }
                        if arguments.len() != signature.parameters.len() {
                            self.diagnostics.push(Diagnostic::new(
                                format!("Function '{}' expects {} arguments, but {} were provided", name, signature.parameters.len(), arguments.len()),
                                "MER0110".to_string(),
                                *span,
                                DiagnosticCategory::Semantic,
                                None,
                            ));
                        } else {
                            for (i, arg) in arguments.iter().enumerate() {
                                let arg_ty = self.analyze_expression(arg);
                                let expected_ty = &signature.parameters[i];
                                if arg_ty != *expected_ty && arg_ty != Type::Error && arg_ty != Type::Unknown && *expected_ty != Type::Unknown {
                                    self.diagnostics.push(Diagnostic::new(
                                        format!("Type mismatch in argument {}: expected {:?}, found {:?}", i + 1, expected_ty, arg_ty),
                                        "MER0102".to_string(),
                                        arg.span(),
                                        DiagnosticCategory::Type,
                                        None,
                                    ));
                                }
                            }
                        }
                        signature.return_type
                    } else {
                        self.diagnostics.push(Diagnostic::new(
                            format!("Cannot find function '{}' in this scope", name),
                            "MER0100".to_string(),
                            *span,
                            DiagnosticCategory::Semantic,
                            None,
                        ));
                        Type::Error
                    }
                } else {
                    self.diagnostics.push(Diagnostic::new(
                        "Cannot call a non-identifier expression".to_string(),
                        "MER0112".to_string(),
                        callee.span(),
                        DiagnosticCategory::Syntax,
                        None,
                    ));
                    for arg in arguments {
                        self.analyze_expression(arg);
                    }
                    Type::Error
                }
            }
            Expr::Group(expr, _) => {
                self.analyze_expression(expr)
            }
            Expr::Range { start, end, .. } => {
                let start_ty = self.analyze_expression(start);
                let end_ty = self.analyze_expression(end);
                
                if start_ty != Type::Number && start_ty != Type::Error && start_ty != Type::Unknown {
                    self.diagnostics.push(Diagnostic::new(
                        format!("Range start must be a Number, found {:?}", start_ty),
                        "MER0117".to_string(),
                        start.span(),
                        DiagnosticCategory::Type,
                        None,
                    ));
                }
                
                if end_ty != Type::Number && end_ty != Type::Error && end_ty != Type::Unknown {
                    self.diagnostics.push(Diagnostic::new(
                        format!("Range end must be a Number, found {:?}", end_ty),
                        "MER0118".to_string(),
                        end.span(),
                        DiagnosticCategory::Type,
                        None,
                    ));
                }
                
                Type::Number
            }
            Expr::Borrow { expr, is_mut, span } => {
                let inner_ty = self.analyze_expression(expr);
                
                if let Expr::Identifier(name, _) = &**expr {
                    let kind = if *is_mut { BorrowKind::Exclusive } else { BorrowKind::Shared };
                    self.borrow_variable(name, kind, *span);
                }
                
                if inner_ty == Type::Error {
                    Type::Error
                } else {
                    Type::Reference(Box::new(inner_ty), *is_mut)
                }
            }
            Expr::Dereference { expr, span } => {
                let inner_ty = self.analyze_expression(expr);
                match inner_ty {
                    Type::Reference(ty, _) => *ty,
                    Type::RawPointer(ty, _) => {
                        if !self.in_unsafe_block {
                            self.diagnostics.push(Diagnostic::new(
                                "Dereferencing a raw pointer is unsafe and requires an unsafe block".to_string(),
                                "MER0151".to_string(),
                                *span,
                                DiagnosticCategory::Semantic,
                                Some("Wrap the dereference in an unsafe { ... } block.".to_string()),
                            ));
                        }
                        *ty
                    }
                    Type::Error | Type::Unknown => Type::Error,
                    _ => {
                        self.diagnostics.push(Diagnostic::new(
                            format!("Cannot dereference type {:?}", inner_ty),
                            "MER0124".to_string(),
                            *span,
                            DiagnosticCategory::Type,
                            None,
                        ));
                        Type::Error
                    }
                }
            }
            Expr::AsyncBlock { statements, span: _ } => {
                self.enter_scope();
                let mut last_type = Type::Unit;
                for stmt in statements {
                    self.analyze_statement(stmt);
                }
                self.exit_scope();
                Type::Future(Box::new(last_type))
            }
            Expr::Await { expr, span } => {
                let inner_ty = self.analyze_expression(expr);
                if let Type::Future(t) = inner_ty {
                    *t
                } else {
                    self.diagnostics.push(Diagnostic::new(
                        format!("Cannot await non-future type `{:?}`", inner_ty),
                        "MER0131".to_string(),
                        *span,
                        DiagnosticCategory::Semantic,
                        None,
                    ));
                    Type::Error
                }
            }
            Expr::Spawn { expr, span: _ } => {
                let inner_ty = self.analyze_expression(expr);
                if let Type::Future(_) = inner_ty {
                    inner_ty
                } else {
                    self.diagnostics.push(Diagnostic::new(
                        format!("Cannot spawn non-future type `{:?}`", inner_ty),
                        "MER0132".to_string(),
                        expr.span(),
                        DiagnosticCategory::Semantic,
                        None,
                    ));
                    Type::Error
                }
            }
            Expr::MacroCall { macro_name, arguments, span } => {
                if self.macro_expansion_depth > 10 {
                    self.diagnostics.push(Diagnostic::new(
                        "Macro expansion depth limit exceeded (Safe Harbor limit: 10)".to_string(),
                        "MER0140".to_string(),
                        *span,
                        DiagnosticCategory::Semantic,
                        None,
                    ));
                    return Type::Error;
                }

                if let Some(macro_def) = self.macros.get(macro_name).cloned() {
                    if let Some(def_span) = self.index.definitions.get(macro_name) {
                        self.index.usages.insert(*span, *def_span);
                    }
                    if macro_def.parameters.len() != arguments.len() {
                        self.diagnostics.push(Diagnostic::new(
                            format!("Macro `{}` expects {} arguments, found {}", macro_name, macro_def.parameters.len(), arguments.len()),
                            "MER0141".to_string(),
                            *span,
                            DiagnosticCategory::Semantic,
                            None,
                        ));
                        return Type::Error;
                    }

                    let mut args_map = std::collections::HashMap::new();
                    for (param, arg) in macro_def.parameters.iter().zip(arguments.iter()) {
                        args_map.insert(param.name.clone(), arg.clone());
                    }

                    let expanded_expr = macro_def.body.substitute(&args_map);
                    
                    self.macro_expansion_depth += 1;
                    let result_type = self.analyze_expression(&expanded_expr);
                    self.macro_expansion_depth -= 1;
                    
                    result_type
                } else {
                    self.diagnostics.push(Diagnostic::new(
                        format!("Unknown macro: {}", macro_name),
                        "MER0142".to_string(),
                        *span,
                        DiagnosticCategory::Semantic,
                        None,
                    ));
                    Type::Error
                }
            }
            Expr::MethodCall { object, method_name: _, arguments, span: _ } => {
                self.analyze_expression(object);
                for arg in arguments {
                    self.analyze_expression(arg);
                }
                Type::Unknown
            }
            Expr::Index { object, index, span: _ } => {
                self.analyze_expression(object);
                self.analyze_expression(index);
                Type::Unknown
            }
            Expr::Error(_) => {
                Type::Error
            },
            Expr::UnsafeBlock { statements, span: _ } => {
                let prev_unsafe = self.in_unsafe_block;
                self.in_unsafe_block = true;
                self.enter_scope();
                
                let mut block_type = Type::Unit;
                for (i, stmt) in statements.iter().enumerate() {
                    if i == statements.len() - 1 {
                        if let Stmt::Expr(e) = stmt {
                            block_type = self.analyze_expression(e);
                        } else {
                            self.analyze_statement(stmt);
                        }
                    } else {
                        self.analyze_statement(stmt);
                    }
                }
                
                self.exit_scope();
                self.in_unsafe_block = prev_unsafe;
                block_type
            }
        }
    }
}
