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
    type_params: Vec<meridian_ast::TypeParam>,
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
    type_params: Vec<meridian_ast::TypeParam>,
}

#[derive(Clone)]
#[allow(dead_code)]
struct EnumSignature {
    variants: HashMap<String, Vec<Type>>,
    span: Span,
    type_params: Vec<meridian_ast::TypeParam>,
}

#[derive(Clone)]
struct TraitSignature {
    methods: HashMap<String, FunctionSignature>,
    #[allow(dead_code)]
    span: Span,
    #[allow(dead_code)]
    type_params: Vec<meridian_ast::TypeParam>,
}

pub struct SemanticAnalyzer {
    pub diagnostics: Vec<Diagnostic>,
    scopes: Vec<HashMap<String, SymbolInfo>>,
    functions: HashMap<String, FunctionSignature>,
    macros: HashMap<String, MacroDefinition>,
    structs: HashMap<String, StructSignature>,
    enums: HashMap<String, EnumSignature>,
    traits: HashMap<String, TraitSignature>,
    methods: HashMap<String, HashMap<String, FunctionSignature>>,
    
    // For monomorphization
    generic_functions: HashMap<String, Stmt>,
    generic_structs: HashMap<String, Stmt>,
    generic_enums: HashMap<String, Stmt>,
    monomorphized_stmts: Vec<Stmt>,

