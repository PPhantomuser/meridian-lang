use meridian_ast::{BinaryOperator, Expr, Program, Stmt, Type};

pub struct Formatter {
    indent_level: usize,
    output: String,
}

impl Formatter {
    pub fn new() -> Self {
        Self {
            indent_level: 0,
            output: String::new(),
        }
    }

    pub fn format_program(mut self, program: &Program) -> String {
        for (i, stmt) in program.statements.iter().enumerate() {
            self.format_stmt(stmt);
            self.output.push('\n');
            // Add extra newline between top-level statements for spacing, except imports
            if i < program.statements.len() - 1 && !matches!(stmt, Stmt::Import(_, _)) {
                self.output.push('\n');
            }
        }
        self.output
    }

    fn push_indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str("    "); // 4 spaces
        }
    }

    fn format_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Import(path, _) => {
                self.push_indent();
                self.output.push_str(&format!("import \"{}\";", path));
            }
            Stmt::Let { name, mutable, type_annotation, initializer, .. } => {
                self.push_indent();
                self.output.push_str("let ");
                if *mutable {
                    self.output.push_str("mut ");
                }
                self.output.push_str(name);
                if let Some(ty) = type_annotation {
                    self.output.push_str(": ");
                    self.format_type(ty);
                }
                self.output.push_str(" = ");
                self.format_expr(initializer);
                self.output.push(';');
            }
            Stmt::Print(expr, _) => {
                self.push_indent();
                self.output.push_str("print ");
                self.format_expr(expr);
                self.output.push(';');
            }
            Stmt::Expr(expr) => {
                self.push_indent();
                self.format_expr(expr);
                self.output.push(';');
            }
            Stmt::Function { name, parameters, return_type, is_async, body, .. } => {
                self.push_indent();
                if *is_async {
                    self.output.push_str("async ");
                }
                self.output.push_str(&format!("fn {}(", name));
                for (i, param) in parameters.iter().enumerate() {
                    self.output.push_str(&param.name);
                    self.output.push_str(": ");
                    self.format_type(&param.ty);
                    if i < parameters.len() - 1 {
                        self.output.push_str(", ");
                    }
                }
                self.output.push_str(") -> ");
                self.format_type(return_type);
                self.output.push_str(" ");
                self.format_expr(body);
            }
            Stmt::While { condition, body, .. } => {
                self.push_indent();
                self.output.push_str("while ");
                self.format_expr(condition);
                self.output.push_str(" ");
                self.format_expr(body);
            }
            Stmt::For { iterator, iterable, body, .. } => {
                self.push_indent();
                self.output.push_str(&format!("for {} in ", iterator));
                self.format_expr(iterable);
                self.output.push_str(" ");
                self.format_expr(body);
            }
            Stmt::Break(_) => {
                self.push_indent();
                self.output.push_str("break;");
            }
            Stmt::Continue(_) => {
                self.push_indent();
                self.output.push_str("continue;");
            }
            Stmt::MacroDef { name, parameters, body, .. } => {
                self.push_indent();
                self.output.push_str(&format!("macro {}(", name));
                for (i, param) in parameters.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(&param.name);
                    self.output.push_str(": ");
                    self.format_type(&param.ty);
                }
                self.output.push_str(") ");
                self.format_expr(body);
            }
            Stmt::ExternBlock { functions, .. } => {
                self.push_indent();
                self.output.push_str("extern \"C\" {\n");
                self.indent_level += 1;
                for func in functions {
                    self.format_stmt(func);
                    self.output.push('\n');
                }
                self.indent_level -= 1;
                self.push_indent();
                self.output.push('}');
            }
            Stmt::StructDef { name, fields, .. } => {
                self.push_indent();
                self.output.push_str(&format!("struct {} {{\n", name));
                self.indent_level += 1;
                for field in fields {
                    self.push_indent();
                    self.output.push_str(&field.name);
                    self.output.push_str(": ");
                    self.format_type(&field.ty);
                    self.output.push_str(",\n");
                }
                self.indent_level -= 1;
                self.push_indent();
                self.output.push_str("}");
            }
            Stmt::EnumDef { name, variants, .. } => {
                self.push_indent();
                self.output.push_str(&format!("enum {} {{\n", name));
                self.indent_level += 1;
                for (variant_name, variant_type) in variants {
                    self.push_indent();
                    self.output.push_str(variant_name);
                    if let Some(ty) = variant_type {
                        self.output.push_str("(");
                        self.format_type(ty);
                        self.output.push_str(")");
                    }
                    self.output.push_str(",\n");
                }
                self.indent_level -= 1;
                self.push_indent();
                self.output.push_str("}");
            }
        }
    }

    fn format_type(&mut self, ty: &Type) {
        match ty {
            Type::Number => self.output.push_str("Number"),
            Type::Int => self.output.push_str("Int"),
            Type::String => self.output.push_str("String"),
            Type::Bool => self.output.push_str("Bool"),
            Type::Unit => self.output.push_str("()"),
            Type::Unknown => self.output.push_str("Unknown"),
            Type::Error => self.output.push_str("Error"),
            Type::Reference(inner, is_mut) => {
                self.output.push('&');
                if *is_mut {
                    self.output.push_str("mut ");
                }
                self.format_type(inner);
            }
            Type::Future(inner) => {
                self.output.push_str("Future<");
                self.format_type(inner);
                self.output.push('>');
            }
            Type::Generic(name, args) => {
                self.output.push_str(name);
                if !args.is_empty() {
                    self.output.push('<');
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.format_type(arg);
                    }
                    self.output.push('>');
                }
            }
            Type::Meta(name) => {
                self.output.push_str(name);
            }
            Type::RawPointer(inner, is_mut) => {
                self.output.push('*');
                if *is_mut {
                    self.output.push_str("mut ");
                } else {
                    self.output.push_str("const ");
                }
                self.format_type(inner);
            }
            Type::Native(name) => {
                self.output.push_str(name);
            }
            Type::Struct(name) => {
                self.output.push_str(name);
            }
            Type::Array(inner) => {
                self.output.push('[');
                self.format_type(inner);
                self.output.push(']');
            }
            Type::Enum(name) => {
                self.output.push_str(name);
            }
            Type::Option(inner) => {
                self.output.push_str("Option<");
                self.format_type(inner);
                self.output.push('>');
            }
            Type::Result(ok, err) => {
                self.output.push_str("Result<");
                self.format_type(ok);
                self.output.push_str(", ");
                self.format_type(err);
                self.output.push('>');
            }
        }
    }

    fn format_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Number(n, _) => self.output.push_str(&n.to_string()),
            Expr::Int(n, _) => self.output.push_str(&n.to_string()),
            Expr::String(s, _) => self.output.push_str(&format!("\"{}\"", s)),
            Expr::Bool(b, _) => self.output.push_str(if *b { "true" } else { "false" }),
            Expr::Identifier(name, _) => self.output.push_str(name),
            Expr::Binary { left, operator, right, .. } => {
                self.format_expr(left);
                self.output.push(' ');
                match operator {
                    BinaryOperator::Add => self.output.push('+'),
                    BinaryOperator::Subtract => self.output.push('-'),
                    BinaryOperator::Multiply => self.output.push('*'),
                    BinaryOperator::Divide => self.output.push('/'),
                    BinaryOperator::Equal => self.output.push_str("=="),
                    BinaryOperator::NotEqual => self.output.push_str("!="),
                    BinaryOperator::LessThan => self.output.push('<'),
                    BinaryOperator::LessThanEqual => self.output.push_str("<="),
                    BinaryOperator::GreaterThan => self.output.push('>'),
                    BinaryOperator::GreaterThanEqual => self.output.push_str(">="),
                }
                self.output.push(' ');
                self.format_expr(right);
            }
            Expr::Assign { target, value, .. } => {
                self.format_expr(target);
                self.output.push_str(" = ");
                self.format_expr(value);
            }
            Expr::Call { callee, arguments, .. } => {
                self.format_expr(callee);
                self.output.push('(');
                for (i, arg) in arguments.iter().enumerate() {
                    self.format_expr(arg);
                    if i < arguments.len() - 1 {
                        self.output.push_str(", ");
                    }
                }
                self.output.push(')');
            }
            Expr::MethodCall { object, method_name, arguments, .. } => {
                self.format_expr(object);
                self.output.push('.');
                self.output.push_str(method_name);
                self.output.push('(');
                for (i, arg) in arguments.iter().enumerate() {
                    self.format_expr(arg);
                    if i < arguments.len() - 1 {
                        self.output.push_str(", ");
                    }
                }
                self.output.push(')');
            }
            Expr::Index { object, index, .. } => {
                self.format_expr(object);
                self.output.push('[');
                self.format_expr(index);
                self.output.push(']');
            }
            Expr::FieldAccess { object, field_name, .. } => {
                self.format_expr(object);
                self.output.push('.');
                self.output.push_str(field_name);
            }
            Expr::FieldAssign { object, field_name, value, .. } => {
                self.format_expr(object);
                self.output.push('.');
                self.output.push_str(field_name);
                self.output.push_str(" = ");
                self.format_expr(value);
            }
            Expr::StructInit { name, fields, .. } => {
                self.output.push_str(name);
                self.output.push_str(" { ");
                for (i, (k, v)) in fields.iter().enumerate() {
                    self.output.push_str(k);
                    self.output.push_str(": ");
                    self.format_expr(v);
                    if i < fields.len() - 1 {
                        self.output.push_str(", ");
                    }
                }
                self.output.push_str(" }");
            }
            Expr::ArrayInit { elements, .. } => {
                self.output.push('[');
                for (i, e) in elements.iter().enumerate() {
                    self.format_expr(e);
                    if i < elements.len() - 1 {
                        self.output.push_str(", ");
                    }
                }
                self.output.push(']');
            }
            Expr::If { condition, then_branch, else_branch, .. } => {
                self.output.push_str("if ");
                self.format_expr(condition);
                self.output.push_str(" ");
                self.format_expr(then_branch);
                if let Some(else_branch) = else_branch {
                    self.output.push_str(" else ");
                    self.format_expr(else_branch);
                }
            }
            Expr::Block(statements, _) => {
                self.output.push_str("{\n");
                self.indent_level += 1;
                for stmt in statements {
                    self.format_stmt(stmt);
                    self.output.push('\n');
                }
                self.indent_level -= 1;
                self.push_indent();
                self.output.push('}');
            }
            Expr::Group(expr, _) => {
                self.output.push('(');
                self.format_expr(expr);
                self.output.push(')');
            }
            Expr::Range { start, end, inclusive, .. } => {
                self.format_expr(start);
                if *inclusive {
                    self.output.push_str("..=");
                } else {
                    self.output.push_str("..");
                }
                self.format_expr(end);
            }
            Expr::Borrow { expr, is_mut, .. } => {
                self.output.push('&');
                if *is_mut {
                    self.output.push_str("mut ");
                }
                self.format_expr(expr);
            }
            Expr::Dereference { expr, .. } => {
                self.output.push('*');
                self.format_expr(expr);
            }
            Expr::AsyncBlock { statements, .. } => {
                self.output.push_str("async {\n");
                self.indent_level += 1;
                for stmt in statements {
                    self.format_stmt(stmt);
                    self.output.push('\n');
                }
                self.indent_level -= 1;
                self.push_indent();
                self.output.push('}');
            }
            Expr::Await { expr, .. } => {
                self.format_expr(expr);
                self.output.push_str(".await");
            }
            Expr::Spawn { expr, .. } => {
                self.output.push_str("spawn ");
                self.format_expr(expr);
            }
            Expr::Error(_) => {
                self.output.push_str("/* error */");
            }

            Expr::UnsafeBlock { statements, .. } => {
                self.output.push_str("unsafe {\n");
                self.indent_level += 1;
                for stmt in statements {
                    self.format_stmt(stmt);
                    self.output.push('\n');
                }
                self.indent_level -= 1;
                self.push_indent();
                self.output.push('}');
            }
            Expr::MacroCall { macro_name, arguments, .. } => {
                self.output.push_str(macro_name);
                self.output.push_str("!(");
                for (i, arg) in arguments.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.format_expr(arg);
                }
                self.output.push(')');
            }
            Expr::Match { value, arms, .. } => {
                self.output.push_str("match ");
                self.format_expr(value);
                self.output.push_str(" {\n");
                self.indent_level += 1;
                for (pat, expr) in arms {
                    self.push_indent();
                    self.format_pattern(pat);
                    self.output.push_str(" => ");
                    self.format_expr(expr);
                    self.output.push_str(",\n");
                }
                self.indent_level -= 1;
                self.push_indent();
                self.output.push('}');
            }
            Expr::EnumInit { enum_name, variant_name, value, .. } => {
                self.output.push_str(enum_name);
                self.output.push_str("::");
                self.output.push_str(variant_name);
                if let Some(val) = value {
                    self.output.push('(');
                    self.format_expr(val);
                    self.output.push(')');
                }
            }
        }
    }

    fn format_pattern(&mut self, pat: &meridian_ast::Pattern) {
        match pat {
            meridian_ast::Pattern::CatchAll(_) => self.output.push('_'),
            meridian_ast::Pattern::Identifier(name, _) => self.output.push_str(name),
            meridian_ast::Pattern::EnumVariant { enum_name, variant_name, binding_name, .. } => {
                self.output.push_str(enum_name);
                self.output.push_str("::");
                self.output.push_str(variant_name);
                if let Some(b) = binding_name {
                    self.output.push('(');
                    self.output.push_str(b);
                    self.output.push(')');
                }
            }
            meridian_ast::Pattern::Number(n, _) => self.output.push_str(&n.to_string()),
            meridian_ast::Pattern::Int(n, _) => self.output.push_str(&n.to_string()),
            meridian_ast::Pattern::String(s, _) => self.output.push_str(&format!("\"{}\"", s)),
            meridian_ast::Pattern::Bool(b, _) => self.output.push_str(if *b { "true" } else { "false" }),
        }
    }
}
