use meridian_diagnostics::{Diagnostic, DiagnosticCategory, Span};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Identifier(String),
    Number(f64),
    String(String),
    Let,
    Mut,
    Print,
    If,
    Else,
    Fn,
    While,
    For,
    In,
    Break,
    Continue,
    Async,
    Await,
    Spawn,
    Macro,
    Extern,
    Unsafe,
    Import,
    Struct,
    Enum,
    Impl,
    Trait,
    Match,
    Return,
    Int(i64),
    True,
    False,
    Equal,
    EqualEqual,
    NotEqual,
    LessThan,
    LessThanEqual,
    GreaterThan,
    GreaterThanEqual,
    Plus,
    Minus,
    Star,
    Slash,
    Semicolon,
    Colon,
    Comma,
    Arrow,
    FatArrow,
    Dot,
    DotDot,
    DotDotEqual,
    Ampersand,
    Bang,
    Dollar,
    Question,
    DocComment(String),
    LBrace,
    RBrace,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Hash,
    EOF,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

pub struct Lexer<'a> {
    source: &'a str,
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    pub diagnostics: Vec<Diagnostic>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.char_indices().peekable(),
            diagnostics: Vec::new(),
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let (start, ch) = match self.chars.next() {
            Some(res) => res,
            None => {
                return Token {
                    kind: TokenKind::EOF,
                    span: Span::new(self.source.len(), self.source.len()),
                }
            }
        };

        match ch {
            '+' => Token { kind: TokenKind::Plus, span: Span::new(start, start + 1) },
            '-' => {
                if let Some(&(next_idx, '>')) = self.chars.peek() {
                    self.chars.next();
                    Token { kind: TokenKind::Arrow, span: Span::new(start, next_idx + 1) }
                } else {
                    Token { kind: TokenKind::Minus, span: Span::new(start, start + 1) }
                }
            }
            '*' => Token { kind: TokenKind::Star, span: Span::new(start, start + 1) },
            '/' => {
                if let Some(&(_, '/')) = self.chars.peek() {
                    self.chars.next(); // consume second '/'
                    if let Some(&(_, '/')) = self.chars.peek() {
                        self.chars.next(); // consume third '/'
                        return self.lex_doc_comment(start);
                    }
                    self.skip_line_comment();
                    self.next_token()
                } else if let Some(&(_, '*')) = self.chars.peek() {
                    self.skip_block_comment();
                    self.next_token()
                } else {
                    Token { kind: TokenKind::Slash, span: Span::new(start, start + 1) }
                }
            }
            ';' => Token { kind: TokenKind::Semicolon, span: Span::new(start, start + 1) },
            ':' => Token { kind: TokenKind::Colon, span: Span::new(start, start + 1) },
            ',' => Token { kind: TokenKind::Comma, span: Span::new(start, start + 1) },
            '{' => Token { kind: TokenKind::LBrace, span: Span::new(start, start + 1) },
            '}' => Token { kind: TokenKind::RBrace, span: Span::new(start, start + 1) },
            '(' => Token { kind: TokenKind::LParen, span: Span::new(start, start + 1) },
            ')' => Token { kind: TokenKind::RParen, span: Span::new(start, start + 1) },
            '[' => Token { kind: TokenKind::LBracket, span: Span::new(start, start + 1) },
            ']' => Token { kind: TokenKind::RBracket, span: Span::new(start, start + 1) },
            '#' => Token { kind: TokenKind::Hash, span: Span::new(start, start + 1) },
            '&' => Token { kind: TokenKind::Ampersand, span: Span::new(start, start + 1) },
            '=' => {
                if let Some(&(next_idx, '=')) = self.chars.peek() {
                    self.chars.next();
                    Token { kind: TokenKind::EqualEqual, span: Span::new(start, next_idx + 1) }
                } else if let Some(&(next_idx, '>')) = self.chars.peek() {
                    self.chars.next();
                    Token { kind: TokenKind::FatArrow, span: Span::new(start, next_idx + 1) }
                } else {
                    Token { kind: TokenKind::Equal, span: Span::new(start, start + 1) }
                }
            }
            '<' => {
                if let Some(&(next_idx, '=')) = self.chars.peek() {
                    self.chars.next();
                    Token { kind: TokenKind::LessThanEqual, span: Span::new(start, next_idx + 1) }
                } else {
                    Token { kind: TokenKind::LessThan, span: Span::new(start, start + 1) }
                }
            }
            '>' => {
                if let Some(&(next_idx, '=')) = self.chars.peek() {
                    self.chars.next();
                    Token { kind: TokenKind::GreaterThanEqual, span: Span::new(start, next_idx + 1) }
                } else {
                    Token { kind: TokenKind::GreaterThan, span: Span::new(start, start + 1) }
                }
            }
            '!' => {
                if let Some(&(next_idx, '=')) = self.chars.peek() {
                    self.chars.next();
                    Token { kind: TokenKind::NotEqual, span: Span::new(start, next_idx + 1) }
                } else {
                    Token { kind: TokenKind::Bang, span: Span::new(start, start + 1) }
                }
            }
            '.' => {
                if let Some(&(next_idx, '.')) = self.chars.peek() {
                    self.chars.next();
                    if let Some(&(next_next_idx, '=')) = self.chars.peek() {
                        self.chars.next();
                        Token { kind: TokenKind::DotDotEqual, span: Span::new(start, next_next_idx + 1) }
                    } else {
                        Token { kind: TokenKind::DotDot, span: Span::new(start, next_idx + 1) }
                    }
                } else {
                    Token { kind: TokenKind::Dot, span: Span::new(start, start + 1) }
                }
            }
            '$' => Token { kind: TokenKind::Dollar, span: Span::new(start, start + 1) },
            '?' => Token { kind: TokenKind::Question, span: Span::new(start, start + 1) },
            '"' => self.lex_string(start),
            _ if ch.is_ascii_digit() => self.lex_number(start, ch),
            _ if ch.is_alphabetic() || ch == '_' => self.lex_identifier(start, ch),
            _ => {
                let span = Span::new(start, start + ch.len_utf8());
                self.diagnostics.push(Diagnostic::new(
                    format!("Unexpected character: {}", ch),
                    "MER0001".to_string(),
                    span,
                    DiagnosticCategory::Lexical,
                    None,
                ));
                Token { kind: TokenKind::Error, span }
            }
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(&(_, ch)) = self.chars.peek() {
            if ch.is_whitespace() {
                self.chars.next();
            } else {
                break;
            }
        }
    }

    fn skip_line_comment(&mut self) {
        while let Some(&(_, ch)) = self.chars.peek() {
            if ch == '\n' {
                break;
            }
            self.chars.next();
        }
    }

    fn lex_doc_comment(&mut self, start: usize) -> Token {
        let mut text = String::new();
        let mut end = start + 3; // "///" is 3 bytes

        while let Some(&(idx, ch)) = self.chars.peek() {
            if ch == '\n' {
                break;
            }
            text.push(ch);
            end = idx + ch.len_utf8();
            self.chars.next();
        }

        Token {
            kind: TokenKind::DocComment(text.trim().to_string()),
            span: Span::new(start, end),
        }
    }

    fn skip_block_comment(&mut self) {
        self.chars.next(); // consume '*'
        let mut depth = 1;
        while let Some((_, ch)) = self.chars.next() {
            if ch == '/' {
                if let Some(&(_, '*')) = self.chars.peek() {
                    self.chars.next();
                    depth += 1;
                }
            } else if ch == '*' {
                if let Some(&(_, '/')) = self.chars.peek() {
                    self.chars.next();
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
            }
        }
    }

    fn lex_number(&mut self, start: usize, first: char) -> Token {
        let mut text = String::new();
        text.push(first);
        let mut end = start + first.len_utf8();

        let mut is_float = false;

        while let Some(&(idx, ch)) = self.chars.peek() {
            if ch.is_ascii_digit() {
                text.push(ch);
                end = idx + ch.len_utf8();
                self.chars.next();
            } else if ch == '.' {
                let mut lookahead = self.chars.clone();
                lookahead.next(); // skip the '.'
                if let Some(&(_, next_ch)) = lookahead.peek() {
                    if next_ch == '.' {
                        break; // It's a range operator `..`, don't consume `.`
                    }
                }
                text.push(ch);
                end = idx + ch.len_utf8();
                self.chars.next();
                is_float = true;
            } else {
                break;
            }
        }

        if is_float {
            let num = text.parse::<f64>().unwrap_or(0.0);
            Token {
                kind: TokenKind::Number(num),
                span: Span::new(start, end),
            }
        } else {
            let num = text.parse::<i64>().unwrap_or(0);
            Token {
                kind: TokenKind::Int(num),
                span: Span::new(start, end),
            }
        }
    }

    fn lex_identifier(&mut self, start: usize, first: char) -> Token {
        let mut text = String::new();
        text.push(first);
        let mut end = start + first.len_utf8();

        while let Some(&(idx, ch)) = self.chars.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                text.push(ch);
                end = idx + ch.len_utf8();
                self.chars.next();
            } else {
                break;
            }
        }

        let kind = match text.as_str() {
            "let" => TokenKind::Let,
            "mut" => TokenKind::Mut,
            "print" => TokenKind::Print,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "fn" => TokenKind::Fn,
            "while" => TokenKind::While,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "async" => TokenKind::Async,
            "await" => TokenKind::Await,
            "spawn" => TokenKind::Spawn,
            "macro" => TokenKind::Macro,
            "extern" => TokenKind::Extern,
            "unsafe" => TokenKind::Unsafe,
            "import" => TokenKind::Import,
            "struct" => TokenKind::Struct,
            "enum" => TokenKind::Enum,
            "impl" => TokenKind::Impl,
            "trait" => TokenKind::Trait,
            "match" => TokenKind::Match,
            "return" => TokenKind::Return,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            _ => TokenKind::Identifier(text),
        };

        Token {
            kind,
            span: Span::new(start, end),
        }
    }

    fn lex_string(&mut self, start: usize) -> Token {
        let mut text = String::new();
        let mut end = start + 1;
        let mut closed = false;

        for (idx, ch) in self.chars.by_ref() {
            end = idx + ch.len_utf8();
            if ch == '"' {
                closed = true;
                break;
            }
            text.push(ch);
        }

        let span = Span::new(start, end);

        if !closed {
            self.diagnostics.push(Diagnostic::new(
                "Unterminated string literal".to_string(),
                "MER0002".to_string(),
                span,
                DiagnosticCategory::Lexical,
                Some("Add a closing quote '\"'.".to_string()),
            ));
            return Token { kind: TokenKind::Error, span };
        }

        Token {
            kind: TokenKind::String(text),
            span,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_let() {
        let mut lexer = Lexer::new("let mut x = 42;");
        assert_eq!(lexer.next_token().kind, TokenKind::Let);
        assert_eq!(lexer.next_token().kind, TokenKind::Mut);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("x".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::Equal);
        assert_eq!(lexer.next_token().kind, TokenKind::Int(42));
        assert_eq!(lexer.next_token().kind, TokenKind::Semicolon);
        assert_eq!(lexer.next_token().kind, TokenKind::EOF);
        assert!(lexer.diagnostics.is_empty());
    }

    #[test]
    fn test_lex_error() {
        let mut lexer = Lexer::new("@");
        assert_eq!(lexer.next_token().kind, TokenKind::Error);
        assert_eq!(lexer.diagnostics.len(), 1);
        assert_eq!(lexer.diagnostics[0].machine_code, "MER0001");
    }
}
