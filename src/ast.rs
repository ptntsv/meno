enum Op {
  Add, Sub, Mul, Div
}
pub enum AstNode {
}
pub enum Expr {
  Nat(i32),
  Unary {
    op: Op,
    child: Box<Expr>
  },
  Binary {
    op: Op,
    left: Box<Expr>,
    right: Box<Expr>,
  }
}
