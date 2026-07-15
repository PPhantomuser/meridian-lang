use meridian_diagnostics::Span;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Type {
    Number,
    Int,
    String,
    Bool,
    Unit,
    Reference(Box<Type>, bool), // (type, is_mut)
    Future(Box<Type>),
    Generic(String, Vec<Type>),
    Meta(String), // For macro metaparameters like "Expr"
    RawPointer(Box<Type>, bool), // (type, is_mut)
    Native(String), // Opaque native handles (e.g. "HashMap")
    Struct(String),
    Enum(String),
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
    Array(Box<Type>),
    Unknown,
    Error,
}

pub fn type_to_string(ty: &Type) -> String {
    match ty {
        Type::Number => "Number".to_string(),
        Type::Int => "Int".to_string(),
        Type::String => "String".to_string(),
        Type::Bool => "Bool".to_string(),
        Type::Unit => "Unit".to_string(),
        Type::Struct(name) => name.clone(),
        Type::Enum(name) => name.clone(),
        Type::Option(inner) => format!("Option_{}", type_to_string(inner)),
        Type::Result(ok, err) => format!("Result_{}_{}", type_to_string(ok), type_to_string(err)),
        Type::Array(inner) => format!("Array_{}", type_to_string(inner)),
        Type::Generic(name, args) => format!("{}_{}", name, args.iter().map(type_to_string).collect::<Vec<_>>().join("_")),
        _ => "Unknown".to_string(),
    }
}

