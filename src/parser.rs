use crate::lexer;
use crate::ast;
pub enum Token {
}
pub struct Parser {
  lexemes: Vec<lexer::Lexeme>,
  idx : usize
}
impl Parser {
  pub fn new(_lexemes : &[lexer::Lexeme]) -> Self {
    Parser {lexemes: _lexemes.to_vec(), idx: 0}
  }
  // pub fn tokenize() -> Vec<Token> {
  // }
  // pub fn parse(&self) -> ast::AstNode {
  // }
}
