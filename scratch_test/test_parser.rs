use meridian_parser::{Parser, Lexer};
use meridian_ast::Program;

fn main() {
    let source = "
    fn missing_brace() {
        let x = 10;
        
    fn valid_fn() {
        print 20;
    }
    ";
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    
    for diag in &parser.diagnostics {
        println!("Diagnostic: {:?}", diag);
    }
    
    println!("{:#?}", program);
}
