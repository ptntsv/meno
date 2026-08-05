use crate::ast::BinaryOp;
use crate::ast::Program;
use crate::ast::Stmt;
use crate::lexer::Token::BoolType;
use crate::lexer::Token::CharType;
use crate::lexer::Token::IntType;
use crate::type_checker::Type;

use crate::ast::Expr;
use crate::ast::UnaryOp;
use crate::lexer;
use crate::lexer::Token;

pub struct Parser {
  tokens: Vec<Token>,
  pos: usize,
  idtable: Vec<String>,
}
impl Parser {
  fn get(&mut self, off: usize) -> Option<&Token> {
    self.tokens.get(self.pos + off)
  }
  fn putback(&mut self) {
    self.pos = std::cmp::max(self.pos - 1, 0);
  }
  fn peek(&mut self) -> Option<&Token> {
    self.get(0)
  }
  fn next(&mut self) -> Option<&Token> {
    let mbtok = self.tokens.get(self.pos);
    if mbtok.is_some() {
      self.pos += 1;
    }
    mbtok
  }
  fn eat(&mut self, expected: &Token) -> bool {
    match self.peek() {
      Some(tok) if tok == expected => {
        self.next();
        true
      }
      _ => false,
    }
  }
  fn expect(&mut self, expected: &Token) {
    if !self.eat(expected) {
      let got = self.peek();
      panic!("Expected {expected:?}, got {got:?}");
    }
  }
  fn expect_type(&mut self) -> &Token {
    match self.next() {
      Some(tok @ (IntType | CharType | BoolType)) => tok,
      other => panic!("Expected type, got {other:?}"),
    }
  }
  fn expect_identifier(&mut self) -> usize {
    match self.next() {
      Some(&Token::Id(id)) => id,
      other => panic!("Expected identifier, got {other:?}"),
    }
  }
  pub fn new(lexemes: &[lexer::Token], table: Vec<String>) -> Self {
    Parser {
      tokens: lexemes.to_vec(),
      pos: 0,
      idtable: table,
    }
  }
  pub fn parse_program(mut self) -> Program {
    let mut stmts = Vec::new();
    while let Some(_) = self.peek() {
      stmts.push(self.parse_stmt());
    }
    Program {
      stmts: stmts,
      idtable: self.idtable,
    }
  }
  // Stmt = Decl | Assignment | Expr ';'
  // Decl = 'let' Id (':' Type )? '=' Expr
  // Assignment = Id '=' Expr
  fn parse_stmt(&mut self) -> Stmt {
    let stmt: Stmt;
    if let Some(&Token::Let) = self.peek() {
      stmt = self.parse_decl();
    } else if let Some(&Token::Assign) = self.get(1) {
      stmt = self.parse_assignment();
    } else {
      stmt = Stmt::Expr(self.parse_expr());
    }
    self.expect(&Token::Semicolon);
    stmt
  }
  fn parse_assignment(&mut self) -> Stmt {
    let lhs = self.expect_identifier();
    self.expect(&Token::Assign);
    let rhs = self.parse_expr();
    Stmt::Assignment {
      name_id: lhs,
      rhs: Box::new(rhs),
    }
  }
  fn parse_decl(&mut self) -> Stmt {
    self.expect(&Token::Let);
    let id = self.expect_identifier();
    let mut maybe_type = None;
    if self.eat(&Token::Colon) {
      maybe_type = match self.expect_type() {
        Token::IntType => Some(Type::Int),
        Token::CharType => Some(Type::Char),
        Token::BoolType => Some(Type::Bool),
        other => panic!("Expected type but got {other:?}"),
      };
    }
    self.expect(&Token::Assign);
    let rhs = self.parse_expr();
    Stmt::Decl {
      name_id: id,
      decl_type: maybe_type,
      rhs: Box::new(rhs),
    }
  }
  fn parse_if(&mut self) -> Expr {
    self.expect(&Token::If);
    let cond = Box::new(self.parse_expr());
    let then = Box::new(self.parse_block());
    let mut otherwise: Option<Box<Expr>> = None;
    if self.eat(&Token::Else) {
      if let Some(&Token::If) = self.peek() {
        otherwise = Some(Box::new(self.parse_if()));
      } else {
        otherwise = Some(Box::new(self.parse_block()));
      }
    }
    Expr::If {
      cond: cond,
      tbranch: then,
      fbranch: otherwise,
    }
  }
  fn parse_expr(&mut self) -> Expr {
    self.parse_logical_or()
  }
  fn parse_block(&mut self) -> Expr {
    let mut content = Vec::new();
    self.expect(&Token::LCBrace);
    while Some(&Token::RCBrace) != self.peek() {
      content.push(self.parse_stmt());
    }
    self.expect(&Token::RCBrace);
    Expr::Block { content: content }
  }
  fn parse_logical_or(&mut self) -> Expr {
    let mut root = self.parse_logical_and();
    loop {
      if self.eat(&Token::LogicOr) {
        root = Expr::Binary {
          op: BinaryOp::Or,
          left: Box::new(root),
          right: Box::new(self.parse_logical_and()),
        };
      } else {
        break;
      }
    }
    root
  }
  fn parse_logical_and(&mut self) -> Expr {
    let mut root = self.parse_comparison();
    loop {
      if self.eat(&Token::LogicAnd) {
        root = Expr::Binary {
          op: BinaryOp::And,
          left: Box::new(root),
          right: Box::new(self.parse_comparison()),
        };
      } else {
        break;
      }
    }
    root
  }
  // C = E (op E)*
  fn parse_comparison(&mut self) -> Expr {
    let mut root = self.parse_aexpr();
    let tok_to_op = |tok: Option<&Token>| match tok {
      Some(Token::Lt) => Some(BinaryOp::Lt),
      Some(Token::Lte) => Some(BinaryOp::Lte),
      Some(Token::Gt) => Some(BinaryOp::Gt),
      Some(Token::Gte) => Some(BinaryOp::Gte),
      Some(Token::Eq) => Some(BinaryOp::Eq),
      _ => None,
    };
    let mut op: BinaryOp;
    loop {
      if let Some(_op) = tok_to_op(self.peek()) {
        self.next();
        op = _op;
      } else {
        break;
      }
      let right = self.parse_aexpr();
      root = Expr::Binary {
        op: op,
        left: Box::new(root),
        right: Box::new(right),
      };
    }
    root
  }
  // E = T ('+'|'-' T)*
  fn parse_aexpr(&mut self) -> Expr {
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
  // F = Int | '(' C ')'
  fn parse_factor(&mut self) -> Expr {
    match self.next() {
      Some(&Token::LParen) => {
        let expr = self.parse_expr();
        self.expect(&Token::RParen);
        expr
      }
      Some(&Token::If) => {
        self.putback();
        self.parse_if()
      }
      Some(&Token::LCBrace) => {
        self.putback();
        self.parse_block()
      }
      Some(&Token::TrueLit) => Expr::ConstBool(true),
      Some(&Token::FalseLit) => Expr::ConstBool(false),
      Some(&Token::IntLit(x)) => Expr::ConstInt(x),
      Some(&Token::Id(id)) => Expr::Var(id),
      other => panic!("Expected factor but got {other:?}"),
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
    let mut parser = Parser::new(lxms, lexer.idtable);
    parser.parse_expr()
  }
  fn _parse_stmt(s: &str) -> Stmt {
    let mut lexer = Lexer::new(s);
    let lxms = &lexer.tokenize();
    let mut parser = Parser::new(lxms, lexer.idtable);
    parser.parse_stmt()
  }
  #[test]
  fn int1() {
    let s = "1";
    match _parse_expr(s) {
      Expr::ConstInt(1) => (),
      _ => panic!(),
    }
  }
  #[test]
  fn unary_int() {
    let s = "-42";
    let expected = Expr::Unary {
      op: Neg,
      child: Box::new(Expr::ConstInt(42)),
    };
    assert_eq!(expected, _parse_expr(s));
    let s = "--42";
    let expected = Expr::Unary {
      op: Neg,
      child: Box::new(Expr::Unary {
        op: Neg,
        child: Box::new(Expr::ConstInt(42)),
      }),
    };
    assert_eq!(expected, _parse_expr(s));
    let s = "-1-2";
    let expected = Expr::Binary {
      op: Sub,
      left: Box::new(Expr::Unary {
        op: Neg,
        child: Box::new(Expr::ConstInt(1)),
      }),
      right: Box::new(Expr::ConstInt(2)),
    };
    assert_eq!(expected, _parse_expr(s));
    let s = "-1--2";
    let expected = Expr::Binary {
      op: Sub,
      left: Box::new(Expr::Unary {
        op: Neg,
        child: Box::new(Expr::ConstInt(1)),
      }),
      right: Box::new(Expr::Unary {
        op: Neg,
        child: Box::new(ConstInt(2)),
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
        left: Box::new(Expr::ConstInt(1)),
        right: Box::new(Expr::ConstInt(2)),
      }),
    };
    assert_eq!(expected, _parse_expr(s));
  }
  #[test]
  fn precedence_test() {
    let s = "1+2*3";
    let expected = Expr::Binary {
      op: Add,
      left: Box::new(ConstInt(1)),
      right: Box::new(Expr::Binary {
        op: Mul,
        left: Box::new(ConstInt(2)),
        right: Box::new(ConstInt(3)),
      }),
    };
    assert_eq!(expected, _parse_expr(s));
    let s = "(1+2)*3";
    let expected = Expr::Binary {
      op: Mul,
      left: Box::new(Expr::Binary {
        op: Add,
        left: Box::new(ConstInt(1)),
        right: Box::new(ConstInt(2)),
      }),
      right: Box::new(ConstInt(3)),
    };
    assert_eq!(expected, _parse_expr(s));
    let s = "1+(2+3)";
    let expected = Expr::Binary {
      op: Add,
      left: Box::new(ConstInt(1)),
      right: Box::new(Expr::Binary {
        op: Add,
        left: Box::new(ConstInt(2)),
        right: Box::new(ConstInt(3)),
      }),
    };
    assert_eq!(expected, _parse_expr(s));
  }
  #[test]
  fn decl() {
    let s = "let x: int = 13;";
    let expected = Stmt::Decl {
      name_id: 0,
      decl_type: Some(Type::Int),
      rhs: Box::new(Expr::ConstInt(13)),
    };
    assert_eq!(expected, _parse_stmt(s))
  }
  #[test]
  fn lt_test() {
    let s = "1 < 2";
    let expected = Expr::Binary {
      op: BinaryOp::Lt,
      left: Box::new(Expr::ConstInt(1)),
      right: Box::new(Expr::ConstInt(2)),
    };
    assert_eq!(_parse_expr(s), expected);
    let s = "1 < (2 + 3)";
    let expected = Expr::Binary {
      op: BinaryOp::Lt,
      left: Box::new(Expr::ConstInt(1)),
      right: Box::new(Expr::Binary {
        op: BinaryOp::Add,
        left: Box::new(Expr::ConstInt(2)),
        right: Box::new(Expr::ConstInt(3)),
      }),
    };
    assert_eq!(_parse_expr(s), expected);
  }
  #[test]
  fn comp_precedence_test() {
    let s = "1 < 2 + 3";
    let expected = Expr::Binary {
      op: BinaryOp::Lt,
      left: Box::new(Expr::ConstInt(1)),
      right: Box::new(Expr::Binary {
        op: BinaryOp::Add,
        left: Box::new(Expr::ConstInt(2)),
        right: Box::new(Expr::ConstInt(3)),
      }),
    };
    assert_eq!(_parse_expr(s), expected);
    let s = "1 < 2 || 3 > 5";
    let expected = Expr::Binary {
      op: BinaryOp::Or,
      left: Box::new(Expr::Binary {
        op: BinaryOp::Lt,
        left: Box::new(Expr::ConstInt(1)),
        right: Box::new(Expr::ConstInt(2)),
      }),
      right: Box::new(Expr::Binary {
        op: BinaryOp::Gt,
        left: Box::new(Expr::ConstInt(3)),
        right: Box::new(Expr::ConstInt(5)),
      }),
    };
    assert_eq!(_parse_expr(s), expected);
  }
  #[test]
  fn logic_precedence_test() {
    let s = "true || false && true";
    let expected = Expr::Binary {
      op: BinaryOp::Or,
      left: Box::new(Expr::ConstBool(true)),
      right: Box::new(Expr::Binary {
        op: BinaryOp::And,
        left: Box::new(Expr::ConstBool(false)),
        right: Box::new(Expr::ConstBool(true)),
      }),
    };
    assert_eq!(_parse_expr(s), expected);
    let s = "true || false || true";
    let expected = Expr::Binary {
      op: BinaryOp::Or,
      left: Box::new(Expr::Binary {
        op: BinaryOp::Or,
        left: Box::new(Expr::ConstBool(true)),
        right: Box::new(Expr::ConstBool(false)),
      }),
      right: Box::new(Expr::ConstBool(true)),
    };
    assert_eq!(_parse_expr(s), expected);
    let s = "true == false && false";
    let expected = Expr::Binary {
      op: BinaryOp::And,
      left: Box::new(Expr::Binary {
        op: BinaryOp::Eq,
        left: Box::new(Expr::ConstBool(true)),
        right: Box::new(Expr::ConstBool(false)),
      }),
      right: Box::new(Expr::ConstBool(false)),
    };
    assert_eq!(_parse_expr(s), expected);
  }
  #[test]
  fn if_simple() {
    let s = "if 1 < 2 { 2; } else { 3; }";
    let exp = Expr::If {
      cond: Box::new(Expr::Binary {
        op: BinaryOp::Lt,
        left: Box::new(Expr::ConstInt(1)),
        right: Box::new(Expr::ConstInt(2)),
      }),
      tbranch: Box::new(Expr::Block {
        content: vec![Stmt::Expr(Expr::ConstInt(2))],
      }),
      fbranch: Some(Box::new(Expr::Block {
        content: vec![Stmt::Expr(Expr::ConstInt(3))],
      })),
    };
    assert_eq!(_parse_expr(s), exp);
  }
  #[test]
  fn no_then_if() {
    let s = "if 1 < 2 { 2; }";
    let exp = Expr::If {
      cond: Box::new(Expr::Binary {
        op: BinaryOp::Lt,
        left: Box::new(Expr::ConstInt(1)),
        right: Box::new(Expr::ConstInt(2)),
      }),
      tbranch: Box::new(Expr::Block {
        content: vec![Stmt::Expr(Expr::ConstInt(2))],
      }),
      fbranch: None,
    };
    assert_eq!(_parse_expr(s), exp);
    let s = "if 1 < 2 {
      let x: int = 42;
      2;
    }";
    let exp = Expr::If {
      cond: Box::new(Expr::Binary {
        op: BinaryOp::Lt,
        left: Box::new(Expr::ConstInt(1)),
        right: Box::new(Expr::ConstInt(2)),
      }),
      tbranch: Box::new(Expr::Block {
        content: vec![
          Stmt::Decl {
            name_id: 0,
            decl_type: Some(Type::Int),
            rhs: Box::new(Expr::ConstInt(42)),
          },
          Stmt::Expr(Expr::ConstInt(2)),
        ],
      }),
      fbranch: None,
    };
    assert_eq!(_parse_expr(s), exp);
  }
  #[test]
  fn nested_if() {
    let s = "
    if 1 == 1 {
      2;
    } else if 1 < 2 {
      3;
    } else {
      5;
    };";
    let b1 = Box::new(Expr::Block {
      content: vec![Stmt::Expr(Expr::ConstInt(2))],
    });
    let b2 = Box::new(Expr::Block {
      content: vec![Stmt::Expr(Expr::ConstInt(3))],
    });
    let b3 = Box::new(Expr::Block {
      content: vec![Stmt::Expr(Expr::ConstInt(5))],
    });
    let elseif = Box::new(Expr::If {
      cond: Box::new(Expr::Binary {
        op: BinaryOp::Lt,
        left: Box::new(Expr::ConstInt(1)),
        right: Box::new(Expr::ConstInt(2)),
      }),
      tbranch: b2,
      fbranch: Some(b3),
    });
    let exp = Expr::If {
      cond: Box::new(Expr::Binary {
        op: BinaryOp::Eq,
        left: Box::new(Expr::ConstInt(1)),
        right: Box::new(Expr::ConstInt(1)),
      }),
      tbranch: b1,
      fbranch: Some(elseif),
    };
    assert_eq!(_parse_expr(s), exp);
  }
  // TODO:
  // - To remove Peekable [x]
  // - If :: 'if' expr Block 'else' (If | Block)* [x]
  // - To implement assignment [x]
  #[test]
  fn assignment_silly() {
    let s = "x = 1 + 2;";
    let exp = Stmt::Assignment {
      name_id: 0,
      rhs: Box::new(Expr::Binary {
        op: BinaryOp::Add,
        left: Box::new(Expr::ConstInt(1)),
        right: Box::new(Expr::ConstInt(2)),
      }),
    };
    assert_eq!(_parse_stmt(s), exp);
    let s = "x = true && false;";
    let exp = Stmt::Assignment {
      name_id: 0,
      rhs: Box::new(Expr::Binary {
        op: BinaryOp::And,
        left: Box::new(Expr::ConstBool(true)),
        right: Box::new(Expr::ConstBool(false)),
      }),
    };
    assert_eq!(_parse_stmt(s), exp);
  }

  #[test]
  #[should_panic]
  fn wrong_lhs() {
    let s = "1 = x";
    _parse_stmt(s);
  }
  #[test]
  fn assign_with_if() {
    let s = "x = if true { false; } else { true; } && false; ";
    let _if = Expr::If {
      cond: Box::new(Expr::ConstBool(true)),
      tbranch: Box::new(Expr::Block {
        content: vec![Stmt::Expr(Expr::ConstBool(false))],
      }),
      fbranch: Some(Box::new(Expr::Block {
        content: vec![Stmt::Expr(Expr::ConstBool(true))],
      })),
    };
    let rhs = Expr::Binary {
      op: BinaryOp::And,
      left: Box::new(_if),
      right: Box::new(Expr::ConstBool(false)),
    };
    let exp = Stmt::Assignment {
      name_id: 0,
      rhs: Box::new(rhs),
    };
    assert_eq!(_parse_stmt(s), exp);
  }
  #[test]
  fn assign_with_var() {
    let s = "x = a;";
    let expctd = Stmt::Assignment {
      name_id: 0,
      rhs: Box::new(Expr::Var(1)),
    };
    assert_eq!(_parse_stmt(s), expctd);
  }
}