    current_function_return_type: Option<Type>,
    current_impl_target: Option<String>,
    in_loop_depth: usize,
    macro_expansion_depth: usize,
    in_unsafe_block: bool,
    pub index: SemanticIndex,
    pub type_map: std::collections::HashMap<Span, Type>,
    pub resolved_names: std::collections::HashMap<Span, String>,
    pub capabilities: std::collections::HashSet<String>,
    pub auto_borrows: std::collections::HashSet<Span>,
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
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
            traits: HashMap::new(),
            methods: HashMap::new(),
            generic_functions: HashMap::new(),
            generic_structs: HashMap::new(),
            generic_enums: HashMap::new(),
            monomorphized_stmts: Vec::new(),
            current_function_return_type: None,
            current_impl_target: None,
            in_loop_depth: 0,
            macro_expansion_depth: 0,
            in_unsafe_block: false,
            index: SemanticIndex::default(),
            type_map: std::collections::HashMap::new(),
            resolved_names: std::collections::HashMap::new(),
            capabilities: std::collections::HashSet::new(),
            auto_borrows: std::collections::HashSet::new(),
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
            ("string_split", vec![Type::String, Type::String], Type::Array(Box::new(Type::String))),
            ("string_contains", vec![Type::String, Type::String], Type::Bool),
            ("string_substring", vec![Type::String, Type::Int, Type::Int], Type::String),
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
            type_params: vec![],
        });
    }

    fn types_compatible(&self, expected: &Type, actual: &Type) -> bool {
        if expected == actual {
            return true;
        }
        if *expected == Type::Unknown || *actual == Type::Unknown || *actual == Type::Error {
            return true;
        }
        match (expected, actual) {
            (Type::Result(ok1, err1), Type::Result(ok2, err2)) => {
                self.types_compatible(ok1, ok2) && self.types_compatible(err1, err2)
            }
            (Type::Option(inner1), Type::Option(inner2)) => {
                self.types_compatible(inner1, inner2)
            }
            (Type::Array(inner1), Type::Array(inner2)) => {
                self.types_compatible(inner1, inner2)
            }
            (Type::Reference(inner1, mut1), Type::Reference(inner2, mut2)) => {
                (*mut1 == *mut2 || (!*mut1 && *mut2)) && self.types_compatible(inner1, inner2)
            }
            _ => false,
        }
    }

    #[allow(clippy::collapsible_match)]
    fn resolve_type(&mut self, ty: &mut Type, span: Span) {
        if let Type::Struct(name) = ty {
            if name == "Self" {
                if let Some(target) = &self.current_impl_target {
                    *ty = Type::Struct(target.clone());
                    return;
                }
            }
        }
        match ty {
            Type::Struct(name) => {
                if self.enums.contains_key(name) {
                    *ty = Type::Enum(name.clone());
                } else if !self.structs.contains_key(name) && !matches!(name.as_str(), "HashMap" | "HashSet" | "VecDeque" | "TcpListener" | "TcpStream" | "Library" | "Function") {
                    self.diagnostics.push(Diagnostic::new(
                        format!("Unknown type '{}'", name),
                        "MER0100".to_string(),
                        span,
                        DiagnosticCategory::Semantic,
                        None,
                    ));
                    *ty = Type::Error;
                }
            }
            Type::Reference(inner, _) => self.resolve_type(inner, span),
            Type::Future(inner) => self.resolve_type(inner, span),
            Type::Generic(name, type_args) => {
                for arg in type_args.iter_mut() {
                    self.resolve_type(arg, span);
                }
                
                let type_args_strings: Vec<String> = type_args.iter().map(meridian_ast::type_to_string).collect();
                let mono_name = format!("{}_{}", name, type_args_strings.join("_"));
                self.resolved_names.insert(span, mono_name.clone());
                
                if self.structs.contains_key(&mono_name) {
                    *ty = Type::Struct(mono_name);
                } else if self.enums.contains_key(&mono_name) {
                    *ty = Type::Enum(mono_name);
                } else if let Some(generic_stmt) = self.generic_structs.get(name).cloned() {
                    if let Stmt::StructDef { type_params, fields: _, .. } = &generic_stmt {
                        if type_params.len() != type_args.len() {
                            self.diagnostics.push(Diagnostic::new(
                                format!("Struct '{}' expects {} type arguments, but {} were provided", name, type_params.len(), type_args.len()),
                                "MER0200".to_string(),
                                span,
                                DiagnosticCategory::Type,
                                None,
                            ));
                            *ty = Type::Error;
                            return;
                        }
                        
                        let mut type_bindings = HashMap::new();
                        for (i, param) in type_params.iter().enumerate() {
                                                        type_bindings.insert(param.name.clone(), type_args[i].clone());
                            for bound in &param.bounds {
                                let arg_name = meridian_ast::type_to_string(&type_args[i]);
                                let cap_str = format!("{} implements {}", arg_name, bound);
                                if !self.capabilities.contains(&cap_str) {
                                    self.diagnostics.push(Diagnostic::new(
                                        format!("Type '{}' does not implement required trait '{}'", arg_name, bound),
                                        "MER0107".to_string(),
                                        span,
                                        DiagnosticCategory::Type,
                                        None,
                                    ));
                                }
                            }
                        }
                        
                        let mut mono_stmt = generic_stmt.monomorphize(&type_bindings);
                        if let Stmt::StructDef { name: mono_stmt_name, fields, .. } = &mut mono_stmt {
                            *mono_stmt_name = mono_name.clone();
                            let mut field_map = HashMap::new();
                            for param in fields {
                                field_map.insert(param.name.clone(), param.ty.clone());
                            }
                            self.structs.insert(mono_name.clone(), StructSignature {
                                fields: field_map,
                                span,
                                type_params: vec![],
                            });
                        }
                        self.monomorphized_stmts.push(mono_stmt);
                        *ty = Type::Struct(mono_name);
                    }
                } else if let Some(generic_stmt) = self.generic_enums.get(name).cloned() {
                    if let Stmt::EnumDef { type_params, variants: _, .. } = &generic_stmt {
                        if type_params.len() != type_args.len() {
                            self.diagnostics.push(Diagnostic::new(
                                format!("Enum '{}' expects {} type arguments, but {} were provided", name, type_params.len(), type_args.len()),
                                "MER0200".to_string(),
                                span,
                                DiagnosticCategory::Type,
                                None,
                            ));
                            *ty = Type::Error;
                            return;
                        }
                        
                        let mut type_bindings = HashMap::new();
                        for (i, param) in type_params.iter().enumerate() {
                                                        type_bindings.insert(param.name.clone(), type_args[i].clone());
                            for bound in &param.bounds {
                                let arg_name = meridian_ast::type_to_string(&type_args[i]);
                                let cap_str = format!("{} implements {}", arg_name, bound);
                                if !self.capabilities.contains(&cap_str) {
                                    self.diagnostics.push(Diagnostic::new(
                                        format!("Type '{}' does not implement required trait '{}'", arg_name, bound),
                                        "MER0107".to_string(),
                                        span,
                                        DiagnosticCategory::Type,
                                        None,
                                    ));
                                }
                            }
                        }
                        
                        let mut mono_stmt = generic_stmt.monomorphize(&type_bindings);
                        if let Stmt::EnumDef { name: mono_stmt_name, variants, .. } = &mut mono_stmt {
                            *mono_stmt_name = mono_name.clone();
                            let mut var_map = HashMap::new();
                            for (v_name, v_type) in variants {
                                var_map.insert(v_name.clone(), v_type.clone());
                            }
                            self.enums.insert(mono_name.clone(), EnumSignature {
                                variants: var_map,
                                span,
                                type_params: vec![],
                            });
                        }
                        self.monomorphized_stmts.push(mono_stmt);
                        *ty = Type::Enum(mono_name);
                    }
                } else {
                    self.diagnostics.push(Diagnostic::new(
                        format!("Unknown generic type '{}'", name),
                        "MER0201".to_string(),
                        span,
                        DiagnosticCategory::Type,
                        None,
                    ));
                    *ty = Type::Error;
                }
            }
            Type::Option(inner) => self.resolve_type(inner, span),
            Type::Result(ok, err) => {
                self.resolve_type(ok, span);
                self.resolve_type(err, span);
            }
            Type::Array(inner) => self.resolve_type(inner, span),
            _ => {}
        }
    }

    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    #[allow(clippy::map_entry)]
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

    pub fn get_monomorphized_statements(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        std::mem::swap(&mut stmts, &mut self.monomorphized_stmts);
        stmts
    }

    pub fn analyze_program(&mut self, program: &Program) {
        // Pass 1: Hoist functions
        for stmt in &program.statements {
            if let Stmt::Function { name, type_params, parameters, return_type, span, .. } = stmt {
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
                        type_params: type_params.clone(),
                    });
                    self.index.definitions.insert(name.clone(), *span);
                    if !type_params.is_empty() {
                        self.generic_functions.insert(name.clone(), stmt.clone());
                    }
                }
            } else if let Stmt::ExternBlock { functions, .. } = stmt {
                for func in functions {
                    if let Stmt::Function { name, type_params, parameters, return_type, span, .. } = func {
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
                                type_params: type_params.clone(),
                            });
                            self.index.definitions.insert(name.clone(), *span);
                            if !type_params.is_empty() {
                                self.generic_functions.insert(name.clone(), func.clone());
                            }
                        }
                    }
                }
            } else if let Stmt::StructDef { name, type_params, fields, span } = stmt {
                let mut field_map = HashMap::new();
                for param in fields {
                    field_map.insert(param.name.clone(), param.ty.clone());
                }
                self.structs.insert(name.clone(), StructSignature { fields: field_map, span: *span, type_params: type_params.clone() });
                self.index.definitions.insert(name.clone(), *span);
                if !type_params.is_empty() {
                    self.generic_structs.insert(name.clone(), stmt.clone());
                }
            } else if let Stmt::EnumDef { name, type_params, variants, span } = stmt {
                let mut var_map = HashMap::new();
                for (v_name, v_type) in variants {
                    var_map.insert(v_name.clone(), v_type.clone());
                }
                self.enums.insert(name.clone(), EnumSignature { variants: var_map, span: *span, type_params: type_params.clone() });
                self.index.definitions.insert(name.clone(), *span);
                if !type_params.is_empty() {
                    self.generic_enums.insert(name.clone(), stmt.clone());
                }
            } else if let Stmt::TraitDef { name, type_params, methods, span } = stmt {
                let mut trait_methods = HashMap::new();
                for method in methods {
                    if let Stmt::Function { name: m_name, type_params: m_type_params, parameters: m_parameters, return_type: m_return_type, span: m_span, .. } = method {
                        let param_types = m_parameters.iter().map(|p| p.ty.clone()).collect();
                        trait_methods.insert(m_name.clone(), FunctionSignature {
                            parameters: param_types,
                            return_type: m_return_type.clone(),
                            span: *m_span,
                            is_extern: false,
                            type_params: m_type_params.clone(),
                        });
                    }
                }
                self.traits.insert(name.clone(), TraitSignature {
                    methods: trait_methods,
                    span: *span,
                    type_params: type_params.clone(),
                });
                self.index.definitions.insert(name.clone(), *span);
            } else if let Stmt::Impl { trait_name: _, target_name, type_params: _, methods, span: _ } = stmt {
                let target_methods = self.methods.entry(target_name.clone()).or_default();
                for method in methods {
                    if let Stmt::Function { name: m_name, type_params: m_type_params, parameters: m_parameters, return_type: m_return_type, span: m_span, .. } = method {
                        let mut param_types = Vec::new();
                        for p in m_parameters {
                            let mut ty = p.ty.clone();
                            // Expand `Self` to `target_name`
                            if let Type::Struct(n) = &ty {
                                if n == "Self" {
                                    ty = Type::Struct(target_name.clone());
                                }
                            } else if let Type::Reference(inner, is_mut) = &ty {
                                if let Type::Struct(n) = &**inner {
                                    if n == "Self" {
                                        ty = Type::Reference(Box::new(Type::Struct(target_name.clone())), *is_mut);
                                    }
                                }
                            }
                            param_types.push(ty);
                        }
                        
                        target_methods.insert(m_name.clone(), FunctionSignature {
                            parameters: param_types,
                            return_type: m_return_type.clone(),
                            span: *m_span,
                            is_extern: false,
                            type_params: m_type_params.clone(),
                        });
                        
                        // We also need to add them to `self.functions` as mangled names for generic monomorphization to work!
                        let mangled_name = format!("{}_{}", target_name, m_name);
                        self.functions.insert(mangled_name.clone(), FunctionSignature {
                            parameters: target_methods.get(m_name).unwrap().parameters.clone(),
                            return_type: m_return_type.clone(),
                            span: *m_span,
                            is_extern: false,
                            type_params: m_type_params.clone(),
                        });
                    }
                }
            }
        }

        // Pass 1.5: Resolve types (upgrade Struct to Enum if it's actually an Enum)
        let mut resolved_functions = self.functions.clone();
        for (_, sig) in resolved_functions.iter_mut() {
            if sig.type_params.is_empty() {
                let span = sig.span;
                for p in &mut sig.parameters {
                    self.resolve_type(p, span);
                }
                self.resolve_type(&mut sig.return_type, span);
            }
        }
        self.functions = resolved_functions;

        let mut resolved_structs = self.structs.clone();
        for (_, sig) in resolved_structs.iter_mut() {
            if sig.type_params.is_empty() {
                let span = sig.span;
                for (_, ty) in sig.fields.iter_mut() {
                    self.resolve_type(ty, span);
                }
            }
        }
        self.structs = resolved_structs;

        let mut resolved_enums = self.enums.clone();
        for (_, sig) in resolved_enums.iter_mut() {
            if sig.type_params.is_empty() {
                let span = sig.span;
                for (_, ty) in sig.variants.iter_mut() {
                    for t in ty.iter_mut() {
                        self.resolve_type(t, span);
                    }
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
                    self.resolve_type(t, *span);
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
            Stmt::Function { name, type_params, parameters, return_type, is_async, body, span, doc_comment: _, attributes: _ } => {
                if !type_params.is_empty() {
                    return;
                }
                
                let mut resolved_parameters = parameters.clone();
                for param in &mut resolved_parameters {
                    let p_span = param.span;
                    self.resolve_type(&mut param.ty, p_span);
                }
                let mut resolved_return_type = return_type.clone();
                self.resolve_type(&mut resolved_return_type, *span);

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
                    type_params: type_params.clone(),
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
            Stmt::Return(expr_opt, span) => {
                let actual_ty = match expr_opt {
                    Some(e) => self.analyze_expression(e),
                    None => Type::Unit,
                };

                if let Some(expected_ty) = self.current_function_return_type.clone() {
                    if !self.types_compatible(&expected_ty, &actual_ty) {
                        self.diagnostics.push(Diagnostic::new(
                            format!(
                                "Return type mismatch: expected `{:?}`, but found `{:?}`",
                                expected_ty, actual_ty
                            ),
                            "MER0117".to_string(),
                            *span,
                            DiagnosticCategory::Semantic,
                            None,
                        ));
                    }
                } else {
                    self.diagnostics.push(Diagnostic::new(
                        "'return' outside of function".to_string(),
                        "MER0118".to_string(),
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
            Stmt::TraitDef { .. } => {
                // Verified in Pass 1 and Impl blocks
            }
            Stmt::Impl { trait_name, target_name, type_params: _, methods, span } => {
                // If it implements a trait, verify that all trait methods are present and match signatures!
                if let Some(t_name) = trait_name {
                    if let Some(trait_sig) = self.traits.get(t_name).cloned() {
                        let target_methods = self.methods.get(target_name).cloned().unwrap_or_default();
                        for (req_name, req_sig) in trait_sig.methods.iter() {
                            if let Some(impl_sig) = target_methods.get(req_name) {
                                // Check if signatures match (ignoring exact self type references for now since they are expanded)
                                if req_sig.parameters.len() != impl_sig.parameters.len() {
                                    self.diagnostics.push(Diagnostic::new(
                                        format!("Method '{}' in trait '{}' expects {} parameters, but implementation has {}", req_name, t_name, req_sig.parameters.len(), impl_sig.parameters.len()),
                                        "MER0104".to_string(),
                                        *span,
                                        DiagnosticCategory::Semantic,
                                        None,
                                    ));
                                }
                                if req_sig.return_type != impl_sig.return_type {
                                    self.diagnostics.push(Diagnostic::new(
                                        format!("Method '{}' return type mismatch. Expected {:?}, found {:?}", req_name, req_sig.return_type, impl_sig.return_type),
                                        "MER0105".to_string(),
                                        *span,
                                        DiagnosticCategory::Semantic,
                                        None,
                                    ));
                                }
                            } else {
                                self.diagnostics.push(Diagnostic::new(
                                    format!("Missing implementation for trait method '{}'", req_name),
                                    "MER0103".to_string(),
                                    *span,
                                    DiagnosticCategory::Semantic,
                                    None,
                                ));
                            }
                        }
                    } else {
                        self.diagnostics.push(Diagnostic::new(
                            format!("Trait '{}' not found", t_name),
                            "MER0106".to_string(),
                            *span,
                            DiagnosticCategory::Semantic,
                            None,
                        ));
                    }
                    self.capabilities.insert(format!("{} implements {}", target_name, t_name));
                }

                let prev_impl = self.current_impl_target.clone();
                self.current_impl_target = Some(target_name.clone());
                for method in methods {
                    self.analyze_statement(method);
                }
                self.current_impl_target = prev_impl;
            },
        }
    }

    fn analyze_expression(&mut self, expr: &Expr) -> Type {
        let ty = self.analyze_expression_inner(expr);
        self.type_map.insert(expr.span(), ty.clone());
        ty
    }

    #[allow(clippy::collapsible_match)]
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
            Expr::Try(expr, span) => {
                let inner_ty = self.analyze_expression(expr);
                if let Type::Result(ok, err) = inner_ty {
                    if let Some(Type::Result(_, func_err)) = &self.current_function_return_type {
                        if !self.types_compatible(func_err, &err) {
                            self.diagnostics.push(Diagnostic::new(
                                format!("Try operator `?` error type mismatch: function returns `Result<_, {:?}>` but expression gives `Result<_, {:?}>`", func_err, err),
                                "MER0161".to_string(),
                                *span,
                                DiagnosticCategory::Type,
                                None,
                            ));
                        }
                    } else {
                        self.diagnostics.push(Diagnostic::new(
                            "Cannot use `?` operator in a function that does not return `Result`".to_string(),
                            "MER0162".to_string(),
                            *span,
                            DiagnosticCategory::Semantic,
                            Some("Change the function return type to `Result<T, E>`".to_string()),
                        ));
                    }
                    *ok
                } else if inner_ty != Type::Error {
                    self.diagnostics.push(Diagnostic::new(
                        format!("Try operator `?` can only be applied to `Result`, found `{:?}`", inner_ty),
                        "MER0163".to_string(),
                        *span,
                        DiagnosticCategory::Type,
                        None,
                    ));
                    Type::Error
                } else {
                    Type::Error
                }
            }
            Expr::Block(statements, _) => {
                self.enter_scope();
                let mut block_type = Type::Unit;
                let mut diverges = false;
                
                for (i, stmt) in statements.iter().enumerate() {
                    if matches!(stmt, Stmt::Return(_, _) | Stmt::Break(_) | Stmt::Continue(_)) {
                        diverges = true;
                    }
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
                if diverges {
                    Type::Unknown
                } else {
                    block_type
                }
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
                    let mut actual_name = name.clone();
                    
                    if let Some(generic_stmt) = self.generic_functions.get(name).cloned() {
                            if let Stmt::Function { ref type_params, ref parameters, .. } = generic_stmt {
                                let mut inferred_args = vec![Type::Unknown; type_params.len()];
                                let mut arg_types = Vec::new();
                                for arg in arguments {
                                    arg_types.push(self.analyze_expression(arg));
                                }
                                
                                for (i, param) in parameters.iter().enumerate() {
                                    if i < arg_types.len() {
                                        if let Type::Struct(t_name) = &param.ty {
                                            if let Some(pos) = type_params.iter().position(|p| p.name == *t_name) {
                                                if inferred_args[pos] == Type::Unknown {
                                                    inferred_args[pos] = arg_types[i].clone();
                                                }
                                            }
                                        }
                                    }
                                }
                                
                                
                                if inferred_args.iter().all(|t| *t != Type::Unknown) {
                                    let type_args_strings: Vec<String> = inferred_args.iter().map(meridian_ast::type_to_string).collect();
                                    let mono_name = format!("{}_{}", name, type_args_strings.join("_"));
                                    self.resolved_names.insert(*callee_span, mono_name.clone());

                                    if !self.functions.contains_key(&mono_name) {
                                        let mut type_bindings = std::collections::HashMap::new();
                                        for (j, param) in type_params.iter().enumerate() {
                                                                                        type_bindings.insert(param.name.clone(), inferred_args[j].clone());
                                            for bound in &param.bounds {
                                                let arg_name = meridian_ast::type_to_string(&inferred_args[j]);
                                                let cap_str = format!("{} implements {}", arg_name, bound);
                                                if !self.capabilities.contains(&cap_str) {
                                                    self.diagnostics.push(Diagnostic::new(
                                                        format!("Type '{}' does not implement required trait '{}'", arg_name, bound),
                                                        "MER0107".to_string(),
                                                        *span,
                                                        DiagnosticCategory::Type,
                                                        None,
                                                    ));
                                                }
                                            }
                                        }
                                        
                                        let mut mono_stmt = generic_stmt.monomorphize(&type_bindings);
                                        if let Stmt::Function { name: mono_stmt_name, parameters: mono_params, return_type: mono_ret, span, .. } = &mut mono_stmt {
                                            *mono_stmt_name = mono_name.clone();
                                            let param_types = mono_params.iter().map(|p| p.ty.clone()).collect();
                                            self.functions.insert(mono_name.clone(), FunctionSignature {
                                                parameters: param_types,
                                                return_type: mono_ret.clone(),
                                                span: *span,
                                                is_extern: false,
                                                type_params: vec![],
                                            });
                                        }
                                        self.monomorphized_stmts.push(mono_stmt);
                                    }
                                    actual_name = mono_name;
                                }
                            }
                    }

                    if let Some(signature) = self.functions.get(&actual_name).cloned() {
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
                        
                        // Capability checks
                        let needs_net = ["tcp_bind", "tcp_accept", "tcp_read", "tcp_write"];
                        let needs_fs = ["file_write", "file_append", "file_delete", "file_exists", "read_file"];
                        let needs_run = ["process_output", "process_spawn"];
                        let needs_ffi = ["dlopen", "dlsym", "dlcall"];
                        
                        let has_all = self.capabilities.contains("--allow-all");
                        
                        if needs_net.contains(&name.as_str()) && !has_all && !self.capabilities.contains("--allow-net") {
                            self.diagnostics.push(Diagnostic::new(
                                format!("Call to '{}' requires the --allow-net capability", name),
                                "MER0160".to_string(),
                                *span,
                                DiagnosticCategory::Semantic,
                                Some("Run the compiler/VM with the --allow-net flag.".to_string()),
                            ));
                        } else if needs_fs.contains(&name.as_str()) && !has_all && !self.capabilities.contains("--allow-fs") {
                            self.diagnostics.push(Diagnostic::new(
                                format!("Call to '{}' requires the --allow-fs capability", name),
                                "MER0160".to_string(),
                                *span,
                                DiagnosticCategory::Semantic,
                                Some("Run the compiler/VM with the --allow-fs flag.".to_string()),
                            ));
                        } else if needs_run.contains(&name.as_str()) && !has_all && !self.capabilities.contains("--allow-run") {
                            self.diagnostics.push(Diagnostic::new(
                                format!("Call to '{}' requires the --allow-run capability", name),
                                "MER0160".to_string(),
                                *span,
                                DiagnosticCategory::Semantic,
                                Some("Run the compiler/VM with the --allow-run flag.".to_string()),
                            ));
                        } else if needs_ffi.contains(&name.as_str()) && !has_all && !self.capabilities.contains("--allow-ffi") {
                            self.diagnostics.push(Diagnostic::new(
                                format!("Call to '{}' requires the --allow-ffi capability", name),
                                "MER0160".to_string(),
                                *span,
                                DiagnosticCategory::Semantic,
                                Some("Run the compiler/VM with the --allow-ffi flag.".to_string()),
                            ));
                        }
                        
                        if self.macro_expansion_depth > 0 && unsafe_funcs.contains(&name.as_str()) {
                                self.diagnostics.push(Diagnostic::new(
                                    format!("Macro expansion resulted in a call to unsafe system function '{}'", name),
                                    "MER0155".to_string(),
                                    *span,
                                    DiagnosticCategory::Semantic,
                                    Some("Macros are sandboxed and cannot invoke file/net I/O".to_string()),
                                ));
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
                let mut arg_types = Vec::new();
                for arg in arguments {
                    arg_types.push(self.analyze_expression(arg));
                }

                let obj_base_type_name = match &obj_ty {
                    Type::Struct(n) => Some(n.clone()),
                    Type::Reference(inner, _) => {
                        if let Type::Struct(n) = &**inner {
                            Some(n.clone())
                        } else {
                            None
                        }
                    },
                    _ => None,
                };

                if let Some(type_name) = obj_base_type_name {
                    // It could be a generic type like Box_Int, so we look it up!
                    // Wait, pass 1 registered methods under the generic target name (e.g., 'Box') or the monomorphized name?
                    // Methods were registered under `target_name` from the `impl Box<T>` -> target_name is `Box`.
                    // But wait, `obj_ty` might be `Box_Int`.
                    // So we strip everything after `_` if it's monomorphized?
                    // Actually, if we monomorphize the `impl` block, it will re-register under `Box_Int`! Wait, we didn't implement monomorphization for `impl` blocks yet. Let's just lookup directly and see if `self.methods` contains it.
                    let mut base_name = type_name.clone();
                    if !self.methods.contains_key(&base_name) {
                        if let Some(idx) = base_name.find('_') {
                            base_name = base_name[..idx].to_string();
                        }
                    }

                    if let Some(target_methods) = self.methods.get(&base_name).cloned() {
                        if let Some(signature) = target_methods.get(method_name).cloned() {
                            // Synthesize function call mangled name (which we know is `base_name_method_name`)
                            let mangled_name = format!("{}_{}", base_name, method_name);
                            // Store it in resolved_names so IR compiler knows what to call
                            self.resolved_names.insert(*span, mangled_name);

                            // We check the 'self' argument which is the first argument
                            let expected_self_ty = &signature.parameters[0];

                            let mut adjusted_obj_ty = obj_ty.clone();
                            if let Type::Reference(_, is_mut_expected) = expected_self_ty {
                                if !matches!(obj_ty, Type::Reference(_, _)) {
                                    adjusted_obj_ty = Type::Reference(Box::new(obj_ty.clone()), *is_mut_expected);
                                    self.auto_borrows.insert(object.span());
                                }
                            }

                            if !self.types_compatible(expected_self_ty, &adjusted_obj_ty) {
                                self.diagnostics.push(Diagnostic::new(
                                    format!("Type mismatch in 'self' argument: expected {:?}, found {:?}", expected_self_ty, obj_ty),
                                    "MER0102".to_string(),
                                    object.span(),
                                    DiagnosticCategory::Type,
                                    None,
                                ));
                            }

                            // Check other arguments
                            if arguments.len() + 1 != signature.parameters.len() {
                                self.diagnostics.push(Diagnostic::new(
                                    format!("Method '{}' expects {} arguments, but {} were provided", method_name, signature.parameters.len() - 1, arguments.len()),
                                    "MER0110".to_string(),
                                    *span,
                                    DiagnosticCategory::Semantic,
                                    None,
                                ));
                            } else {
                                for (i, arg_ty) in arg_types.iter().enumerate() {
                                    let expected_ty = &signature.parameters[i + 1];
                                    if !self.types_compatible(expected_ty, arg_ty) {
                                        self.diagnostics.push(Diagnostic::new(
                                            format!("Type mismatch in argument {}: expected {:?}, found {:?}", i + 1, expected_ty, arg_ty),
                                            "MER0102".to_string(),
                                            arguments[i].span(),
                                            DiagnosticCategory::Type,
                                            None,
                                        ));
                                    }
                                }
                            }
                            return signature.return_type;
                        } else {
                            self.diagnostics.push(Diagnostic::new(
                                format!("No method named '{}' found for type '{}'", method_name, base_name),
                                "MER0155".to_string(),
                                *span,
                                DiagnosticCategory::Semantic,
                                None,
                            ));
                        }
                    } else {
                        self.diagnostics.push(Diagnostic::new(
                            format!("No methods found for type '{}'", base_name),
                            "MER0155".to_string(),
                            *span,
                            DiagnosticCategory::Semantic,
                            None,
                        ));
                    }
                } else {
                    self.diagnostics.push(Diagnostic::new(
                        format!("Cannot call methods on type {:?}", obj_ty),
                        "MER0155".to_string(),
                        *span,
                        DiagnosticCategory::Semantic,
                        None,
                    ));
                }
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
                let mut obj_ty = self.analyze_expression(object);
                while let Type::Reference(inner, _) = obj_ty {
                    obj_ty = *inner;
                }
                
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
                let mut obj_ty = self.analyze_expression(object);
                while let Type::Reference(inner, _) = obj_ty {
                    obj_ty = *inner;
                }
                
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
            Expr::StructInit { name, type_args, fields, span } => {
                let mut field_types = std::collections::HashMap::new();
                for (f_name, f_val) in fields {
                    field_types.insert(f_name.clone(), self.analyze_expression(f_val));
                }

                let actual_name = if !type_args.is_empty() {
                    let mut dummy_ty = Type::Generic(name.clone(), type_args.clone());
                    self.resolve_type(&mut dummy_ty, *span);
                    if let Type::Struct(mono_name) = dummy_ty {
                        mono_name
                    } else {
                        name.clone()
                    }
                } else if let Some(generic_stmt) = self.generic_structs.get(name).cloned() {
                    if let Stmt::StructDef { type_params, fields: generic_fields, .. } = generic_stmt {
                        let mut inferred_args = vec![Type::Unknown; type_params.len()];
                        for g_field in generic_fields {
                            if let Some(f_type) = field_types.get(&g_field.name) {
                                if let Type::Struct(t_name) = &g_field.ty {
                                    if let Some(pos) = type_params.iter().position(|p| p.name == *t_name) {
                                        if inferred_args[pos] == Type::Unknown {
                                            inferred_args[pos] = f_type.clone();
                                        }
                                    }
                                }
                            }
                        }
                        if inferred_args.iter().all(|t| *t != Type::Unknown) {
                            let mut dummy_ty = Type::Generic(name.clone(), inferred_args);
                            self.resolve_type(&mut dummy_ty, *span);
                            if let Type::Struct(mono_name) = dummy_ty {
                                mono_name
                            } else {
                                name.clone()
                            }
                        } else {
                            name.clone()
                        }
                    } else {
                        name.clone()
                    }
                } else {
                    name.clone()
                };

                if let Some(sig) = self.structs.get(&actual_name).cloned() {
                    for (f_name, _) in fields {
                        let val_ty = field_types.get(f_name).unwrap();
                        if let Some(expected_ty) = sig.fields.get(f_name) {
                            if !self.types_compatible(expected_ty, val_ty) {
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
                    Type::Struct(actual_name)
                } else {
                    self.diagnostics.push(Diagnostic::new(
                        format!("Struct '{}' not found", actual_name),
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
            Expr::EnumInit { enum_name, type_args, variant_name, values, span } => {
                let actual_name = if !type_args.is_empty() {
                    let mut dummy_ty = Type::Generic(enum_name.clone(), type_args.clone());
                    self.resolve_type(&mut dummy_ty, *span);
                    if let Type::Enum(mono_name) = dummy_ty {
                        mono_name
                    } else {
                        enum_name.clone()
                    }
                } else if let Some(generic_stmt) = self.generic_enums.get(enum_name).cloned() {
                    if let Stmt::EnumDef { type_params, variants: generic_variants, .. } = generic_stmt {
                        let mut inferred_args = vec![Type::Unknown; type_params.len()];
                        if let Some(g_variant_types) = generic_variants.iter().find(|(n, _)| n == variant_name).map(|(_, t)| t) {
                            for (i, v_type) in g_variant_types.iter().enumerate() {
                                if i < values.len() {
                                    let arg_ty = self.analyze_expression(&values[i]);
                                    if let Type::Struct(t_name) = v_type {
                                        if let Some(pos) = type_params.iter().position(|p| p.name == *t_name) {
                                            if inferred_args[pos] == Type::Unknown {
                                                inferred_args[pos] = arg_ty.clone();
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if inferred_args.iter().all(|t| *t != Type::Unknown) {
                            let mut dummy_ty = Type::Generic(enum_name.clone(), inferred_args);
                            self.resolve_type(&mut dummy_ty, *span);
                            if let Type::Enum(mono_name) = dummy_ty {
                                mono_name
                            } else {
                                enum_name.clone()
                            }
                        } else {
                            enum_name.clone()
                        }
                    } else {
                        enum_name.clone()
                    }
                } else {
                    enum_name.clone()
                };

                if let Some(enum_sig) = self.enums.get(&actual_name).cloned() {
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
                        return Type::Enum(actual_name);
                    } else {
                        self.diagnostics.push(Diagnostic::new(
                            format!("Variant '{}' not found in enum '{}'", variant_name, actual_name),
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
                
                if !binding_names.is_empty() && payload_tys.len() != binding_names.len() && !matches!(ty, Type::Unknown | Type::Error) {
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
