sed -i '' -e '/"MER0011".to_string(),/c\
                "MER0011".to_string(),\
                self.current_token.span,\
                DiagnosticCategory::Syntax,\
                None,\
            );\
            return Some(Stmt::Let {\
                name,\
                mutable,\
                type_annotation,\
                initializer: Expr::Error(self.current_token.span),\
                span: Span::new(start_span.start, self.current_token.span.end),\
            });\
' parser/src/lib.rs
