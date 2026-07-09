use meridian_ast::{BinaryOperator, Expr, Parameter, Program, Stmt, Type};
use meridian_diagnostics::{Diagnostic, DiagnosticCategory, Span};
use meridian_lexer::{Lexer, Token, TokenKind};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
    peek_token: Token,
    pub diagnostics: Vec<Diagnostic>,
}

impl<'a> Parser<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Self {
        let current_token = lexer.next_token();
        let peek_token = lexer.next_token();
        Self {
            lexer,
            current_token,
            peek_token,
            diagnostics: Vec::new(),
        }
    }

    fn advance(&mut self) {
        self.current_token = std::mem::replace(&mut self.peek_token, self.lexer.next_token());
    }

    pub fn parse_program(&mut self) -> Program {
        let mut statements = Vec::new();
        while self.current_token.kind != TokenKind::EOF {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            } else {
                if self.current_token.kind == TokenKind::RBrace {
                    self.advance();
                } else {
                    self.sync_to_statement();
                }
            }
        }
        for d in &self.lexer.diagnostics {
            self.diagnostics.push(d.clone());
        }
        Program { statements }
    }

    fn sync_to_statement(&mut self) {
        while self.current_token.kind != TokenKind::EOF {
            match self.current_token.kind {
                TokenKind::Semicolon => {
                    self.advance();
                    return;
                }
                TokenKind::RBrace => {
                    return; // Don't consume RBrace, let block parser handle it
                }
                TokenKind::Let 
                | TokenKind::Fn 
                | TokenKind::If 
                | TokenKind::While 
                | TokenKind::For 
                | TokenKind::Print 
                | TokenKind::Break 
                | TokenKind::Continue 
                | TokenKind::Import 
                | TokenKind::Macro 
                | TokenKind::Extern 
                | TokenKind::Unsafe 
                | TokenKind::Async 
                | TokenKind::Spawn 
                | TokenKind::Struct => {
                    return;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn parse_statement(&mut self) -> Option<Stmt> {
        let mut doc_comment = None;
        if let TokenKind::DocComment(text) = &self.current_token.kind {
            doc_comment = Some(text.clone());
            self.advance();
        }

        let mut attributes = Vec::new();
        while self.current_token.kind == TokenKind::Hash {
            self.advance(); // consume #
            if self.current_token.kind == TokenKind::LBracket {
                self.advance(); // consume [
                if let TokenKind::Identifier(attr_name) = &self.current_token.kind {
                    attributes.push(attr_name.clone());
                    self.advance(); // consume ident
                    if self.current_token.kind == TokenKind::RBracket {
                        self.advance(); // consume ]
                    } else {
                        self.diagnostics.push(Diagnostic::new(
                            "Expected ']' after attribute".to_string(),
                            "MER0060".to_string(),
                            self.current_token.span,
                            DiagnosticCategory::Syntax,
                            None,
                        ));
                    }
                }
            }
        }

        match self.current_token.kind {
            TokenKind::Let => self.parse_let_statement(),
            TokenKind::Print => self.parse_print_statement(),
            TokenKind::Fn => self.parse_function_statement(doc_comment, attributes, false),
            TokenKind::Async => {
                if self.peek_token.kind == TokenKind::Fn {
                    self.parse_function_statement(doc_comment, attributes, true)
                } else {
                    self.parse_expression_statement()
                }
            }
            TokenKind::Import => self.parse_import_statement(),
            TokenKind::While => self.parse_while_statement(),
            TokenKind::For => self.parse_for_statement(),
            TokenKind::Break => self.parse_break_statement(),
            TokenKind::Continue => self.parse_continue_statement(),
            TokenKind::Macro => self.parse_macro_declaration(),
            TokenKind::Extern => self.parse_extern_block(),
            TokenKind::Struct => self.parse_struct_def(),
            TokenKind::Enum => self.parse_enum_def(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_struct_def(&mut self) -> Option<Stmt> {
        let start_span = self.current_token.span;
        self.advance(); // consume struct

        let name = match &self.current_token.kind {
            TokenKind::Identifier(name) => name.clone(),
            _ => {
                self.diagnostics.push(Diagnostic::new(
                    "Expected identifier after 'struct'".to_string(),
                    "MER0080".to_string(),
                    self.current_token.span,
                    DiagnosticCategory::Syntax,
                    None,
                ));
                return None;
            }
        };
        self.advance(); // consume name

        if self.current_token.kind != TokenKind::LBrace {
            self.diagnostics.push(Diagnostic::new(
                "Expected '{' after struct name".to_string(),
                "MER0081".to_string(),
                self.current_token.span,
                DiagnosticCategory::Syntax,
                None,
            ));
            return None;
        }
        self.advance(); // consume {

        let mut fields = Vec::new();
        while self.current_token.kind != TokenKind::RBrace && self.current_token.kind != TokenKind::EOF {
            let field_name = match &self.current_token.kind {
                TokenKind::Identifier(name) => name.clone(),
                _ => {
                    self.diagnostics.push(Diagnostic::new(
                        "Expected field name".to_string(),
                        "MER0082".to_string(),
                        self.current_token.span,
                        DiagnosticCategory::Syntax,
                        None,
                    ));
                    return None;
                }
            };
            let field_span = self.current_token.span;
            self.advance(); // consume field name

            if self.current_token.kind != TokenKind::Colon {
                self.diagnostics.push(Diagnostic::new(
                    "Expected ':' after field name".to_string(),
                    "MER0083".to_string(),
                    self.current_token.span,
                    DiagnosticCategory::Syntax,
                    None,
                ));
                return None;
            }
            self.advance(); // consume :

            let ty = self.parse_type_annotation()?;
            fields.push(Parameter {
                name: field_name,
                ty,
                span: field_span,
            });

            if self.current_token.kind == TokenKind::Comma {
                self.advance();
            } else if self.current_token.kind != TokenKind::RBrace {
                self.diagnostics.push(Diagnostic::new(
                    "Expected ',' or '}' after field".to_string(),
                    "MER0084".to_string(),
                    self.current_token.span,
                    DiagnosticCategory::Syntax,
                    None,
                ));
                return None;
            }
        }

        let end_span = self.current_token.span;
        if self.current_token.kind == TokenKind::RBrace {
            self.advance(); // consume }
        }

        Some(Stmt::StructDef {
            name,
            fields,
            span: Span::new(start_span.start, end_span.end),
        })
    }

    fn parse_enum_def(&mut self) -> Option<Stmt> {
        let start_span = self.current_token.span;
        self.advance(); // consume enum

        let name = match &self.current_token.kind {
            TokenKind::Identifier(name) => name.clone(),
            _ => {
                self.diagnostics.push(Diagnostic::new(
                    "Expected identifier after 'enum'".to_string(),
                    "MER0090".to_string(),
                    self.current_token.span,
                    DiagnosticCategory::Syntax,
                    None,
                ));
                return None;
            }
        };
        self.advance(); // consume name

        if self.current_token.kind != TokenKind::LBrace {
            self.diagnostics.push(Diagnostic::new(
                "Expected '{' after enum name".to_string(),
                "MER0091".to_string(),
                self.current_token.span,
                DiagnosticCategory::Syntax,
                None,
            ));
            return None;
        }
        self.advance(); // consume {

        let mut variants = Vec::new();
        while self.current_token.kind != TokenKind::RBrace && self.current_token.kind != TokenKind::EOF {
            let variant_name = match &self.current_token.kind {
                TokenKind::Identifier(name) => name.clone(),
                _ => {
                    self.diagnostics.push(Diagnostic::new(
                        "Expected enum variant name".to_string(),
                        "MER0092".to_string(),
                        self.current_token.span,
                        DiagnosticCategory::Syntax,
                        None,
                    ));
                    return None;
                }
            };
            self.advance(); // consume variant name

            let mut variant_type = None;
            if self.current_token.kind == TokenKind::LParen {
                self.advance(); // consume (
                variant_type = Some(self.parse_type_annotation()?);
                if self.current_token.kind != TokenKind::RParen {
                    self.diagnostics.push(Diagnostic::new(
                        "Expected ')' after enum variant type".to_string(),
                        "MER0093".to_string(),
                        self.current_token.span,
                        DiagnosticCategory::Syntax,
                        None,
                    ));
                    return None;
                }
                self.advance(); // consume )
            }

            variants.push((variant_name, variant_type));

            if self.current_token.kind == TokenKind::Comma {
                self.advance();
            } else if self.current_token.kind != TokenKind::RBrace {
                self.diagnostics.push(Diagnostic::new(
                    "Expected ',' or '}' after enum variant".to_string(),
                    "MER0094".to_string(),
                    self.current_token.span,
                    DiagnosticCategory::Syntax,
                    None,
                ));
                return None;
            }
        }

        let end_span = self.current_token.span;
        if self.current_token.kind == TokenKind::RBrace {
            self.advance(); // consume }
        }

        Some(Stmt::EnumDef {
            name,
            variants,
            span: Span::new(start_span.start, end_span.end),
        })
    }

    fn parse_import_statement(&mut self) -> Option<Stmt> {
        let start_span = self.current_token.span;
        self.advance(); // consume import

        let path = match &self.current_token.kind {
            TokenKind::String(path) => path.clone(),
            _ => {
                self.diagnostics.push(Diagnostic::new(
                    "Expected string literal after import".to_string(),
                    "MER0030".to_string(),
                    self.current_token.span,
                    DiagnosticCategory::Syntax,
                    None,
                ));
                return None;
            }
        };
        self.advance(); // consume string

        let mut end_span = self.current_token.span;
        if self.current_token.kind == TokenKind::Semicolon {
            end_span = self.current_token.span;
            self.advance();
        } else {
            self.diagnostics.push(Diagnostic::new(
                "Expected ';' after import statement".to_string(),
                "MER0031".to_string(),
                self.current_token.span,
                DiagnosticCategory::Syntax,
                None,
            ));
        }

        Some(Stmt::Import(path, Span::new(start_span.start, end_span.end)))
    }

    fn parse_extern_block(&mut self) -> Option<Stmt> {
        let start_span = self.current_token.span;
        self.advance(); // consume extern

        // Expect "C"
        match &self.current_token.kind {
            TokenKind::String(s) if s == "C" => {}
            _ => {
                self.diagnostics.push(Diagnostic::new(
                    "Expected \"C\" after extern".to_string(),
                    "MER0070".to_string(),
                    self.current_token.span,
                    DiagnosticCategory::Syntax,
                    None,
                ));
                return None;
            }
        }
        self.advance(); // consume "C"

        if self.current_token.kind != TokenKind::LBrace {
            self.diagnostics.push(Diagnostic::new(
                "Expected '{' after extern \"C\"".to_string(),
                "MER0071".to_string(),
                self.current_token.span,
                DiagnosticCategory::Syntax,
                None,
            ));
            return None;
        }
        self.advance(); // consume {

        let mut functions = Vec::new();
        while self.current_token.kind != TokenKind::RBrace && self.current_token.kind != TokenKind::EOF {
            if self.current_token.kind == TokenKind::Fn {
                if let Some(mut f) = self.parse_function_statement(None, vec![], false) {
                    if let Stmt::Function { body: Expr::Block(ref stmts, _), ref mut span, .. } = f {
                        if !stmts.is_empty() {
                            self.diagnostics.push(Diagnostic::new(
                                "Extern functions cannot have a body".to_string(),
                                "MER0072".to_string(),
                                *span,
                                DiagnosticCategory::Syntax,
                                None,
                            ));
                        }
                    }
                    functions.push(f);
                } else {
                    return None;
                }
            } else {
                self.diagnostics.push(Diagnostic::new(
                    "Expected function declaration in extern block".to_string(),
                    "MER0073".to_string(),
                    self.current_token.span,
                    DiagnosticCategory::Syntax,
                    None,
                ));
                return None;
            }
        }

        let end_span = self.current_token.span;
        if self.current_token.kind == TokenKind::RBrace {
            self.advance();
        }

        Some(Stmt::ExternBlock {
            functions,
            span: Span::new(start_span.start, end_span.end),
        })
    }

    fn parse_function_statement(&mut self, doc_comment: Option<String>, attributes: Vec<String>, is_async: bool) -> Option<Stmt> {
        let start_span = self.current_token.span;
        if is_async {
            self.advance(); // consume async
        }
        self.advance(); // consume fn

        let name = match &self.current_token.kind {
            TokenKind::Identifier(name) => name.clone(),
            _ => {
                self.diagnostics.push(Diagnostic::new(
                    "Expected function name".to_string(),
                    "MER0020".to_string(),
                    self.current_token.span,
                    DiagnosticCategory::Syntax,
                    None,
                ));
                return None;
            }
        };
        self.advance(); // consume name

        if self.current_token.kind != TokenKind::LParen {
            self.diagnostics.push(Diagnostic::new(
                "Expected '(' after function name".to_string(),
                "MER0021".to_string(),
                self.current_token.span,
                DiagnosticCategory::Syntax,
                None,
            ));
            return None;
        }
        self.advance(); // consume (

        let mut parameters = Vec::new();
        while self.current_token.kind != TokenKind::RParen && self.current_token.kind != TokenKind::EOF {
            let param_span_start = self.current_token.span.start;
            let param_name = match &self.current_token.kind {
                TokenKind::Identifier(name) => name.clone(),
                _ => {
                    self.diagnostics.push(Diagnostic::new(
                        "Expected parameter name".to_string(),
                        "MER0022".to_string(),
                        self.current_token.span,
                        DiagnosticCategory::Syntax,
                        None,
                    ));
                    return None;
                }
            };
            self.advance(); // consume param name

            if self.current_token.kind != TokenKind::Colon {
                self.diagnostics.push(Diagnostic::new(
                    "Expected ':' after parameter name".to_string(),
                    "MER0023".to_string(),
                    self.current_token.span,
                    DiagnosticCategory::Syntax,
                    None,
                ));
                return None;
            }
            self.advance(); // consume :

            let param_type = self.parse_type_annotation()?;
            
            parameters.push(Parameter {
                name: param_name,
                ty: param_type,
                span: Span::new(param_span_start, self.current_token.span.end),
            });

            if self.current_token.kind == TokenKind::Comma {
                self.advance();
            } else if self.current_token.kind != TokenKind::RParen {
                self.diagnostics.push(Diagnostic::new(
                    "Expected ',' or ')' in parameter list".to_string(),
                    "MER0024".to_string(),
                    self.current_token.span,
                    DiagnosticCategory::Syntax,
                    None,
                ));
                return None;
            }
        }

        if self.current_token.kind != TokenKind::RParen {
            self.diagnostics.push(Diagnostic::new(
                "Expected ')' after parameters".to_string(),
                "MER0025".to_string(),
                self.current_token.span,
                DiagnosticCategory::Syntax,
                None,
            ));
            return None;
        }
        self.advance(); // consume )

        let mut return_type = Type::Unit;
        if self.current_token.kind == TokenKind::Arrow {
            self.advance(); // consume ->
            return_type = self.parse_type_annotation()?;
        }

        let mut body = Expr::Block(vec![], Span::new(start_span.start, self.current_token.span.end)); // empty block for extern
        if self.current_token.kind == TokenKind::LBrace {
            body = self.parse_block_expression()?;
        } else if self.current_token.kind == TokenKind::Semicolon {
            self.advance(); // consume ; for extern function declarations
        } else {
            self.diagnostics.push(Diagnostic::new(
                "Expected '{' or ';' after function signature".to_string(),
                "MER0026".to_string(),
                self.current_token.span,
                DiagnosticCategory::Syntax,
                None,
            ));
            body = Expr::Error(self.current_token.span);
        }

        let end_span = body.span();

        Some(Stmt::Function {
            name,
            parameters,
            return_type,
            is_async,
            body,
            span: Span::new(start_span.start, end_span.end),
            doc_comment,
            attributes,
        })
    }

    fn parse_macro_declaration(&mut self) -> Option<Stmt> {
        let start_span = self.current_token.span;
        self.advance(); // consume macro

        let name = match &self.current_token.kind {
            TokenKind::Identifier(name) => name.clone(),
            _ => {
                self.diagnostics.push(Diagnostic::new(
                    "Expected macro name".to_string(),
                    "MER0050".to_string(),
                    self.current_token.span,
                    DiagnosticCategory::Syntax,
                    None,
                ));
                return None;
            }
        };
        self.advance(); // consume name

        if self.current_token.kind != TokenKind::LParen {
            self.diagnostics.push(Diagnostic::new(
                "Expected '(' after macro name".to_string(),
                "MER0051".to_string(),
                self.current_token.span,
                DiagnosticCategory::Syntax,
                None,
            ));
            return None;
        }
        self.advance(); // consume (

        let mut parameters = Vec::new();
        while self.current_token.kind != TokenKind::RParen && self.current_token.kind != TokenKind::EOF {
            let param_span_start = self.current_token.span.start;
            
            let mut is_dollar = false;
            if self.current_token.kind == TokenKind::Dollar {
                self.advance();
                is_dollar = true;
            }

            let param_name = match &self.current_token.kind {
                TokenKind::Identifier(name) => name.clone(),
                _ => {
                    self.diagnostics.push(Diagnostic::new(
                        "Expected parameter name".to_string(),
                        "MER0052".to_string(),
                        self.current_token.span,
                        DiagnosticCategory::Syntax,
                        None,
                    ));
                    return None;
                }
            };
            self.advance(); // consume param name

            if self.current_token.kind != TokenKind::Colon {
                self.diagnostics.push(Diagnostic::new(
                    "Expected ':' after parameter name".to_string(),
                    "MER0053".to_string(),
                    self.current_token.span,
                    DiagnosticCategory::Syntax,
                    None,
                ));
                return None;
            }
            self.advance(); // consume :

            let param_type = self.parse_type_annotation()?;
            
            let final_name = if is_dollar { format!("${}", param_name) } else { param_name };
            
            parameters.push(Parameter {
                name: final_name,
                ty: param_type,
                span: Span::new(param_span_start, self.current_token.span.end),
            });

            if self.current_token.kind == TokenKind::Comma {
                self.advance();
            } else if self.current_token.kind != TokenKind::RParen {
                self.diagnostics.push(Diagnostic::new(
                    "Expected ',' or ')' in parameter list".to_string(),
                    "MER0054".to_string(),
                    self.current_token.span,
                    DiagnosticCategory::Syntax,
                    None,
                ));
                return None;
            }
        }
        
        if self.current_token.kind != TokenKind::RParen {
            self.diagnostics.push(Diagnostic::new(
                "Expected ')' to close parameter list".to_string(),
                "MER0055".to_string(),
                self.current_token.span,
                DiagnosticCategory::Syntax,
                None,
            ));
            return None;
        }
        self.advance(); // consume )

        let body = self.parse_block_expression()?;
        
        Some(Stmt::MacroDef {
            name,
            parameters,
            body,
            span: Span::new(start_span.start, self.current_token.span.end),
        })
    }

    fn parse_while_statement(&mut self) -> Option<Stmt> {
        let start_span = self.current_token.span;
        self.advance(); // consume while

        let condition = self.parse_expression_impl(0, false)?;
        
        if self.current_token.kind != TokenKind::LBrace {
            self.diagnostics.push(Diagnostic::new(
                "Expected '{' after while condition".to_string(),
                "MER0032".to_string(),
                self.current_token.span,
                DiagnosticCategory::Syntax,
                None,
            ));
            return None;
        }

        let body = self.parse_block_expression()?;
        let end_span = body.span();

        Some(Stmt::While {
            condition,
            body,
            span: Span::new(start_span.start, end_span.end),
        })
    }

    fn parse_for_statement(&mut self) -> Option<Stmt> {
        let start_span = self.current_token.span;
        self.advance(); // consume for

        let iterator = match &self.current_token.kind {
            TokenKind::Identifier(name) => name.clone(),
            _ => {
                self.diagnostics.push(Diagnostic::new(
                    "Expected identifier in for loop".to_string(),
                    "MER0033".to_string(),
                    self.current_token.span,
                    DiagnosticCategory::Syntax,
                    None,
                ));
                return None;
            }
        };
        self.advance(); // consume identifier

        if self.current_token.kind != TokenKind::In {
            self.diagnostics.push(Diagnostic::new(
                "Expected 'in' in for loop".to_string(),
                "MER0034".to_string(),
                self.current_token.span,
                DiagnosticCategory::Syntax,
                None,
            ));
            return None;
        }
        self.advance(); // consume in

        let iterable = self.parse_expression_impl(0, false)?;

        if self.current_token.kind != TokenKind::LBrace {
            self.diagnostics.push(Diagnostic::new(
                "Expected '{' after for iterable".to_string(),
                "MER0035".to_string(),
                self.current_token.span,
                DiagnosticCategory::Syntax,
                None,
            ));
            return None;
        }

        let body = self.parse_block_expression()?;
        let end_span = body.span();

        Some(Stmt::For {
            iterator,
            iterable,
            body,
            span: Span::new(start_span.start, end_span.end),
        })
    }

    fn parse_break_statement(&mut self) -> Option<Stmt> {
        let start_span = self.current_token.span;
        self.advance(); // consume break
        
        let mut end_span = start_span;
        if self.current_token.kind == TokenKind::Semicolon {
            end_span = self.current_token.span;
            self.advance();
        }

        Some(Stmt::Break(Span::new(start_span.start, end_span.end)))
    }

    fn parse_continue_statement(&mut self) -> Option<Stmt> {
        let start_span = self.current_token.span;
        self.advance(); // consume continue

        let mut end_span = start_span;
        if self.current_token.kind == TokenKind::Semicolon {
            end_span = self.current_token.span;
            self.advance();
        }

        Some(Stmt::Continue(Span::new(start_span.start, end_span.end)))
    }

    fn parse_let_statement(&mut self) -> Option<Stmt> {
        let start_span = self.current_token.span;
        self.advance(); // consume let

        let mutable = if self.current_token.kind == TokenKind::Mut {
            self.advance();
            true
        } else {
            false
        };

        let name = match &self.current_token.kind {
            TokenKind::Identifier(name) => name.clone(),
            _ => {
                self.diagnostics.push(Diagnostic::new(
                    "Expected identifier after let".to_string(),
                    "MER0010".to_string(),
                    self.current_token.span,
                    DiagnosticCategory::Syntax,
                    None,
                ));
                return None;
            }
        };
        self.advance(); // consume identifier

        let mut type_annotation = None;
        if self.current_token.kind == TokenKind::Colon {
            self.advance(); // consume :
            type_annotation = Some(self.parse_type_annotation()?);
        }

        if self.current_token.kind != TokenKind::Equal {
            self.diagnostics.push(Diagnostic::new(
                "Expected '=' in let statement".to_string(),
                "MER0011".to_string(),
                self.current_token.span,
                DiagnosticCategory::Syntax,
                None,
            ));
            // Recover by assuming an Error expression
            return Some(Stmt::Let {
                name,
                mutable,
                type_annotation,
                initializer: Expr::Error(self.current_token.span),
                span: Span::new(start_span.start, self.current_token.span.end),
            });
        }
        self.advance(); // consume =

        let initializer = self.parse_expression(0).unwrap_or_else(|| {
            // Recover from missing expression
            Expr::Error(self.current_token.span)
        });

        let mut end_span = initializer.span();
        if self.current_token.kind == TokenKind::Semicolon {
            end_span = self.current_token.span;
            self.advance();
        }

        Some(Stmt::Let {
            name,
            mutable,
            type_annotation,
            initializer,
            span: Span::new(start_span.start, end_span.end),
        })
    }

    fn parse_type_annotation(&mut self) -> Option<Type> {
        if self.current_token.kind == TokenKind::Ampersand {
            self.advance(); // consume &
            let mut is_mut = false;
            if self.current_token.kind == TokenKind::Mut {
                is_mut = true;
                self.advance(); // consume mut
            }
            let inner_ty = self.parse_type_annotation()?;
            return Some(Type::Reference(Box::new(inner_ty), is_mut));
        }

        if self.current_token.kind == TokenKind::Star {
            self.advance(); // consume *
            let mut is_mut = false;
            if self.current_token.kind == TokenKind::Mut {
                is_mut = true;
                self.advance(); // consume mut
            } else if let TokenKind::Identifier(ref id) = self.current_token.kind {
                if id == "const" {
                    self.advance(); // consume const
                } else {
                    self.diagnostics.push(Diagnostic::new(
                        "Expected 'mut' or 'const' after '*'".to_string(),
                        "MER0074".to_string(),
                        self.current_token.span,
                        DiagnosticCategory::Syntax,
                        None,
                    ));
                    return None;
                }
            } else {
                self.diagnostics.push(Diagnostic::new(
                    "Expected 'mut' or 'const' after '*'".to_string(),
                    "MER0074".to_string(),
                    self.current_token.span,
                    DiagnosticCategory::Syntax,
                    None,
                ));
                return None;
            }
            let inner_ty = self.parse_type_annotation()?;
            return Some(Type::RawPointer(Box::new(inner_ty), is_mut));
        }

        let token_kind = self.current_token.kind.clone();
        match token_kind {
            TokenKind::Identifier(name) => {
                let ty = match name.as_str() {
                    "Number" => { self.advance(); Type::Number },
                    "Int" => { self.advance(); Type::Int },
                    "String" => { self.advance(); Type::String },
                    "Bool" => { self.advance(); Type::Bool },
                    "Unit" => { self.advance(); Type::Unit },
                    "Option" => {
                        self.advance(); // consume Option
                        if self.current_token.kind == TokenKind::LessThan {
                            self.advance(); // consume <
                            let inner = self.parse_type_annotation()?;
                            if self.current_token.kind == TokenKind::GreaterThan {
                                self.advance(); // consume >
                                return Some(Type::Option(Box::new(inner)));
                            } else {
                                self.diagnostics.push(Diagnostic::new(
                                    "Expected '>' after Option type".to_string(),
                                    "MER0042".to_string(),
                                    self.current_token.span,
                                    DiagnosticCategory::Syntax,
                                    None,
                                ));
                                return None;
                            }
                        } else {
                            self.diagnostics.push(Diagnostic::new(
                                "Expected '<' after Option type".to_string(),
                                "MER0043".to_string(),
                                self.current_token.span,
                                DiagnosticCategory::Syntax,
                                None,
                            ));
                            return None;
                        }
                    }
                    "Result" => {
                        self.advance(); // consume Result
                        if self.current_token.kind == TokenKind::LessThan {
                            self.advance(); // consume <
                            let ok = self.parse_type_annotation()?;
                            if self.current_token.kind != TokenKind::Comma {
                                self.diagnostics.push(Diagnostic::new(
                                    "Expected ',' in Result type".to_string(),
                                    "MER0044".to_string(),
                                    self.current_token.span,
                                    DiagnosticCategory::Syntax,
                                    None,
                                ));
                                return None;
                            }
                            self.advance(); // consume ,
                            let err = self.parse_type_annotation()?;
                            
                            if self.current_token.kind == TokenKind::GreaterThan {
                                self.advance(); // consume >
                                return Some(Type::Result(Box::new(ok), Box::new(err)));
                            } else {
                                self.diagnostics.push(Diagnostic::new(
                                    "Expected '>' after Result type".to_string(),
                                    "MER0045".to_string(),
                                    self.current_token.span,
                                    DiagnosticCategory::Syntax,
                                    None,
                                ));
                                return None;
                            }
                        } else {
                            self.diagnostics.push(Diagnostic::new(
                                "Expected '<' after Result type".to_string(),
                                "MER0046".to_string(),
                                self.current_token.span,
                                DiagnosticCategory::Syntax,
                                None,
                            ));
                            return None;
                        }
                    }
                    "Future" => {
                        self.advance(); // consume "Future"
                        if self.current_token.kind == TokenKind::LessThan {
                            self.advance(); // consume "<"
                            let inner = self.parse_type_annotation()?;
                            if self.current_token.kind == TokenKind::GreaterThan {
                                self.advance(); // consume ">"
                                return Some(Type::Future(Box::new(inner))); // RETURN EARLY to avoid trailing advance
                            } else {
                                self.diagnostics.push(Diagnostic::new(
                                    "Expected '>' after Future type".to_string(),
                                    "MER0040".to_string(),
                                    self.current_token.span,
                                    DiagnosticCategory::Syntax,
                                    None,
                                ));
                                return None;
                            }
                        } else {
                            self.diagnostics.push(Diagnostic::new(
                                "Expected '<' after Future type".to_string(),
                                "MER0041".to_string(),
                                self.current_token.span,
                                DiagnosticCategory::Syntax,
                                None,
                            ));
                            return None;
                        }
                    }
                    "Expr" | "Ident" | "Stmt" | "Block" => { self.advance(); Type::Meta(name.clone()) },
                    _ => {
                        // Check if it's a generic, like HashMap<K, V>
                        self.advance(); // consume ident
                        if self.current_token.kind == TokenKind::LessThan {
                            self.advance(); // consume <
                            let mut type_args = Vec::new();
                            if self.current_token.kind != TokenKind::GreaterThan {
                                loop {
                                    if let Some(t) = self.parse_type_annotation() {
                                        type_args.push(t);
                                    } else {
                                        return None;
                                    }
                                    if self.current_token.kind == TokenKind::Comma {
                                        self.advance();
                                    } else {
                                        break;
                                    }
                                }
                            }
                            if self.current_token.kind == TokenKind::GreaterThan {
                                self.advance(); // consume >
                                return Some(Type::Generic(name.clone(), type_args));
                            } else {
                                self.diagnostics.push(Diagnostic::new(
                                    "Expected '>' after generic arguments".to_string(),
                                    "MER0061".to_string(),
                                    self.current_token.span,
                                    DiagnosticCategory::Syntax,
                                    None,
                                ));
                                return None;
                            }
                        } else {
                            // Non-generic named type (struct)
                            return Some(Type::Struct(name.clone()));
                        }
                    }
                };
                return Some(ty);
            }
            TokenKind::LParen => {
                self.advance(); // consume (
                if self.current_token.kind == TokenKind::RParen {
                    self.advance(); // consume )
                    Some(Type::Unit)
                } else {
                    self.diagnostics.push(Diagnostic::new(
                        "Expected ')' after '(' for Unit type".to_string(),
                        "MER0037".to_string(),
                        self.current_token.span,
                        DiagnosticCategory::Syntax,
                        None,
                    ));
                    None
                }
            }
            TokenKind::LBracket => {
                self.advance(); // consume [
                let inner_ty = self.parse_type_annotation()?;
                if self.current_token.kind == TokenKind::RBracket {
                    self.advance(); // consume ]
                    Some(Type::Array(Box::new(inner_ty)))
                } else {
                    self.diagnostics.push(Diagnostic::new(
                        "Expected ']' after array type".to_string(),
                        "MER0038".to_string(),
                        self.current_token.span,
                        DiagnosticCategory::Syntax,
                        None,
                    ));
                    None
                }
            }
            _ => {
                self.diagnostics.push(Diagnostic::new(
                    "Expected type identifier".to_string(),
                    "MER0016".to_string(),
                    self.current_token.span,
                    DiagnosticCategory::Syntax,
                    None,
                ));
                None
            }
        }
    }

    fn parse_print_statement(&mut self) -> Option<Stmt> {
        let start_span = self.current_token.span;
        self.advance(); // consume print
        let expr = self.parse_expression(0)?;
        
        let mut end_span = expr.span();
        if self.current_token.kind == TokenKind::Semicolon {
            end_span = self.current_token.span;
            self.advance();
        }

        Some(Stmt::Print(expr, Span::new(start_span.start, end_span.end)))
    }

    fn parse_expression_statement(&mut self) -> Option<Stmt> {
        let expr = self.parse_expression(0)?;
        if self.current_token.kind == TokenKind::Semicolon {
            self.advance();
        }
        Some(Stmt::Expr(expr))
    }

    fn parse_expression(&mut self, precedence: u8) -> Option<Expr> {
        self.parse_expression_impl(precedence, true)
    }

    fn parse_expression_impl(&mut self, precedence: u8, allow_struct: bool) -> Option<Expr> {
        let mut left = match self.current_token.kind {
            TokenKind::Ampersand => {
                let start_span = self.current_token.span;
                self.advance();
                let mut is_mut = false;
                if self.current_token.kind == TokenKind::Mut {
                    is_mut = true;
                    self.advance();
                }
                let expr = self.parse_expression(25)?;
                let end_span = expr.span();
                Expr::Borrow {
                    expr: Box::new(expr),
                    is_mut,
                    span: Span::new(start_span.start, end_span.end),
                }
            }
            TokenKind::Star => {
                let start_span = self.current_token.span;
                self.advance();
                let expr = self.parse_expression(25)?;
                let end_span = expr.span();
                Expr::Dereference {
                    expr: Box::new(expr),
                    span: Span::new(start_span.start, end_span.end),
                }
            }
            TokenKind::Spawn => {
                let start_span = self.current_token.span;
                self.advance();
                let expr = self.parse_expression(25)?;
                let end_span = expr.span();
                Expr::Spawn {
                    expr: Box::new(expr),
                    span: Span::new(start_span.start, end_span.end),
                }
            }
            TokenKind::Async => {
                let start_span = self.current_token.span;
                self.advance();
                if self.current_token.kind != TokenKind::LBrace {
                    self.diagnostics.push(Diagnostic::new(
                        "Expected '{' after 'async'".to_string(),
                        "MER0038".to_string(),
                        self.current_token.span,
                        DiagnosticCategory::Syntax,
                        None,
                    ));
                    return None;
                }
                let block = self.parse_block_expression()?;
                let end_span = block.span();
                if let Expr::Block(statements, _) = block {
                    Expr::AsyncBlock {
                        statements,
                        span: Span::new(start_span.start, end_span.end),
                    }
                } else {
                    return None;
                }
            }
            TokenKind::Unsafe => {
                let start_span = self.current_token.span;
                self.advance();
                if self.current_token.kind != TokenKind::LBrace {
                    self.diagnostics.push(Diagnostic::new(
                        "Expected '{' after 'unsafe'".to_string(),
                        "MER0075".to_string(),
                        self.current_token.span,
                        DiagnosticCategory::Syntax,
                        None,
                    ));
                    return None;
                }
                let block = self.parse_block_expression()?;
                let end_span = block.span();
                if let Expr::Block(statements, _) = block {
                    Expr::UnsafeBlock {
                        statements,
                        span: Span::new(start_span.start, end_span.end),
                    }
                } else {
                    return None;
                }
            }
            TokenKind::Number(n) => {
                let expr = Expr::Number(n, self.current_token.span);
                self.advance();
                expr
            }
            TokenKind::Int(n) => {
                let expr = Expr::Int(n, self.current_token.span);
                self.advance();
                expr
            }
            TokenKind::Match => {
                let expr = self.parse_match_expression()?;
                expr
            }
            TokenKind::String(ref s) => {
                let expr = Expr::String(s.clone(), self.current_token.span);
                self.advance();
                expr
            }
            TokenKind::True => {
                let expr = Expr::Bool(true, self.current_token.span);
                self.advance();
                expr
            }
            TokenKind::False => {
                let expr = Expr::Bool(false, self.current_token.span);
                self.advance();
                expr
            }
            TokenKind::Dollar => {
                let start_span = self.current_token.span;
                self.advance();
                if let TokenKind::Identifier(ref s) = self.current_token.kind {
                    let name = format!("${}", s);
                    let end_span = self.current_token.span;
                    self.advance();
                    Expr::Identifier(name, Span::new(start_span.start, end_span.end))
                } else {
                    self.diagnostics.push(Diagnostic::new(
                        "Expected identifier after '$'".to_string(),
                        "MER0059".to_string(),
                        self.current_token.span,
                        DiagnosticCategory::Syntax,
                        None,
                    ));
                    return None;
                }
            }
            TokenKind::Identifier(ref s) => {
                let name = s.clone();
                let start_span = self.current_token.span;
                self.advance();
                
                if self.current_token.kind == TokenKind::Bang {
                    self.advance(); // consume !
                    if self.current_token.kind != TokenKind::LParen {
                        self.diagnostics.push(Diagnostic::new(
                            "Expected '(' after macro name!".to_string(),
                            "MER0056".to_string(),
                            self.current_token.span,
                            DiagnosticCategory::Syntax,
                            None,
                        ));
                        return None;
                    }
                    self.advance(); // consume (
                    
                    let mut arguments = Vec::new();
                    while self.current_token.kind != TokenKind::RParen && self.current_token.kind != TokenKind::EOF {
                        arguments.push(self.parse_expression(0)?);
                        if self.current_token.kind == TokenKind::Comma {
                            self.advance();
                        } else if self.current_token.kind != TokenKind::RParen {
                            self.diagnostics.push(Diagnostic::new(
                                "Expected ',' or ')' in macro arguments".to_string(),
                                "MER0057".to_string(),
                                self.current_token.span,
                                DiagnosticCategory::Syntax,
                                None,
                            ));
                            return None;
                        }
                    }
                    
                    if self.current_token.kind != TokenKind::RParen {
                        self.diagnostics.push(Diagnostic::new(
                            "Expected ')' to close macro arguments".to_string(),
                            "MER0058".to_string(),
                            self.current_token.span,
                            DiagnosticCategory::Syntax,
                            None,
                        ));
                        return None;
                    }
                    let end_span = self.current_token.span;
                    self.advance(); // consume )
                    
                    Expr::MacroCall {
                        macro_name: name,
                        arguments,
                        span: Span::new(start_span.start, end_span.end),
                    }
                } else if self.current_token.kind == TokenKind::Colon && self.peek_token.kind == TokenKind::Colon {
                    self.advance(); // consume :
                    self.advance(); // consume :
                    
                    let variant_name = match &self.current_token.kind {
                        TokenKind::Identifier(n) => n.clone(),
                        _ => {
                            self.diagnostics.push(Diagnostic::new(
                                "Expected variant name after '::'".to_string(),
                                "MER0095".to_string(),
                                self.current_token.span,
                                DiagnosticCategory::Syntax,
                                None,
                            ));
                            return None;
                        }
                    };
                    self.advance(); // consume variant name
                    
                    let mut value = None;
                    if self.current_token.kind == TokenKind::LParen {
                        self.advance(); // consume (
                        value = Some(Box::new(self.parse_expression(0)?));
                        if self.current_token.kind != TokenKind::RParen {
                            self.diagnostics.push(Diagnostic::new(
                                "Expected ')' after enum variant value".to_string(),
                                "MER0096".to_string(),
                                self.current_token.span,
                                DiagnosticCategory::Syntax,
                                None,
                            ));
                            return None;
                        }
                        self.advance(); // consume )
                    }
                    let end_span = self.current_token.span; // well, previous token's span is better, but close enough
                    
                    Expr::EnumInit {
                        enum_name: name,
                        variant_name,
                        value,
                        span: Span::new(start_span.start, end_span.end),
                    }
                } else if allow_struct && self.current_token.kind == TokenKind::LBrace {
                    self.advance(); // consume {
                    let mut fields = Vec::new();
                    while self.current_token.kind != TokenKind::RBrace && self.current_token.kind != TokenKind::EOF {
                        let field_name = match &self.current_token.kind {
                            TokenKind::Identifier(n) => n.clone(),
                            _ => {
                                self.diagnostics.push(Diagnostic::new(
                                    "Expected field name".to_string(),
                                    "MER0085".to_string(),
                                    self.current_token.span,
                                    DiagnosticCategory::Syntax,
                                    None,
                                ));
                                return None;
                            }
                        };
                        self.advance(); // consume field name

                        if self.current_token.kind != TokenKind::Colon {
                            self.diagnostics.push(Diagnostic::new(
                                "Expected ':' after field name".to_string(),
                                "MER0086".to_string(),
                                self.current_token.span,
                                DiagnosticCategory::Syntax,
                                None,
                            ));
                            return None;
                        }
                        self.advance(); // consume :
                        
                        let value = self.parse_expression(0)?;
                        fields.push((field_name, value));

                        if self.current_token.kind == TokenKind::Comma {
                            self.advance();
                        } else if self.current_token.kind != TokenKind::RBrace {
                            self.diagnostics.push(Diagnostic::new(
                                "Expected ',' or '}' in struct initialization".to_string(),
                                "MER0087".to_string(),
                                self.current_token.span,
                                DiagnosticCategory::Syntax,
                                None,
                            ));
                            return None;
                        }
                    }
                    let end_span = self.current_token.span;
                    if self.current_token.kind == TokenKind::RBrace {
                        self.advance(); // consume }
                    }
                    Expr::StructInit {
                        name,
                        fields,
                        span: Span::new(start_span.start, end_span.end),
                    }
                } else {
                    Expr::Identifier(name, start_span)
                }
            }
            TokenKind::If => {
                self.parse_if_expression()?
            }
            TokenKind::LBrace => {
                self.parse_block_expression()?
            }
            TokenKind::LBracket => {
                let start_span = self.current_token.span;
                self.advance(); // consume [
                let mut elements = Vec::new();
                while self.current_token.kind != TokenKind::RBracket && self.current_token.kind != TokenKind::EOF {
                    elements.push(self.parse_expression(0)?);
                    if self.current_token.kind == TokenKind::Comma {
                        self.advance();
                    } else if self.current_token.kind != TokenKind::RBracket {
                        self.diagnostics.push(Diagnostic::new(
                            "Expected ',' or ']' in array initialization".to_string(),
                            "MER0088".to_string(),
                            self.current_token.span,
                            DiagnosticCategory::Syntax,
                            None,
                        ));
                        return None;
                    }
                }
                let end_span = self.current_token.span;
                if self.current_token.kind == TokenKind::RBracket {
                    self.advance(); // consume ]
                }
                Expr::ArrayInit {
                    elements,
                    span: Span::new(start_span.start, end_span.end),
                }
            }
            TokenKind::LParen => {
                let start_span = self.current_token.span;
                self.advance();
                let expr = self.parse_expression(0)?;
                if self.current_token.kind != TokenKind::RParen {
                    self.diagnostics.push(Diagnostic::new(
                        "Expected ')' after grouped expression".to_string(),
                        "MER0036".to_string(),
                        self.current_token.span,
                        DiagnosticCategory::Syntax,
                        None,
                    ));
                    return None;
                }
                let end_span = self.current_token.span;
                self.advance();
                Expr::Group(Box::new(expr), Span::new(start_span.start, end_span.end))
            }
            _ => {
                self.diagnostics.push(Diagnostic::new(
                    format!("Expected expression, found {:?}", self.current_token.kind),
                    "MER0012".to_string(),
                    self.current_token.span,
                    DiagnosticCategory::Syntax,
                    None,
                ));
                return None;
            }
        };

        while precedence < self.peek_precedence() {
            if self.current_token.kind == TokenKind::Equal {
                self.advance(); // consume =
                let right = self.parse_expression_impl(0, allow_struct)?;
                let span = Span::new(left.span().start, right.span().end);
                
                if let Expr::FieldAccess { object, field_name, .. } = left {
                    left = Expr::FieldAssign {
                        object,
                        field_name,
                        value: Box::new(right),
                        span,
                    };
                } else {
                    left = Expr::Assign {
                        target: Box::new(left),
                        value: Box::new(right),
                        span,
                    };
                }
            } else if self.current_token.kind == TokenKind::LParen {
                self.advance(); // consume (
                let mut arguments = Vec::new();
                while self.current_token.kind != TokenKind::RParen && self.current_token.kind != TokenKind::EOF {
                    arguments.push(self.parse_expression(0)?);
                    if self.current_token.kind == TokenKind::Comma {
                        self.advance();
                    } else if self.current_token.kind != TokenKind::RParen {
                        self.diagnostics.push(Diagnostic::new(
                            "Expected ',' or ')' in argument list".to_string(),
                            "MER0027".to_string(),
                            self.current_token.span,
                            DiagnosticCategory::Syntax,
                            None,
                        ));
                        return None;
                    }
                }
                
                if self.current_token.kind != TokenKind::RParen {
                    self.diagnostics.push(Diagnostic::new(
                        "Expected ')' after arguments".to_string(),
                        "MER0028".to_string(),
                        self.current_token.span,
                        DiagnosticCategory::Syntax,
                        None,
                    ));
                    return None;
                }
                let end_span = self.current_token.span;
                self.advance(); // consume )
                
                let span = Span::new(left.span().start, end_span.end);
                left = Expr::Call {
                    callee: Box::new(left),
                    arguments,
                    span,
                };
            } else if self.current_token.kind == TokenKind::Dot {
                if self.peek_token.kind == TokenKind::Await {
                    self.advance(); // consume .
                    let end_span = self.current_token.span;
                    self.advance(); // consume await
                    let span = Span::new(left.span().start, end_span.end);
                    left = Expr::Await {
                        expr: Box::new(left),
                        span,
                    };
                } else {
                    self.advance(); // consume .
                    if let TokenKind::Identifier(method_name) = &self.current_token.kind {
                        let method_name = method_name.clone();
                        self.advance(); // consume ident
                        if self.current_token.kind != TokenKind::LParen {
                            let span = Span::new(left.span().start, self.current_token.span.start);
                            left = Expr::FieldAccess {
                                object: Box::new(left),
                                field_name: method_name,
                                span,
                            };
                            continue;
                        }
                        self.advance(); // consume (
                        let mut arguments = Vec::new();
                        while self.current_token.kind != TokenKind::RParen && self.current_token.kind != TokenKind::EOF {
                            arguments.push(self.parse_expression(0)?);
                            if self.current_token.kind == TokenKind::Comma {
                                self.advance();
                            } else if self.current_token.kind != TokenKind::RParen {
                                self.diagnostics.push(Diagnostic::new(
                                    "Expected ',' or ')' in argument list".to_string(),
                                    "MER0027".to_string(),
                                    self.current_token.span,
                                    DiagnosticCategory::Syntax,
                                    None,
                                ));
                                return None;
                            }
                        }
                        if self.current_token.kind != TokenKind::RParen {
                            self.diagnostics.push(Diagnostic::new(
                                "Expected ')' after arguments".to_string(),
                                "MER0028".to_string(),
                                self.current_token.span,
                                DiagnosticCategory::Syntax,
                                None,
                            ));
                            return None;
                        }
                        let end_span = self.current_token.span;
                        self.advance(); // consume )
                        
                        let span = Span::new(left.span().start, end_span.end);
                        left = Expr::MethodCall {
                            object: Box::new(left),
                            method_name,
                            arguments,
                            span,
                        };
                    } else {
                        self.diagnostics.push(Diagnostic::new(
                            "Expected identifier after '.'".to_string(),
                            "MER0063".to_string(),
                            self.current_token.span,
                            DiagnosticCategory::Syntax,
                            None,
                        ));
                        return None;
                    }
                }
            } else if self.current_token.kind == TokenKind::LBracket {
                self.advance(); // consume [
                let index = self.parse_expression(0)?;
                if self.current_token.kind != TokenKind::RBracket {
                    self.diagnostics.push(Diagnostic::new(
                        "Expected ']' after index".to_string(),
                        "MER0064".to_string(),
                        self.current_token.span,
                        DiagnosticCategory::Syntax,
                        None,
                    ));
                    return None;
                }
                let end_span = self.current_token.span;
                self.advance(); // consume ]
                let span = Span::new(left.span().start, end_span.end);
                left = Expr::Index {
                    object: Box::new(left),
                    index: Box::new(index),
                    span,
                };
            } else if self.current_token.kind == TokenKind::DotDot || self.current_token.kind == TokenKind::DotDotEqual {
                let inclusive = self.current_token.kind == TokenKind::DotDotEqual;
                self.advance();
                let right = self.parse_expression(4)?; // Precedence of range operators
                let span = Span::new(left.span().start, right.span().end);
                left = Expr::Range {
                    start: Box::new(left),
                    end: Box::new(right),
                    inclusive,
                    span,
                };
            } else {
                let operator = self.parse_infix_operator(&self.current_token.kind)?;
                self.advance(); // consume operator
                
                let right = self.parse_expression(self.infix_precedence(&operator))?;
                
                let span = Span::new(left.span().start, right.span().end);
                left = Expr::Binary {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                    span,
                };
            }
        }

        Some(left)
    }

    fn parse_if_expression(&mut self) -> Option<Expr> {
        let start_span = self.current_token.span;
        self.advance(); // consume if

        let condition = self.parse_expression_impl(0, false)?;
        
        if self.current_token.kind != TokenKind::LBrace {
            self.diagnostics.push(Diagnostic::new(
                "Expected '{' after if condition".to_string(),
                "MER0013".to_string(),
                self.current_token.span,
                DiagnosticCategory::Syntax,
                None,
            ));
            return None;
        }

        let then_branch = self.parse_block_expression()?;
        let mut end_span = then_branch.span();

        let mut else_branch = None;
        if self.current_token.kind == TokenKind::Else {
            self.advance(); // consume else
            if self.current_token.kind == TokenKind::If {
                let else_if = self.parse_if_expression()?;
                end_span = else_if.span();
                else_branch = Some(Box::new(else_if));
            } else if self.current_token.kind == TokenKind::LBrace {
                let block = self.parse_block_expression()?;
                end_span = block.span();
                else_branch = Some(Box::new(block));
            } else {
                self.diagnostics.push(Diagnostic::new(
                    "Expected '{' or 'if' after else".to_string(),
                    "MER0014".to_string(),
                    self.current_token.span,
                    DiagnosticCategory::Syntax,
                    None,
                ));
                return None;
            }
        }

        Some(Expr::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch,
            span: Span::new(start_span.start, end_span.end),
        })
    }

    fn parse_pattern(&mut self) -> Option<meridian_ast::Pattern> {
        match &self.current_token.kind {
            TokenKind::Identifier(name) => {
                let id_name = name.clone();
                let start_span = self.current_token.span;
                self.advance();

                if self.current_token.kind == TokenKind::Colon && self.peek_token.kind == TokenKind::Colon {
                    self.advance(); // consume :
                    self.advance(); // consume :

                    let variant_name = match &self.current_token.kind {
                        TokenKind::Identifier(n) => n.clone(),
                        _ => {
                            self.diagnostics.push(Diagnostic::new(
                                "Expected variant name after '::'".to_string(),
                                "MER0097".to_string(),
                                self.current_token.span,
                                DiagnosticCategory::Syntax,
                                None,
                            ));
                            return None;
                        }
                    };
                    self.advance(); // consume variant_name

                    let mut binding_name = None;
                    if self.current_token.kind == TokenKind::LParen {
                        self.advance(); // consume (
                        if let TokenKind::Identifier(b_name) = &self.current_token.kind {
                            binding_name = Some(b_name.clone());
                            self.advance(); // consume binding name
                        } else {
                            self.diagnostics.push(Diagnostic::new(
                                "Expected identifier in enum variant pattern".to_string(),
                                "MER0098".to_string(),
                                self.current_token.span,
                                DiagnosticCategory::Syntax,
                                None,
                            ));
                            return None;
                        }

                        if self.current_token.kind != TokenKind::RParen {
                            self.diagnostics.push(Diagnostic::new(
                                "Expected ')' after enum variant pattern".to_string(),
                                "MER0099".to_string(),
                                self.current_token.span,
                                DiagnosticCategory::Syntax,
                                None,
                            ));
                            return None;
                        }
                        self.advance(); // consume )
                    }
                    
                    Some(meridian_ast::Pattern::EnumVariant {
                        enum_name: id_name,
                        variant_name,
                        binding_name,
                        span: Span::new(start_span.start, self.current_token.span.end), // approx
                    })
                } else if id_name == "_" {
                    Some(meridian_ast::Pattern::CatchAll(start_span))
                } else {
                    Some(meridian_ast::Pattern::Identifier(id_name, start_span))
                }
            }
            TokenKind::Number(n) => {
                let pat = meridian_ast::Pattern::Number(*n, self.current_token.span);
                self.advance();
                Some(pat)
            }
            TokenKind::Int(n) => {
                let pat = meridian_ast::Pattern::Int(*n, self.current_token.span);
                self.advance();
                Some(pat)
            }
            TokenKind::String(s) => {
                let pat = meridian_ast::Pattern::String(s.clone(), self.current_token.span);
                self.advance();
                Some(pat)
            }
            TokenKind::True => {
                let pat = meridian_ast::Pattern::Bool(true, self.current_token.span);
                self.advance();
                Some(pat)
            }
            TokenKind::False => {
                let pat = meridian_ast::Pattern::Bool(false, self.current_token.span);
                self.advance();
                Some(pat)
            }
            _ => {
                self.diagnostics.push(Diagnostic::new(
                    "Expected pattern".to_string(),
                    "MER0100".to_string(),
                    self.current_token.span,
                    DiagnosticCategory::Syntax,
                    None,
                ));
                None
            }
        }
    }

    fn parse_match_expression(&mut self) -> Option<Expr> {
        let start_span = self.current_token.span;
        self.advance(); // consume match

        let value = self.parse_expression_impl(0, false)?;

        if self.current_token.kind != TokenKind::LBrace {
            self.diagnostics.push(Diagnostic::new(
                "Expected '{' after match expression".to_string(),
                "MER0101".to_string(),
                self.current_token.span,
                DiagnosticCategory::Syntax,
                None,
            ));
            return None;
        }
        self.advance(); // consume {

        let mut arms = Vec::new();
        while self.current_token.kind != TokenKind::RBrace && self.current_token.kind != TokenKind::EOF {
            let pat = self.parse_pattern()?;

            if self.current_token.kind != TokenKind::FatArrow {
                self.diagnostics.push(Diagnostic::new(
                    "Expected '=>' after match pattern".to_string(),
                    "MER0102".to_string(),
                    self.current_token.span,
                    DiagnosticCategory::Syntax,
                    None,
                ));
                return None;
            }
            self.advance(); // consume =>

            // The arm body is an expression
            let body = self.parse_expression(0)?;
            
            arms.push((pat, body.clone()));

            if self.current_token.kind == TokenKind::Comma {
                self.advance();
            } else if self.current_token.kind != TokenKind::RBrace {
                // if it was a block expression, comma is optional but allowed.
                if !matches!(body, Expr::Block(..)) {
                    self.diagnostics.push(Diagnostic::new(
                        "Expected ',' or '}' after match arm".to_string(),
                        "MER0103".to_string(),
                        self.current_token.span,
                        DiagnosticCategory::Syntax,
                        None,
                    ));
                    return None;
                }
            }
        }

        let end_span = self.current_token.span;
        if self.current_token.kind == TokenKind::RBrace {
            self.advance(); // consume }
        }

        Some(Expr::Match {
            value: Box::new(value),
            arms,
            span: Span::new(start_span.start, end_span.end),
        })
    }

    fn parse_block_expression(&mut self) -> Option<Expr> {
        let start_span = self.current_token.span;
        self.advance(); // consume {

        let mut statements = Vec::new();
        while self.current_token.kind != TokenKind::RBrace && self.current_token.kind != TokenKind::EOF {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            } else {
                self.sync_to_statement();
            }
        }

        if self.current_token.kind != TokenKind::RBrace {
            self.diagnostics.push(Diagnostic::new(
                "Unclosed block, expected '}'".to_string(),
                "MER0017".to_string(),
                self.current_token.span,
                DiagnosticCategory::Syntax,
                None,
            ));
            return Some(Expr::Block(statements, Span::new(start_span.start, self.current_token.span.end)));
        }

        let end_span = self.current_token.span;
        self.advance(); // consume }

        Some(Expr::Block(statements, Span::new(start_span.start, end_span.end)))
    }

    fn peek_precedence(&self) -> u8 {
        match self.current_token.kind {
            TokenKind::LParen => 30,
            TokenKind::Plus | TokenKind::Minus => 10,
            TokenKind::Star | TokenKind::Slash => 20,
            TokenKind::EqualEqual 
            | TokenKind::NotEqual
            | TokenKind::LessThan 
            | TokenKind::LessThanEqual
            | TokenKind::GreaterThan 
            | TokenKind::GreaterThanEqual => 5,
            TokenKind::DotDot | TokenKind::DotDotEqual => 4,
            TokenKind::Equal => 2,
            TokenKind::Dot | TokenKind::LBracket => 40,
            _ => 0,
        }
    }

    fn parse_infix_operator(&self, kind: &TokenKind) -> Option<BinaryOperator> {
        match kind {
            TokenKind::Plus => Some(BinaryOperator::Add),
            TokenKind::Minus => Some(BinaryOperator::Subtract),
            TokenKind::Star => Some(BinaryOperator::Multiply),
            TokenKind::Slash => Some(BinaryOperator::Divide),
            TokenKind::EqualEqual => Some(BinaryOperator::Equal),
            TokenKind::NotEqual => Some(BinaryOperator::NotEqual),
            TokenKind::LessThan => Some(BinaryOperator::LessThan),
            TokenKind::LessThanEqual => Some(BinaryOperator::LessThanEqual),
            TokenKind::GreaterThan => Some(BinaryOperator::GreaterThan),
            TokenKind::GreaterThanEqual => Some(BinaryOperator::GreaterThanEqual),
            _ => None,
        }
    }

    fn infix_precedence(&self, op: &BinaryOperator) -> u8 {
        match op {
            BinaryOperator::Add | BinaryOperator::Subtract => 10,
            BinaryOperator::Multiply | BinaryOperator::Divide => 20,
            BinaryOperator::Equal 
            | BinaryOperator::NotEqual
            | BinaryOperator::LessThan
            | BinaryOperator::LessThanEqual
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterThanEqual => 5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_let() {
        let lexer = Lexer::new("let mut x: Number = 42;");
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        assert_eq!(parser.diagnostics.len(), 0);
        assert_eq!(program.statements.len(), 1);
        
        match &program.statements[0] {
            Stmt::Let { name, mutable, type_annotation, initializer, .. } => {
                assert_eq!(name, "x");
                assert_eq!(*mutable, true);
                assert_eq!(*type_annotation, Some(Type::Number));
                if let Expr::Number(n, _) = initializer {
                    assert_eq!(*n, 42.0);
                } else {
                    panic!("Expected Number");
                }
            }
            _ => panic!("Expected Let statement"),
        }
    }

    #[test]
    fn test_error_recovery_incomplete_block() {
        let source = "
            fn my_func() {
                let x = 10;
                let y = ;
                print x;
            }
            fn another_func() {}
        ";
        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        
        assert!(!parser.diagnostics.is_empty());
        assert_eq!(program.statements.len(), 2);
    }
}
