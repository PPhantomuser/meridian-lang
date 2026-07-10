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

#[derive(Clone)]
#[allow(dead_code)]
struct StructSignature {
    fields: HashMap<String, Type>,
    span: Span,
}

#[derive(Clone)]
#[allow(dead_code)]
struct EnumSignature {
    variants: HashMap<String, Vec<Type>>,
    span: Span,
}

pub struct SemanticAnalyzer {
    pub diagnostics: Vec<Diagnostic>,
    scopes: Vec<HashMap<String, SymbolInfo>>,
    functions: HashMap<String, FunctionSignature>,
    macros: HashMap<String, MacroDefinition>,
    structs: HashMap<String, StructSignature>,
    enums: HashMap<String, EnumSignature>,
    current_function_return_type: Option<Type>,
    in_loop_depth: usize,
    macro_expansion_depth: usize,
    in_unsafe_block: bool,
    pub index: SemanticIndex,
    pub type_map: HashMap<Span, Type>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        let mut analyzer = Self {
            diagnostics: Vec::new(),
            scopes: vec![HashMap::new()],
            functions: HashMap::new(),
            macros: HashMap::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
            current_function_return_type: None,
            in_loop_depth: 0,
            macro_expansion_depth: 0,
            in_unsafe_block: false,
            index: SemanticIndex::default(),
            type_map: HashMap::new(),
        };
        // Register standard library
        let native_funcs = vec![
            ("print", vec![Type::Unknown], Type::Unit),
            ("read_file", vec![Type::String], Type::Result(Box::new(Type::String), Box::new(Type::String))),
            ("assert", vec![Type::Bool], Type::Unit),
            ("hashmap_new", vec![], Type::Native("HashMap".to_string())),
            ("hashmap_insert", vec![Type::Native("HashMap".to_string()), Type::String, Type::Unknown], Type::Unknown),
            ("hashmap_get", vec![Type::Native("HashMap".to_string()), Type::String], Type::Option(Box::new(Type::Unknown))),
            ("hashset_new", vec![], Type::Native("HashSet".to_string())),
            ("hashset_insert", vec![Type::Native("HashSet".to_string()), Type::String], Type::Unknown),
            ("hashset_contains", vec![Type::Native("HashSet".to_string()), Type::String], Type::Bool),
            ("vecdeque_new", vec![], Type::Native("VecDeque".to_string())),
            ("vecdeque_push_back", vec![Type::Native("VecDeque".to_string()), Type::Unknown], Type::Unknown),
            ("vecdeque_pop_front", vec![Type::Native("VecDeque".to_string())], Type::Option(Box::new(Type::Unknown))),
            ("tcp_bind", vec![Type::String], Type::Result(Box::new(Type::Native("TcpListener".to_string())), Box::new(Type::String))),
            ("tcp_accept", vec![Type::Native("TcpListener".to_string())], Type::Result(Box::new(Type::Native("TcpStream".to_string())), Box::new(Type::String))),
            ("tcp_read", vec![Type::Native("TcpStream".to_string())], Type::Result(Box::new(Type::String), Box::new(Type::String))),
            ("tcp_write", vec![Type::Native("TcpStream".to_string()), Type::String], Type::Result(Box::new(Type::Bool), Box::new(Type::String))),
            ("file_write", vec![Type::String, Type::String], Type::Result(Box::new(Type::Bool), Box::new(Type::String))),
            ("file_append", vec![Type::String, Type::String], Type::Result(Box::new(Type::Bool), Box::new(Type::String))),
            ("file_delete", vec![Type::String], Type::Result(Box::new(Type::Bool), Box::new(Type::String))),
            ("file_exists", vec![Type::String], Type::Bool),
            ("process_output", vec![Type::String, Type::Unknown], Type::Result(Box::new(Type::String), Box::new(Type::String))),
            ("dlopen", vec![Type::String], Type::Result(Box::new(Type::Native("Library".to_string())), Box::new(Type::String))),
            ("dlsym", vec![Type::Native("Library".to_string()), Type::String, Type::String], Type::Option(Box::new(Type::Native("Function".to_string())))),
            ("dlcall", vec![Type::Native("Function".to_string()), Type::Unknown], Type::Result(Box::new(Type::Unknown), Box::new(Type::String))),
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

    fn types_compatible(&self, expected: &Type, actual: &Type) -> bool {
        if expected == actual {
            return true;
        }
        if *expected == Type::Unknown || *actual == Type::Unknown || *actual == Type::Error {
            return true;
        }
        false
    }

    fn resolve_type(&self, ty: &mut Type) {
        match ty {
            Type::Struct(name) => {
                if self.enums.contains_key(name) {
                    *ty = Type::Enum(name.clone());
                }
            }
            Type::Reference(inner, _) => self.resolve_type(inner),
            Type::Future(inner) => self.resolve_type(inner),
            Type::Generic(_, type_args) => {
                for arg in type_args {
                    self.resolve_type(arg);
                }
            }
            Type::Option(inner) => self.resolve_type(inner),
            Type::Result(ok, err) => {
                self.resolve_type(ok);
                self.resolve_type(err);
            }
            Type::Array(inner) => self.resolve_type(inner),
            _ => {}
        }
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
            } else if let Stmt::StructDef { name, fields, span } = stmt {
                let mut field_map = HashMap::new();
                for param in fields {
                    field_map.insert(param.name.clone(), param.ty.clone());
                }
                self.structs.insert(name.clone(), StructSignature { fields: field_map, span: *span });
                self.index.definitions.insert(name.clone(), *span);
            } else if let Stmt::EnumDef { name, variants, span } = stmt {
                let mut var_map = HashMap::new();
                for (v_name, v_type) in variants {
                    var_map.insert(v_name.clone(), v_type.clone());
                }
                self.enums.insert(name.clone(), EnumSignature { variants: var_map, span: *span });
                self.index.definitions.insert(name.clone(), *span);
            }
        }