impl Type {
    pub fn substitute_types(&self, args: &std::collections::HashMap<String, Type>) -> Type {
        match self {
            Type::Struct(name) => {
                if let Some(t) = args.get(name) {
                    t.clone()
                } else {
                    self.clone()
                }
            }
            Type::Reference(inner, is_mut) => Type::Reference(Box::new(inner.substitute_types(args)), *is_mut),
            Type::Future(inner) => Type::Future(Box::new(inner.substitute_types(args))),
            Type::Option(inner) => Type::Option(Box::new(inner.substitute_types(args))),
            Type::Result(ok, err) => Type::Result(Box::new(ok.substitute_types(args)), Box::new(err.substitute_types(args))),
            Type::Array(inner) => Type::Array(Box::new(inner.substitute_types(args))),
            Type::Generic(name, type_args) => {
                let new_args: Vec<Type> = type_args.iter().map(|a| a.substitute_types(args)).collect();
                Type::Generic(name.clone(), new_args)
            }
            Type::RawPointer(inner, is_mut) => Type::RawPointer(Box::new(inner.substitute_types(args)), *is_mut),
            _ => self.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeParam {
    pub name: String,
    pub bounds: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Stmt {
    Import(String, Span),
    Let {
        name: String,
        mutable: bool,
        type_annotation: Option<Type>,
        initializer: Expr,
        span: Span,
    },
    Expr(Expr),
    Print(Expr, Span),
    Function {
        name: String,
        type_params: Vec<TypeParam>,
        parameters: Vec<Parameter>,
        return_type: Type,
        is_async: bool,
        body: Expr, // Will typically be a Block
        span: Span,
        doc_comment: Option<String>,
        attributes: Vec<String>,
    },
    While {
        condition: Expr,
        body: Expr,
        span: Span,
    },
    For {
        iterator: String,
        iterable: Expr,
        body: Expr,
        span: Span,
    },
    Break(Span),
    Continue(Span),
    Return(Option<Expr>, Span),
    MacroDef {
        name: String,
        parameters: Vec<Parameter>,
        body: Expr, // Usually a Block
        span: Span,
    },
    ExternBlock {
        functions: Vec<Stmt>, // Stmt::Function definitions without bodies
        span: Span,
    },
    StructDef {
        name: String,
        type_params: Vec<TypeParam>,
        fields: Vec<Parameter>,
        span: Span,
    },
    EnumDef {
        name: String,
        type_params: Vec<TypeParam>,
        variants: Vec<(String, Vec<Type>)>,
        span: Span,
    },
    TraitDef {
        name: String,
        type_params: Vec<TypeParam>,
        methods: Vec<Stmt>, // Stmt::Function definitions (can have empty body)
        span: Span,
    },
    Impl {
        trait_name: Option<String>,
        target_name: String,
        type_params: Vec<TypeParam>,
        methods: Vec<Stmt>, // Stmt::Function
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pattern {
    CatchAll(Span),
    Identifier(String, Span),
    EnumVariant {
        enum_name: String,
        variant_name: String,
        binding_names: Vec<String>,
        span: Span,
    },
    Number(f64, Span),
    Int(i64, Span),
    String(String, Span),
    Bool(bool, Span),
}

impl Pattern {
    pub fn span(&self) -> Span {
        match self {
            Pattern::CatchAll(span) => *span,
            Pattern::Identifier(_, span) => *span,
            Pattern::EnumVariant { span, .. } => *span,
            Pattern::Number(_, span) => *span,
            Pattern::Int(_, span) => *span,
            Pattern::String(_, span) => *span,
            Pattern::Bool(_, span) => *span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    Number(f64, Span),
    Int(i64, Span),
    String(String, Span),
    Bool(bool, Span),
    Identifier(String, Span),
    Binary {
        left: Box<Expr>,
        operator: BinaryOperator,
        right: Box<Expr>,
        span: Span,
    },
    If {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Option<Box<Expr>>,
        span: Span,
    },
    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
        span: Span,
    },
    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
        span: Span,
    },
    MethodCall {
        object: Box<Expr>,
        method_name: String,
        arguments: Vec<Expr>,
        span: Span,
    },
    Try(Box<Expr>, Span),
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },
    FieldAccess {
        object: Box<Expr>,
        field_name: String,
        span: Span,
    },
    FieldAssign {
        object: Box<Expr>,
        field_name: String,
        value: Box<Expr>,
        span: Span,
    },
    StructInit {
        name: String,
        type_args: Vec<Type>,
        fields: Vec<(String, Expr)>,
        span: Span,
    },
    ArrayInit {
        elements: Vec<Expr>,
        span: Span,
    },
    Match {
        value: Box<Expr>,
        arms: Vec<(Pattern, Expr)>,
        span: Span,
    },
    EnumInit {
        enum_name: String,
        type_args: Vec<Type>,
        variant_name: String,
        values: Vec<Expr>,
        span: Span,
    },
    Block(Vec<Stmt>, Span),
    Group(Box<Expr>, Span),
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
        inclusive: bool,
        span: Span,
    },
    Borrow {
        expr: Box<Expr>,
        is_mut: bool,
        span: Span,
    },
    Dereference {
        expr: Box<Expr>,
        span: Span,
    },
    AsyncBlock {
        statements: Vec<Stmt>,
        span: Span,
    },
    Await {
        expr: Box<Expr>,
        span: Span,
    },
    MacroCall {
        macro_name: String,
        arguments: Vec<Expr>,
        span: Span,
    },
    Spawn {
        expr: Box<Expr>,
        span: Span,
    },
    UnsafeBlock {
        statements: Vec<Stmt>,
        span: Span,
    },
    Error(Span),
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Number(_, span) => *span,
            Expr::Int(_, span) => *span,
            Expr::String(_, span) => *span,
            Expr::Bool(_, span) => *span,
            Expr::Identifier(_, span) => *span,
            Expr::Binary { span, .. } => *span,
            Expr::If { span, .. } => *span,
            Expr::Assign { span, .. } => *span,
            Expr::Call { span, .. } => *span,
            Expr::MethodCall { span, .. } => *span,
            Expr::Try(_, span) => *span,
            Expr::Index { span, .. } => *span,
            Expr::FieldAccess { span, .. } => *span,
            Expr::FieldAssign { span, .. } => *span,
            Expr::StructInit { span, .. } => *span,
            Expr::ArrayInit { span, .. } => *span,
            Expr::Match { span, .. } => *span,
            Expr::EnumInit { span, .. } => *span,
            Expr::Block(_, span) => *span,
            Expr::Group(_, span) => *span,
            Expr::Range { span, .. } => *span,
            Expr::Borrow { span, .. } => *span,
            Expr::Dereference { span, .. } => *span,
            Expr::AsyncBlock { span, .. } => *span,
            Expr::Await { span, .. } => *span,
            Expr::MacroCall { span, .. } => *span,
            Expr::Spawn { span, .. } => *span,
            Expr::UnsafeBlock { span, .. } => *span,
            Expr::Error(span) => *span,
        }
    }

    pub fn substitute(&self, args: &std::collections::HashMap<String, Expr>) -> Expr {
        match self {
            Expr::Identifier(name, _) => {
                if let Some(arg) = args.get(name) {
                    arg.clone()
                } else {
                    self.clone()
                }
            }
            Expr::Binary { left, operator, right, span } => Expr::Binary {
                left: Box::new(left.substitute(args)),
                operator: *operator,
                right: Box::new(right.substitute(args)),
                span: *span,
            },
            Expr::If { condition, then_branch, else_branch, span } => Expr::If {
                condition: Box::new(condition.substitute(args)),
                then_branch: Box::new(then_branch.substitute(args)),
                else_branch: else_branch.as_ref().map(|e| Box::new(e.substitute(args))),
                span: *span,
            },
            Expr::Assign { target, value, span } => Expr::Assign {
                target: Box::new(target.substitute(args)),
                value: Box::new(value.substitute(args)),
                span: *span,
            },
            Expr::Call { callee, arguments, span } => Expr::Call {
                callee: Box::new(callee.substitute(args)),
                arguments: arguments.iter().map(|a| a.substitute(args)).collect(),
                span: *span,
            },
            Expr::MethodCall { object, method_name, arguments, span } => Expr::MethodCall {
                object: Box::new(object.substitute(args)),
                method_name: method_name.clone(),
                arguments: arguments.iter().map(|a| a.substitute(args)).collect(),
                span: *span,
            },
            Expr::Index { object, index, span } => Expr::Index {
                object: Box::new(object.substitute(args)),
                index: Box::new(index.substitute(args)),
                span: *span,
            },
            Expr::FieldAccess { object, field_name, span } => Expr::FieldAccess {
                object: Box::new(object.substitute(args)),
                field_name: field_name.clone(),
                span: *span,
            },
            Expr::FieldAssign { object, field_name, value, span } => Expr::FieldAssign {
                object: Box::new(object.substitute(args)),
                field_name: field_name.clone(),
                value: Box::new(value.substitute(args)),
                span: *span,
            },
            Expr::StructInit { name, type_args, fields, span } => Expr::StructInit {
                name: name.clone(),
                type_args: type_args.clone(),
                fields: fields.iter().map(|(k, v)| (k.clone(), v.substitute(args))).collect(),
                span: *span,
            },
            Expr::ArrayInit { elements, span } => Expr::ArrayInit {
                elements: elements.iter().map(|e| e.substitute(args)).collect(),
                span: *span,
            },
            Expr::Match { value, arms, span } => Expr::Match {
                value: Box::new(value.substitute(args)),
                arms: arms.iter().map(|(p, e)| (p.clone(), e.substitute(args))).collect(),
                span: *span,
            },
            Expr::EnumInit { enum_name, type_args, variant_name, values, span } => Expr::EnumInit {
                enum_name: enum_name.clone(),
                type_args: type_args.clone(),
                variant_name: variant_name.clone(),
                values: values.iter().map(|v| v.substitute(args)).collect(),
                span: *span,
            },
            Expr::Block(stmts, span) => Expr::Block(stmts.iter().map(|s| s.substitute(args)).collect(), *span),
            Expr::Group(inner, span) => Expr::Group(Box::new(inner.substitute(args)), *span),
            Expr::Range { start, end, inclusive, span } => Expr::Range {
                start: Box::new(start.substitute(args)),
                end: Box::new(end.substitute(args)),
                inclusive: *inclusive,
                span: *span,
            },
            Expr::Borrow { expr, is_mut, span } => Expr::Borrow {
                expr: Box::new(expr.substitute(args)),
                is_mut: *is_mut,
                span: *span,
            },
            Expr::Dereference { expr, span } => Expr::Dereference {
                expr: Box::new(expr.substitute(args)),
                span: *span,
            },
            Expr::AsyncBlock { statements, span } => Expr::AsyncBlock {
                statements: statements.iter().map(|s| s.substitute(args)).collect(),
                span: *span,
            },
            Expr::Await { expr, span } => Expr::Await {
                expr: Box::new(expr.substitute(args)),
                span: *span,
            },
            Expr::MacroCall { macro_name, arguments, span } => Expr::MacroCall {
                macro_name: macro_name.clone(),
                arguments: arguments.iter().map(|a| a.substitute(args)).collect(),
                span: *span,
            },
            Expr::Spawn { expr, span } => Expr::Spawn {
                expr: Box::new(expr.substitute(args)),
                span: *span,
            },
            Expr::UnsafeBlock { statements, span } => Expr::UnsafeBlock {
                statements: statements.iter().map(|s| s.substitute(args)).collect(),
                span: *span,
            },
            _ => self.clone(),
        }
    }
}

impl Stmt {
    pub fn substitute(&self, args: &std::collections::HashMap<String, Expr>) -> Stmt {
        match self {
            Stmt::Let { name, mutable, type_annotation, initializer, span } => Stmt::Let {
                name: name.clone(),
                mutable: *mutable,
                type_annotation: type_annotation.clone(),
                initializer: initializer.substitute(args),
                span: *span,
            },
            Stmt::Expr(expr) => Stmt::Expr(expr.substitute(args)),
            Stmt::Print(expr, span) => Stmt::Print(expr.substitute(args), *span),
            Stmt::Function { name, type_params, parameters, return_type, is_async, body, span, doc_comment, attributes } => Stmt::Function {
                name: name.clone(),
                type_params: type_params.clone(),
                parameters: parameters.clone(),
                return_type: return_type.clone(),
                is_async: *is_async,
                body: body.substitute(args),
                span: *span,
                doc_comment: doc_comment.clone(),
                attributes: attributes.clone(),
            },
            Stmt::While { condition, body, span } => Stmt::While {
                condition: condition.substitute(args),
                body: body.substitute(args),
                span: *span,
            },
            Stmt::For { iterator, iterable, body, span } => Stmt::For {
                iterator: iterator.clone(),
                iterable: iterable.substitute(args),
                body: body.substitute(args),
                span: *span,
            },
            Stmt::MacroDef { name, parameters, body, span } => Stmt::MacroDef {
                name: name.clone(),
                parameters: parameters.clone(),
                body: body.substitute(args),
                span: *span,
            },
            Stmt::ExternBlock { functions, span } => Stmt::ExternBlock {
                functions: functions.iter().map(|f| f.substitute(args)).collect(),
                span: *span,
            },
            Stmt::Return(expr_opt, span) => Stmt::Return(
                expr_opt.as_ref().map(|e| e.substitute(args)),
                *span,
            ),
            Stmt::Impl { trait_name, target_name, type_params, methods, span } => Stmt::Impl {
                trait_name: trait_name.clone(),
                target_name: target_name.clone(),
                type_params: type_params.clone(),
                methods: methods.iter().map(|m| m.substitute(args)).collect(),
                span: *span,
            },
            Stmt::TraitDef { name, type_params, methods, span } => Stmt::TraitDef {
                name: name.clone(),
                type_params: type_params.clone(),
                methods: methods.iter().map(|m| m.substitute(args)).collect(),
                span: *span,
            },
            _ => self.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    LessThan,
    LessThanEqual,
    GreaterThan,
    GreaterThanEqual,
}

impl Expr {
    pub fn monomorphize(&self, type_bindings: &std::collections::HashMap<String, Type>) -> Expr {
        match self {
            Expr::Call { callee, arguments, span } => {
                let mono_callee = callee.monomorphize(type_bindings);
                Expr::Call {
                    callee: Box::new(mono_callee),
                    arguments: arguments.iter().map(|a| a.monomorphize(type_bindings)).collect(),
                    span: *span,
                }
            },
            Expr::StructInit { name, type_args: _, fields, span } => {
                let mono_name = if let Some(Type::Struct(tname)) = type_bindings.get(name) {
                    tname.clone()
                } else if let Some(Type::Generic(n, args)) = type_bindings.get(name) {
                    format!("{}_{}", n, args.iter().map(type_to_string).collect::<Vec<_>>().join("_"))
                } else {
                    name.clone()
                };
                Expr::StructInit {
                    name: mono_name,
                    type_args: vec![],
                    fields: fields.iter().map(|(n, e)| (n.clone(), e.monomorphize(type_bindings))).collect(),
                    span: *span,
                }
            },
            Expr::EnumInit { enum_name, type_args: _, variant_name, values, span } => {
                let mono_name = if let Some(Type::Enum(tname)) = type_bindings.get(enum_name) {
                    tname.clone()
                } else if let Some(Type::Generic(n, args)) = type_bindings.get(enum_name) {
                    format!("{}_{}", n, args.iter().map(type_to_string).collect::<Vec<_>>().join("_"))
                } else {
                    enum_name.clone()
                };
                Expr::EnumInit {
                    enum_name: mono_name,
                    type_args: vec![],
                    variant_name: variant_name.clone(),
                    values: values.iter().map(|v| v.monomorphize(type_bindings)).collect(),
                    span: *span,
                }
            },
            Expr::Binary { left, operator, right, span } => Expr::Binary {
                left: Box::new(left.monomorphize(type_bindings)),
                operator: *operator,
                right: Box::new(right.monomorphize(type_bindings)),
                span: *span,
            },
            Expr::Match { value, arms, span } => Expr::Match {
                value: Box::new(value.monomorphize(type_bindings)),
                arms: arms.iter().map(|(p, e)| (p.clone(), e.monomorphize(type_bindings))).collect(),
                span: *span,
            },
            Expr::Block(stmts, span) => Expr::Block(stmts.iter().map(|s| s.monomorphize(type_bindings)).collect(), *span),
            Expr::Group(inner, span) => Expr::Group(Box::new(inner.monomorphize(type_bindings)), *span),
            Expr::Range { start, end, inclusive, span } => Expr::Range {
                start: Box::new(start.monomorphize(type_bindings)),
                end: Box::new(end.monomorphize(type_bindings)),
                inclusive: *inclusive,
                span: *span,
            },
            Expr::Borrow { expr, is_mut, span } => Expr::Borrow {
                expr: Box::new(expr.monomorphize(type_bindings)),
                is_mut: *is_mut,
                span: *span,
            },
            Expr::Dereference { expr, span } => Expr::Dereference {
                expr: Box::new(expr.monomorphize(type_bindings)),
                span: *span,
            },
            Expr::AsyncBlock { statements, span } => Expr::AsyncBlock {
                statements: statements.iter().map(|s| s.monomorphize(type_bindings)).collect(),
                span: *span,
            },
            Expr::Await { expr, span } => Expr::Await {
                expr: Box::new(expr.monomorphize(type_bindings)),
                span: *span,
            },
            Expr::MacroCall { macro_name, arguments, span } => Expr::MacroCall {
                macro_name: macro_name.clone(),
                arguments: arguments.iter().map(|a| a.monomorphize(type_bindings)).collect(),
                span: *span,
            },
            Expr::Spawn { expr, span } => Expr::Spawn {
                expr: Box::new(expr.monomorphize(type_bindings)),
                span: *span,
            },
            Expr::UnsafeBlock { statements, span } => Expr::UnsafeBlock {
                statements: statements.iter().map(|s| s.monomorphize(type_bindings)).collect(),
                span: *span,
            },
            Expr::Try(expr, span) => Expr::Try(Box::new(expr.monomorphize(type_bindings)), *span),
            Expr::Index { object, index, span } => Expr::Index {
                object: Box::new(object.monomorphize(type_bindings)),
                index: Box::new(index.monomorphize(type_bindings)),
                span: *span,
            },
            Expr::FieldAccess { object, field_name, span } => Expr::FieldAccess {
                object: Box::new(object.monomorphize(type_bindings)),
                field_name: field_name.clone(),
                span: *span,
            },
            Expr::FieldAssign { object, field_name, value, span } => Expr::FieldAssign {
                object: Box::new(object.monomorphize(type_bindings)),
                field_name: field_name.clone(),
                value: Box::new(value.monomorphize(type_bindings)),
                span: *span,
            },
            Expr::ArrayInit { elements, span } => Expr::ArrayInit {
                elements: elements.iter().map(|e| e.monomorphize(type_bindings)).collect(),
                span: *span,
            },
            Expr::MethodCall { object, method_name, arguments, span } => Expr::MethodCall {
                object: Box::new(object.monomorphize(type_bindings)),
                method_name: method_name.clone(),
                arguments: arguments.iter().map(|a| a.monomorphize(type_bindings)).collect(),
                span: *span,
            },
            _ => self.clone(),
        }
    }
}

impl Stmt {
    pub fn monomorphize(&self, type_bindings: &std::collections::HashMap<String, Type>) -> Stmt {
        match self {
            Stmt::Let { name, mutable, type_annotation, initializer, span } => Stmt::Let {
                name: name.clone(),
                mutable: *mutable,
                type_annotation: type_annotation.as_ref().map(|t| t.substitute_types(type_bindings)),
                initializer: initializer.monomorphize(type_bindings),
                span: *span,
            },
            Stmt::Expr(expr) => Stmt::Expr(expr.monomorphize(type_bindings)),
            Stmt::Print(expr, span) => Stmt::Print(expr.monomorphize(type_bindings), *span),
            Stmt::Function { name, type_params: _, parameters, return_type, is_async, body, span, doc_comment, attributes } => Stmt::Function {
                name: name.clone(), // will be renamed by caller if needed
                type_params: vec![], // Monomorphized functions have no generic parameters!
                parameters: parameters.iter().map(|p| Parameter {
                    name: p.name.clone(),
                    ty: p.ty.substitute_types(type_bindings),
                    span: p.span,
                }).collect(),
                return_type: return_type.substitute_types(type_bindings),
                is_async: *is_async,
                body: body.monomorphize(type_bindings),
                span: *span,
                doc_comment: doc_comment.clone(),
                attributes: attributes.clone(),
            },
            Stmt::While { condition, body, span } => Stmt::While {
                condition: condition.monomorphize(type_bindings),
                body: body.monomorphize(type_bindings),
                span: *span,
            },
            Stmt::For { iterator, iterable, body, span } => Stmt::For {
                iterator: iterator.clone(),
                iterable: iterable.monomorphize(type_bindings),
                body: body.monomorphize(type_bindings),
                span: *span,
            },
            Stmt::MacroDef { name, parameters, body, span } => Stmt::MacroDef {
                name: name.clone(),
                parameters: parameters.clone(),
                body: body.monomorphize(type_bindings),
                span: *span,
            },
            Stmt::StructDef { name, type_params: _, fields, span } => Stmt::StructDef {
                name: name.clone(),
                type_params: vec![],
                fields: fields.iter().map(|p| Parameter {
                    name: p.name.clone(),
                    ty: p.ty.substitute_types(type_bindings),
                    span: p.span,
                }).collect(),
                span: *span,
            },
            Stmt::EnumDef { name, type_params: _, variants, span } => Stmt::EnumDef {
                name: name.clone(),
                type_params: vec![],
                variants: variants.iter().map(|(n, ts)| (
                    n.clone(),
                    ts.iter().map(|t| t.substitute_types(type_bindings)).collect()
                )).collect(),
                span: *span,
            },
            Stmt::ExternBlock { functions, span } => Stmt::ExternBlock {
                functions: functions.iter().map(|f| f.monomorphize(type_bindings)).collect(),
                span: *span,
            },
            Stmt::Return(expr_opt, span) => Stmt::Return(
                expr_opt.as_ref().map(|e| e.monomorphize(type_bindings)),
                *span,
            ),
            Stmt::Impl { trait_name, target_name, type_params: _, methods, span } => Stmt::Impl {
                trait_name: trait_name.clone(),
                target_name: target_name.clone(), // will be renamed during monomorphization if generic
                type_params: vec![],
                methods: methods.iter().map(|m| m.monomorphize(type_bindings)).collect(),
                span: *span,
            },
            Stmt::TraitDef { name, type_params: _, methods, span } => Stmt::TraitDef {
                name: name.clone(),
                type_params: vec![],
                methods: methods.iter().map(|m| m.monomorphize(type_bindings)).collect(),
                span: *span,
            },
            _ => self.clone(),
        }
    }
}
