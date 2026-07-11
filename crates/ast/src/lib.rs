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
        fields: Vec<Parameter>,
        span: Span,
    },
    EnumDef {
        name: String,
        variants: Vec<(String, Vec<Type>)>,
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
                operator: operator.clone(),
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
            Expr::StructInit { name, fields, span } => Expr::StructInit {
                name: name.clone(),
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
            Expr::EnumInit { enum_name, variant_name, values, span } => Expr::EnumInit {
                enum_name: enum_name.clone(),
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
            Stmt::Function { name, parameters, return_type, is_async, body, span, doc_comment, attributes } => Stmt::Function {
                name: name.clone(),
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
