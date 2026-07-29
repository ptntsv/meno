#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
  Add,
  Sub,
  Mul,
  Div,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
  Neg,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
  Int(i32),
  Unary {
    op: UnaryOp,
    child: Box<Expr>,
  },
  Binary {
    op: BinaryOp,
    left: Box<Expr>,
    right: Box<Expr>,
  },
}

#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
  Expr(Expr),
}
