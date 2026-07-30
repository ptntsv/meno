use crate::ast::Program;
use crate::ast::Stmt;
use crate::lexer::Token::BoolType;
use crate::lexer::Token::CharType;
use crate::lexer::Token::IntType;
use crate::lexer::Token::Let;
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
  fn eat(&mut self, expected: &Token) -> bool {
    self.strm.next_if_eq(&expected).is_some()
  }
  fn expect(&mut self, expected: &Token) {
    if self.strm.next_if_eq(expected).is_none() {
      let got = self.strm.next();
      panic!("Expected {expected:?}, got {got:?}");
    }
  }
  fn expect_type(&mut self) -> Token {
    match self.strm.next() {
      Some(tok @ (IntType | CharType | BoolType)) => tok,
      other => panic!("Expected type, got {other:?}"),
    }
  }
  fn expect_identifier(&mut self) -> usize {
    match self.strm.next() {
      Some(Token::Id(id)) => id,
      other => panic!("Expected identifier, got {other:?}"),
    }
  }
  pub fn new(lexemes: &'a [lexer::Token]) -> Self {
    Parser {
      strm: lexemes.iter().cloned().peekable(),
    }
  }
  pub fn parse_program(&mut self) -> Program {
    let mut stmts = Vec::new();
    while let Some(tok) = self.strm.peek() {
      if matches!(tok, Token::Semicolon) {
        self.strm.next();
        continue;
      }
      stmts.push(self.parse_stmt());
    }
    Program { stmts: stmts }
  }
  fn parse_stmt(&mut self) -> Stmt {
    if self.eat(&Let) {
      let id = self.expect_identifier();
      if self.eat(&Token::Colon) {
        let t = self.expect_type();
      }
      self.expect(&Token::EQ);
      let rhs = self.parse_expr();
      return Stmt::Assignment {
        name_id: id,
        rhs: Box::new(rhs),
      };
    }
    panic!("Expected statement");
  }
  // E = T ('+'|'-' T)*
  fn parse_expr(&mut self) -> Expr {
    let mut root = self.parse_term();
    let mut op: BinaryOp;
    loop {
      if self.eat(&Token::Plus) {
        op = BinaryOp::Add;
      } else if self.eat(&Token::Minus) {
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
      if self.eat(&Token::Star) {
        op = BinaryOp::Mul;
      } else if self.eat(&Token::Slash) {
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
    if self.eat(&Token::Minus) {
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
    if self.eat(&Token::LParen) {
      let expr = self.parse_expr();
      self.expect(&Token::RParen);
      expr
    } else if let Some(&Token::IntLit(x)) = self.strm.peek() {
      self.strm.next();
      Expr::Int(x)
    } else {
      Expr::Int(42)
    }
  }
}

// ========= UNIT-TESTS =========

#[cfg(test)]
mod tests {
  use super::*;
  use crate::ast::BinaryOp::*;
  use crate::ast::Expr::*;
  use crate::ast::UnaryOp::*;
  use crate::ast::*;
  use crate::lexer::*;
  fn _parse_expr(s: &str) -> Expr {
    let mut lexer = Lexer::new(s);
    let lxms = &lexer.tokenize();
    let mut parser = Parser::new(lxms);
    parser.parse_expr()
  }
  fn _parse_stmt(s: &str) -> Stmt {
    let mut lexer = Lexer::new(s);
    let lxms = &lexer.tokenize();
    let mut parser = Parser::new(lxms);
    parser.parse_stmt()
  }
  #[test]
  fn int1() {
    let s = "1";
    match _parse_expr(s) {
      Expr::Int(1) => (),
      _ => panic!(),
    }
  }
  #[test]
  fn unary_int() {
    let s = "-42";
    let expected = Expr::Unary {
      op: Neg,
      child: Box::new(Expr::Int(42)),
    };
    assert_eq!(expected, _parse_expr(s));
    let s = "--42";
    let expected = Expr::Unary {
      op: Neg,
      child: Box::new(Expr::Unary {
        op: Neg,
        child: Box::new(Expr::Int(42)),
      }),
    };
    assert_eq!(expected, _parse_expr(s));
    let s = "-1-2";
    let expected = Expr::Binary {
      op: Sub,
      left: Box::new(Expr::Unary {
        op: Neg,
        child: Box::new(Expr::Int(1)),
      }),
      right: Box::new(Expr::Int(2)),
    };
    assert_eq!(expected, _parse_expr(s));
    let s = "-1--2";
    let expected = Expr::Binary {
      op: Sub,
      left: Box::new(Expr::Unary {
        op: Neg,
        child: Box::new(Expr::Int(1)),
      }),
      right: Box::new(Expr::Unary {
        op: Neg,
        child: Box::new(Int(2)),
      }),
    };
    assert_eq!(expected, _parse_expr(s));
  }
  #[test]
  fn unary_group() {
    let s = "-(1+2)";
    let expected = Expr::Unary {
      op: Neg,
      child: Box::new(Expr::Binary {
        op: Add,
        left: Box::new(Expr::Int(1)),
        right: Box::new(Expr::Int(2)),
      }),
    };
    assert_eq!(expected, _parse_expr(s));
  }
  #[test]
  fn precedence_test() {
    let s = "1+2*3";
    let expected = Expr::Binary {
      op: Add,
      left: Box::new(Int(1)),
      right: Box::new(Expr::Binary {
        op: Mul,
        left: Box::new(Int(2)),
        right: Box::new(Int(3)),
      }),
    };
    assert_eq!(expected, _parse_expr(s));
    let s = "(1+2)*3";
    let expected = Expr::Binary {
      op: Mul,
      left: Box::new(Expr::Binary {
        op: Add,
        left: Box::new(Int(1)),
        right: Box::new(Int(2)),
      }),
      right: Box::new(Int(3)),
    };
    assert_eq!(expected, _parse_expr(s));
    let s = "1+(2+3)";
    let expected = Expr::Binary {
      op: Add,
      left: Box::new(Int(1)),
      right: Box::new(Expr::Binary {
        op: Add,
        left: Box::new(Int(2)),
        right: Box::new(Int(3)),
      }),
    };
    assert_eq!(expected, _parse_expr(s));
  }
  #[test]
  fn assignment() {
    let s = "let x = 13;";
    let expected = Stmt::Assignment {
      name_id: 0,
      rhs: Box::new(Expr::Int(13)),
    };
    assert_eq!(expected, _parse_stmt(s))
  }
}
