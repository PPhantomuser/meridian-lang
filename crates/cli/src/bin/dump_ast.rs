use meridian_parser::Parser;
use meridian_lexer::Lexer;

fn main() {
    let source = include_str!("../../../../test_phase3.mer");
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::new();
    loop {
        let tok = lexer.next_token();
        tokens.push(tok.clone());
        if tok.kind == meridian_lexer::TokenKind::EOF { break; }
    }
    println!("Tokens: {:#?}", tokens);
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    println!("Program: {:#?}", program);
    for d in &parser.diagnostics {
        println!("Diag: {:#?}", d);
    }
}
