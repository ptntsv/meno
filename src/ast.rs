use crate::type_checker::Type;

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
  Add,
  Sub,
  Mul,
  Div,
  Lt,
  Lte,
  Gt,
  Gte,
  Eq,
  Or,
  And,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
  Neg,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
  ConstInt(i32),
  ConstBool(bool),
  Var(usize),
  Unary {
    op: UnaryOp,
    child: Box<Expr>,
  },
  Binary {
    op: BinaryOp,
    left: Box<Expr>,
    right: Box<Expr>,
  },
  Block {
    content: Vec<Stmt>,
  },
  If {
    cond: Box<Expr>,
    tbranch: Box<Expr>,
    fbranch: Option<Box<Expr>>,
  },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
  Decl {
    name_id: usize,
    decl_type: Option<Type>,
    rhs: Box<Expr>,
  },
  Assignment {
    name_id: usize,
    rhs: Box<Expr>,
  },
  Expr(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
  pub stmts: Vec<Stmt>,
  pub idtable: Vec<String>,
}
