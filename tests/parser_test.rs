use meno::{
  ast::{
    AstNode,
    BinaryOp::{Add, Mul, Sub},
    Expr::{self, Int},
    UnaryOp::Neg,
  },
  lexer::Lexer,
  parser::Parser,
};
fn parse(s: &str) -> AstNode {
  let mut lexer = Lexer::new(s);
  let lxms = &lexer.tokenize();
  let mut parser = Parser::new(lxms);
  parser.parse_program()
}

#[test]
fn int1() {
  let s = "1";
  match parse(s) {
    AstNode::Expr(Expr::Int(n)) if n == 1 => (),
    _ => panic!(),
  }
}

#[test]
fn unary_int() {
  let s = "-42";
  let expected = AstNode::Expr(Expr::Unary {
    op: Neg,
    child: Box::new(Expr::Int(42)),
  });
  assert_eq!(expected, parse(s));
  let s = "--42";
  let expected = AstNode::Expr(Expr::Unary {
    op: Neg,
    child: Box::new(Expr::Unary {
      op: Neg,
      child: Box::new(Expr::Int(42)),
    }),
  });
  assert_eq!(expected, parse(s));
  let s = "-1-2";
  let expected = AstNode::Expr(Expr::Binary {
    op: Sub,
    left: Box::new(Expr::Unary {
      op: Neg,
      child: Box::new(Expr::Int(1)),
    }),
    right: Box::new(Expr::Int(2)),
  });
  assert_eq!(expected, parse(s));
  let s = "-1--2";
  let expected = AstNode::Expr(Expr::Binary {
    op: Sub,
    left: Box::new(Expr::Unary {
      op: Neg,
      child: Box::new(Expr::Int(1)),
    }),
    right: Box::new(Expr::Unary {
      op: Neg,
      child: Box::new(Int(2)),
    }),
  });
  assert_eq!(expected, parse(s));
}

#[test]
fn unary_group() {
  let s = "-(1+2)";
  let expected = AstNode::Expr(Expr::Unary {
    op: Neg,
    child: Box::new(Expr::Binary {
      op: Add,
      left: Box::new(Expr::Int(1)),
      right: Box::new(Expr::Int(2)),
    }),
  });
  assert_eq!(expected, parse(s));
}

#[test]
fn precedence_test() {
  let s = "1+2*3";
  let expected = AstNode::Expr(Expr::Binary {
    op: Add,
    left: Box::new(Int(1)),
    right: Box::new(Expr::Binary {
      op: Mul,
      left: Box::new(Int(2)),
      right: Box::new(Int(3)),
    }),
  });
  assert_eq!(expected, parse(s));
  let s = "(1+2)*3";
  let expected = AstNode::Expr(Expr::Binary {
    op: Mul,
    left: Box::new(Expr::Binary {
      op: Add,
      left: Box::new(Int(1)),
      right: Box::new(Int(2)),
    }),
    right: Box::new(Int(3)),
  });
  assert_eq!(expected, parse(s));
  let s = "1+(2+3)";
  let expected = AstNode::Expr(Expr::Binary {
    op: Add,
    left: Box::new(Int(1)),
    right: Box::new(Expr::Binary {
      op: Add,
      left: Box::new(Int(2)),
      right: Box::new(Int(3)),
    }),
  });
  assert_eq!(expected, parse(s));
}
