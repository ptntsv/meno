use crate::ast::AstNode;
use std::iter::Cloned;
use std::iter::Peekable;
use std::slice::Iter;

use crate::ast::BinaryOp;
use crate::ast::Expr;
use crate::ast::UnaryOp;
use crate::lexer;
use crate::lexer::Token;

pub struct Parser<'a> {
  strm: Peekable<Cloned<Iter<'a, lexer::Token>>>,
}
impl<'a> Parser<'a> {
  fn match_token(&mut self, expected: &Token) -> bool {
    self.strm.next_if_eq(&expected).is_some()
  }
  fn expect_token(&mut self, expected: &Token) {
    assert_eq!(self.strm.next().as_ref(), Some(expected));
  }
  pub fn new(lexemes: &'a [lexer::Token]) -> Self {
    Parser {
      strm: lexemes.iter().cloned().peekable(),
    }
  }
  pub fn parse_program(&mut self) -> AstNode {
    AstNode::Expr(self.parse_expr())
  }
  // E = T ('+'|'-' T)*
  fn parse_expr(&mut self) -> Expr {
    let mut root = self.parse_term();
    let mut op: BinaryOp;
    loop {
      if self.match_token(&Token::Plus) {
        op = BinaryOp::Add;
      } else if self.match_token(&Token::Minus) {
        op = BinaryOp::Sub;
      } else {
        break;
      }
      let right = self.parse_term();
      root = Expr::Binary {
        op: op,
        left: Box::new(root),
        right: Box::new(right),
      };
    }
    root
  }
  // T = U ('*'|'/' U)*
  fn parse_term(&mut self) -> Expr {
    let mut root = self.parse_unary();
    let mut op: BinaryOp;
    loop {
      if self.match_token(&Token::Star) {
        op = BinaryOp::Mul;
      } else if self.match_token(&Token::Slash) {
        op = BinaryOp::Div;
      } else {
        break;
      }
      let right = self.parse_unary();
      root = Expr::Binary {
        op: op,
        left: Box::new(root),
        right: Box::new(right),
      };
    }
    root
  }
  // U = -U | F
  fn parse_unary(&mut self) -> Expr {
    if self.match_token(&Token::Minus) {
      let expr = self.parse_unary();
      Expr::Unary {
        op: UnaryOp::Neg,
        child: Box::new(expr),
      }
    } else {
      self.parse_factor()
    }
  }
  // F = Int | '(' E ')'
  fn parse_factor(&mut self) -> Expr {
    if self.match_token(&Token::LParen) {
      let expr = self.parse_expr();
      self.expect_token(&Token::RParen);
      expr
    } else if let Some(&Token::Int(x)) = self.strm.peek() {
      self.strm.next();
      Expr::Int(x)
    } else {
      Expr::Int(42)
    }
  }
}