        // Pass 1.5: Resolve types (upgrade Struct to Enum if it's actually an Enum)
        let mut resolved_functions = self.functions.clone();
        for (_, sig) in resolved_functions.iter_mut() {
            for p in &mut sig.parameters {
                self.resolve_type(p);
            }
            self.resolve_type(&mut sig.return_type);
        }
        self.functions = resolved_functions;

        let mut resolved_structs = self.structs.clone();
        for (_, sig) in resolved_structs.iter_mut() {
            for (_, ty) in sig.fields.iter_mut() {
                self.resolve_type(ty);
            }
        }
        self.structs = resolved_structs;

        let mut resolved_enums = self.enums.clone();
        for (_, sig) in resolved_enums.iter_mut() {
            for (_, ty) in sig.variants.iter_mut() {
                for t in ty.iter_mut() {
                    self.resolve_type(t);
                }
            }
        }
        self.enums = resolved_enums;

        // Pass 2: Analyze everything
        for stmt in &program.statements {
            self.analyze_statement(stmt);
        }
    }

    fn analyze_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let { name, mutable, type_annotation, initializer, span } => {
                let init_type = self.analyze_expression(initializer);
                let mut resolved_type = type_annotation.clone();
                if let Some(t) = &mut resolved_type {
                    self.resolve_type(t);
                }
                let final_type = if let Some(annotated_type) = &resolved_type {
                    if !self.types_compatible(annotated_type, &init_type) {
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
                let mut resolved_parameters = parameters.clone();
                for param in &mut resolved_parameters {
                    self.resolve_type(&mut param.ty);
                }
                let mut resolved_return_type = return_type.clone();
                self.resolve_type(&mut resolved_return_type);

                let mut ref_param_count = 0;
                for param in &resolved_parameters {
                    if let Type::Reference(_, _) = param.ty {
                        ref_param_count += 1;
                    }
                }

                let returns_ref = matches!(resolved_return_type, Type::Reference(_, _));
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
                    parameters: resolved_parameters.iter().map(|p| p.ty.clone()).collect(),
                    return_type: resolved_return_type.clone(),
                    span: *span,
                    is_extern: false,
                };
                self.functions.insert(name.clone(), sig);

                self.enter_scope();
                for param in &resolved_parameters {
                    self.declare_variable(param.name.clone(), param.ty.clone(), false, param.span);
                }

                self.current_function_return_type = Some(resolved_return_type.clone());
                
                let actual_return_type = self.analyze_expression(body);
                
                let expected_ty = if *is_async {
                    if let Type::Future(inner) = &resolved_return_type {
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
                    resolved_return_type.clone()
                };

                if !self.types_compatible(&expected_ty, &actual_return_type) {
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
                if !self.types_compatible(&Type::Bool, &cond_ty) {
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
                let yielded_ty = if self.types_compatible(&Type::Number, &iter_ty) {
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
            Stmt::Import(_, _) => {},
            Stmt::StructDef { .. } => {},
            Stmt::EnumDef { .. } => {},
        }
    }

    fn analyze_expression(&mut self, expr: &Expr) -> Type {
        let ty = self.analyze_expression_inner(expr);
        self.type_map.insert(expr.span(), ty.clone());
        ty
    }

    fn analyze_expression_inner(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Number(_, _) => Type::Number,
            Expr::String(_, _) => Type::String,
            Expr::Bool(_, _) => Type::Bool,
            Expr::Identifier(name, span) => {
                if let Some(info) = self.lookup_variable(name) {
                    if info.borrows.contains(&BorrowKind::Exclusive) {
                        self.diagnostics.push(Diagnostic::new(
                            format!("Cannot move or access variable '{}' because it is mutably borrowed", name),
                            "MER0156".to_string(),
                            *span,
                            DiagnosticCategory::Semantic,
                            Some("Wait for the mutable borrow to end before accessing.".to_string()),
                        ));
                    }
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

                if !self.types_compatible(&left_ty, &right_ty) {
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
                        } else if left_ty == Type::Int {
                            Type::Int
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
                        if left_ty == Type::Number || left_ty == Type::Int {
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
                if !self.types_compatible(&Type::Bool, &cond_ty) {
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
                    if then_ty != Type::Error && else_ty != Type::Error && !self.types_compatible(&then_ty, &else_ty) {
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
                    if !self.types_compatible(&Type::Unit, &then_ty) {
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
                                Some("Make the variable mutable with 'let mut'.".to_string()),
                            ));
                        } else if !info.borrows.is_empty() {
                            self.diagnostics.push(Diagnostic::new(
                                format!("Cannot mutate variable '{}' because it is currently borrowed", name),
                                "MER0156".to_string(),
                                *span,
                                DiagnosticCategory::Semantic,
                                Some("Wait for the borrow to end before mutating.".to_string()),
                            ));
                        } else if !self.types_compatible(&info.ty, &val_ty) {
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
                } else if let Expr::Index { .. } = &**target {
                    let target_ty = self.analyze_expression(target);
                    if !self.types_compatible(&target_ty, &val_ty) {
                        self.diagnostics.push(Diagnostic::new(
                            format!("Type mismatch: cannot assign type {:?} to array element of type {:?}", val_ty, target_ty),
                            "MER0102".to_string(),
                            *span,
                            DiagnosticCategory::Type,
                            None,
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
                
                Type::Unit
            }
            Expr::Call { callee, arguments, span } => {
                if let Expr::Identifier(name, callee_span) = &**callee {
                    if let Some(signature) = self.functions.get(name).cloned() {
                        self.index.usages.insert(*callee_span, signature.span);
                        let unsafe_funcs = [
                            "process_output", "process_spawn", "tcp_bind", "tcp_accept", "tcp_read", "tcp_write",
                            "file_write", "file_append", "file_delete", "file_exists", "read_file",
                            "dlopen", "dlsym", "dlcall"
                        ];
                        
                        if (signature.is_extern || unsafe_funcs.contains(&name.as_str())) && !self.in_unsafe_block {
                            self.diagnostics.push(Diagnostic::new(
                                format!("Call to unsafe system function '{}' requires an unsafe block", name),
                                "MER0150".to_string(),
                                *span,
                                DiagnosticCategory::Semantic,
                                Some("Wrap the call in an unsafe { ... } block.".to_string()),
                            ));
                        }
                        
                        if self.macro_expansion_depth > 0 {
                            if unsafe_funcs.contains(&name.as_str()) {
                                self.diagnostics.push(Diagnostic::new(
                                    format!("Macro expansion resulted in a call to unsafe system function '{}'", name),
                                    "MER0155".to_string(),
                                    *span,
                                    DiagnosticCategory::Semantic,
                                    Some("Macros are sandboxed and cannot invoke file/net I/O".to_string()),
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
                                if !self.types_compatible(expected_ty, &arg_ty) {
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
                let last_type = Type::Unit;
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
            Expr::MethodCall { object, method_name, arguments, span } => {
                let obj_ty = self.analyze_expression(object);
                for arg in arguments {
                    self.analyze_expression(arg);
                }
                self.diagnostics.push(Diagnostic::new(
                    format!("Method calls are not yet supported in Meridian (called '{}' on type {:?})", method_name, obj_ty),
                    "MER0155".to_string(),
                    *span,
                    DiagnosticCategory::Semantic,
                    Some("Methods using 'impl' blocks are planned for a future release.".to_string()),
                ));
                Type::Error
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
            Expr::Int(_, _) => Type::Int,
            Expr::FieldAccess { object, field_name, span } => {
                let obj_ty = self.analyze_expression(object);
                if let Type::Struct(name) = &obj_ty {
                    if let Some(sig) = self.structs.get(name) {
                        if let Some(ty) = sig.fields.get(field_name) {
                            return ty.clone();
                        }
                        self.diagnostics.push(Diagnostic::new(
                            format!("Field '{}' not found in struct '{}'", field_name, name),
                            "MER0150".to_string(),
                            *span,
                            DiagnosticCategory::Type,
                            None,
                        ));
                    }
                } else if obj_ty != Type::Unknown && obj_ty != Type::Error {
                    self.diagnostics.push(Diagnostic::new(
                        format!("Cannot access field '{}' on non-struct type {:?}", field_name, obj_ty),
                        "MER0151".to_string(),
                        *span,
                        DiagnosticCategory::Type,
                        None,
                    ));
                }
                Type::Unknown
            }
            Expr::FieldAssign { object, field_name, value, span } => {
                let obj_ty = self.analyze_expression(object);
                let val_ty = self.analyze_expression(value);
                if let Type::Struct(name) = &obj_ty {
                    if let Some(sig) = self.structs.get(name) {
                        if let Some(expected_ty) = sig.fields.get(field_name) {
                            if expected_ty != &val_ty && val_ty != Type::Unknown && val_ty != Type::Error {
                                self.diagnostics.push(Diagnostic::new(
                                    format!("Type mismatch: expected {:?}, found {:?}", expected_ty, val_ty),
                                    "MER0152".to_string(),
                                    *span,
                                    DiagnosticCategory::Type,
                                    None,
                                ));
                            }
                            return val_ty;
                        }
                    }
                }
                Type::Unknown
            }
            Expr::StructInit { name, fields, span } => {
                if let Some(sig) = self.structs.get(name).cloned() {
                    for (f_name, f_val) in fields {
                        let val_ty = self.analyze_expression(f_val);
                        if let Some(expected_ty) = sig.fields.get(f_name) {
                            if !self.types_compatible(expected_ty, &val_ty) {
                                self.diagnostics.push(Diagnostic::new(
                                    format!("Type mismatch in field '{}': expected {:?}, found {:?}", f_name, expected_ty, val_ty),
                                    "MER0153".to_string(),
                                    *span,
                                    DiagnosticCategory::Type,
                                    None,
                                ));
                            }
                        } else {
                            self.diagnostics.push(Diagnostic::new(
                                format!("Field '{}' not found in struct '{}'", f_name, name),
                                "MER0154".to_string(),
                                *span,
                                DiagnosticCategory::Type,
                                None,
                            ));
                        }
                    }
                    Type::Struct(name.clone())
                } else {
                    self.diagnostics.push(Diagnostic::new(
                        format!("Struct '{}' not found", name),
                        "MER0155".to_string(),
                        *span,
                        DiagnosticCategory::Semantic,
                        None,
                    ));
                    Type::Error
                }
            }
            Expr::ArrayInit { elements, span } => {
                let mut elem_ty = Type::Unknown;
                for elem in elements {
                    let ty = self.analyze_expression(elem);
                    if elem_ty == Type::Unknown && ty != Type::Error {
                        elem_ty = ty;
                    } else if elem_ty != Type::Unknown && ty != Type::Error && ty != elem_ty {
                        self.diagnostics.push(Diagnostic::new(
                            format!("Array elements have inconsistent types: {:?} and {:?}", elem_ty, ty),
                            "MER0156".to_string(),
                            *span,
                            DiagnosticCategory::Type,
                            None,
                        ));
                    }
                }
                Type::Array(Box::new(elem_ty))
            }
            Expr::Index { object, index, span } => {
                let obj_ty = self.analyze_expression(object);
                let idx_ty = self.analyze_expression(index);
                if idx_ty != Type::Int && idx_ty != Type::Unknown && idx_ty != Type::Error {
                    self.diagnostics.push(Diagnostic::new(
                        format!("Array index must be of type Int, found {:?}", idx_ty),
                        "MER0157".to_string(),
                        *span,
                        DiagnosticCategory::Type,
                        None,
                    ));
                }
                if let Type::Array(inner) = obj_ty {
                    *inner
                } else {
                    Type::Unknown
                }
            }
            Expr::Match { value, arms, span: _ } => {
                let val_ty = self.analyze_expression(value);
                let mut return_type = Type::Unknown;
                for (pat, expr) in arms {
                    self.enter_scope();
                    self.validate_pattern(pat, &val_ty);
                    
                    let ty = self.analyze_expression(expr);
                    if return_type == Type::Unknown && ty != Type::Error {
                        return_type = ty;
                    } else if return_type != Type::Unknown && ty != Type::Error && return_type != ty {
                        self.diagnostics.push(Diagnostic::new(
                            format!("Match arms have incompatible types: {:?} and {:?}", return_type, ty),
                            "MER0159".to_string(),
                            expr.span(),
                            DiagnosticCategory::Type,
                            None,
                        ));
                    }
                    self.exit_scope();
                }
                return_type
            }
            Expr::EnumInit { enum_name, variant_name, values, span } => {
                if let Some(enum_sig) = self.enums.get(enum_name).cloned() {
                    if let Some(expected_types) = enum_sig.variants.get(variant_name) {
                        if values.len() != expected_types.len() {
                            self.diagnostics.push(Diagnostic::new(
                                format!("Enum variant '{}::{}' expects {} arguments, but found {}", enum_name, variant_name, expected_types.len(), values.len()),
                                "MER0160".to_string(),
                                *span,
                                DiagnosticCategory::Semantic,
                                None,
                            ));
                        }
                        for (i, v) in values.iter().enumerate() {
                            let actual_type = self.analyze_expression(v);
                            if i < expected_types.len() && !self.types_compatible(&expected_types[i], &actual_type) {
                                self.diagnostics.push(Diagnostic::new(
                                    format!("Type mismatch in enum variant: expected {:?}, found {:?}", expected_types[i], actual_type),
                                    "MER0161".to_string(),
                                    v.span(),
                                    DiagnosticCategory::Semantic,
                                    None,
                                ));
                            }
                        }
                    } else {
                        self.diagnostics.push(Diagnostic::new(
                            format!("Variant '{}' not found in enum '{}'", variant_name, enum_name),
                            "MER0163".to_string(),
                            *span,
                            DiagnosticCategory::Semantic,
                            None,
                        ));
                    }
                    return Type::Enum(enum_name.clone());
                } else if enum_name == "Option" {
                    if variant_name == "Some" {
                        if values.len() == 1 {
                            let val_ty = self.analyze_expression(&values[0]);
                            return Type::Option(Box::new(val_ty));
                        } else {
                            self.diagnostics.push(Diagnostic::new(
                                "Option::Some expects exactly 1 value".to_string(),
                                "MER0164".to_string(),
                                *span,
                                DiagnosticCategory::Type,
                                None,
                            ));
                            return Type::Unknown;
                        }
                    } else if variant_name == "None" {
                        if !values.is_empty() {
                            self.diagnostics.push(Diagnostic::new(
                                "Option::None does not take a value".to_string(),
                                "MER0165".to_string(),
                                *span,
                                DiagnosticCategory::Type,
                                None,
                            ));
                        }
                        return Type::Unknown;
                    }
                } else if enum_name == "Result" {
                    if variant_name == "Ok" {
                        if values.len() == 1 {
                            let val_ty = self.analyze_expression(&values[0]);
                            return Type::Result(Box::new(val_ty), Box::new(Type::Unknown));
                        } else {
                            self.diagnostics.push(Diagnostic::new(
                                "Result::Ok expects exactly 1 value".to_string(),
                                "MER0164".to_string(),
                                *span,
                                DiagnosticCategory::Type,
                                None,
                            ));
                            return Type::Unknown;
                        }
                    } else if variant_name == "Err" {
                        if values.len() == 1 {
                            let val_ty = self.analyze_expression(&values[0]);
                            return Type::Result(Box::new(Type::Unknown), Box::new(val_ty));
                        } else {
                            self.diagnostics.push(Diagnostic::new(
                                "Result::Err expects exactly 1 value".to_string(),
                                "MER0164".to_string(),
                                *span,
                                DiagnosticCategory::Type,
                                None,
                            ));
                            return Type::Unknown;
                        }
                    }
                }

                self.diagnostics.push(Diagnostic::new(
                    format!("Enum '{}' not found", enum_name),
                    "MER0166".to_string(),
                    *span,
                    DiagnosticCategory::Semantic,
                    None,
                ));
                Type::Error
            }
        }
    }

    fn validate_pattern(&mut self, pat: &meridian_ast::Pattern, ty: &Type) {
        use meridian_ast::Pattern;
        match pat {
            Pattern::CatchAll(_) => {},
            Pattern::Identifier(name, span) => {
                self.scopes.last_mut().unwrap().insert(name.clone(), SymbolInfo {
                    ty: ty.clone(),
                    mutable: false,
                    span: *span,
                    borrows: vec![],
                });
            },
            Pattern::Number(_, span) => {
                if *ty != Type::Number && *ty != Type::Unknown {
                    self.diagnostics.push(Diagnostic::new(
                        format!("Pattern expected type {:?}, found Number", ty),
                        "MER0167".to_string(),
                        *span,
                        DiagnosticCategory::Type,
                        None,
                    ));
                }
            },
            Pattern::Int(_, span) => {
                if *ty != Type::Int && *ty != Type::Unknown {
                    self.diagnostics.push(Diagnostic::new(
                        format!("Pattern expected type {:?}, found Int", ty),
                        "MER0169".to_string(),
                        *span,
                        DiagnosticCategory::Type,
                        None,
                    ));
                }
            },
            Pattern::String(_, span) => {
                if *ty != Type::String && *ty != Type::Unknown {
                    self.diagnostics.push(Diagnostic::new(
                        format!("Pattern expected type {:?}, found String", ty),
                        "MER0168".to_string(),
                        *span,
                        DiagnosticCategory::Type,
                        None,
                    ));
                }
            },
            Pattern::Bool(_, span) => {
                if *ty != Type::Bool && *ty != Type::Unknown {
                    self.diagnostics.push(Diagnostic::new(
                        format!("Pattern expected type {:?}, found Bool", ty),
                        "MER0170".to_string(),
                        *span,
                        DiagnosticCategory::Type,
                        None,
                    ));
                }
            },
            Pattern::EnumVariant { enum_name, variant_name, binding_names, span } => {
                // Check if the enum type matches the matched value's type
                let mut payload_tys: Vec<Type> = Vec::new();
                
                match ty {
                    Type::Enum(expected_enum) => {
                        if expected_enum != enum_name {
                            self.diagnostics.push(Diagnostic::new(
                                format!("Pattern expected enum '{}', found enum '{}'", expected_enum, enum_name),
                                "MER0171".to_string(),
                                *span,
                                DiagnosticCategory::Type,
                                None,
                            ));
                        } else if let Some(sig) = self.enums.get(enum_name) {
                            if let Some(v_tys) = sig.variants.get(variant_name) {
                                payload_tys = v_tys.clone();
                            }
                        }
                    }
                    Type::Option(inner) => {
                        if enum_name != "Option" {
                            self.diagnostics.push(Diagnostic::new(
                                format!("Pattern expected Option, found enum '{}'", enum_name),
                                "MER0171".to_string(),
                                *span,
                                DiagnosticCategory::Type,
                                None,
                            ));
                        } else if variant_name == "Some" {
                            payload_tys = vec![*inner.clone()];
                        } else if variant_name == "None" {
                            payload_tys = vec![];
                        }
                    }
                    Type::Result(ok_ty, err_ty) => {
                        if enum_name != "Result" {
                            self.diagnostics.push(Diagnostic::new(
                                format!("Pattern expected Result, found enum '{}'", enum_name),
                                "MER0171".to_string(),
                                *span,
                                DiagnosticCategory::Type,
                                None,
                            ));
                        } else if variant_name == "Ok" {
                            payload_tys = vec![*ok_ty.clone()];
                        } else if variant_name == "Err" {
                            payload_tys = vec![*err_ty.clone()];
                        }
                    }
                    Type::Unknown | Type::Error => {}
                    _ => {
                        self.diagnostics.push(Diagnostic::new(
                            format!("Pattern expected enum '{}', found {:?}", enum_name, ty),
                            "MER0172".to_string(),
                            *span,
                            DiagnosticCategory::Type,
                            None,
                        ));
                    }
                }
                
                if binding_names.len() > 0 && payload_tys.len() != binding_names.len() && !matches!(ty, Type::Unknown | Type::Error) {
                    self.diagnostics.push(Diagnostic::new(
                        format!("Pattern expected {} bindings, found {}", payload_tys.len(), binding_names.len()),
                        "MER0162".to_string(),
                        *span,
                        DiagnosticCategory::Type,
                        None,
                    ));
                }
                
                // If there are bindings, assign them the type of the inner payloads if any
                for (i, b_name) in binding_names.iter().enumerate() {
                    let b_ty = if i < payload_tys.len() { payload_tys[i].clone() } else { Type::Unknown };
                    self.scopes.last_mut().unwrap().insert(b_name.clone(), SymbolInfo {
                        ty: b_ty,
                        mutable: false,
                        span: *span,
                        borrows: vec![],
                    });
                }
            }
        }
    }
}
