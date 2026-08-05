use crate::ast;

type Reg = usize;
type Label = usize;

#[derive(Debug, Clone, PartialEq)]
enum IrVal {
  Reg(Reg),
  ConstInt(i32),
  Var(usize),
  ConstBool(bool),
}
#[derive(Debug, Clone, PartialEq)]
enum BinaryOp {
  Add,
  Mul,
  And,
}
#[derive(Debug, Clone, PartialEq)]
enum IrDest {
  Reg(Reg),
  Var(usize),
}
#[derive(Debug, Clone, PartialEq)]
pub enum IrInst {
  Binary {
    dest: IrDest,
    op: BinaryOp,
    left: IrVal,
    right: IrVal,
  },
  Assign {
    dest: IrDest,
    src: IrVal,
  },
  Jmp(Label),
  CondJmp {
    cond: Reg,
    tlabel: Label,
    flabel: Label,
  },
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrFunction {
  pub instr: Vec<IrInst>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrProgram {
  pub functions: Vec<IrFunction>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrEmitter {
  next_reg: Reg,
  next_label: Label,
}

impl IrEmitter {
  pub fn new() -> IrEmitter {
    IrEmitter {
      next_reg: 0,
      next_label: 0,
    }
  }
  fn new_reg(&mut self) -> Reg {
    self.next_reg += 1;
    self.next_reg - 1
  }
  fn new_label(&mut self) -> Label {
    self.next_label += 1;
    self.next_label - 1
  }
  fn emit(&mut self, expr: &ast::Expr, instrs: &mut Vec<IrInst>) -> IrVal {
    let node2op = |tok: &ast::BinaryOp| match tok {
      ast::BinaryOp::Add => BinaryOp::Add,
      ast::BinaryOp::Mul => BinaryOp::Mul,
      ast::BinaryOp::And => BinaryOp::And,
      _ => todo!(),
    };
    match expr {
      ast::Expr::ConstInt(x) => IrVal::ConstInt(*x),
      ast::Expr::Var(id) => IrVal::Var(*id),
      ast::Expr::Binary { op, left, right } => {
        let lhs = self.new_reg();
        let aux_inst = IrInst::Binary {
          dest: IrDest::Reg(lhs),
          op: node2op(&op),
          left: self.emit(&*left, instrs),
          right: self.emit(&*right, instrs),
        };
        instrs.push(aux_inst);
        IrVal::Reg(lhs)
      }
      _ => IrVal::Reg(67),
    }
  }
  pub fn emit_ir(&mut self, program: &ast::Program) -> Vec<IrInst> {
    let mut ir: Vec<IrInst> = Vec::new();
    for stmt in &program.stmts {
      let inst = match stmt {
        ast::Stmt::Assignment { name_id, rhs } => IrInst::Assign {
          dest: IrDest::Var(*name_id),
          src: self.emit(&(**rhs), &mut ir),
        },
        ast::Stmt::Decl { name_id, rhs, .. } => IrInst::Assign {
          dest: IrDest::Var(*name_id),
          src: self.emit(&(**rhs), &mut ir),
        },
        ast::Stmt::Expr(expr) => IrInst::Assign {
          dest: IrDest::Reg(self.new_reg()),
          src: self.emit(expr, &mut ir),
        },
      };
      ir.push(inst);
    }
    ir
  }
}

// ========= UNIT-TESTS =========

#[cfg(test)]
mod tests {
  use crate::{lexer::Lexer, parser::Parser};

  use super::*;
  fn emit_ir(s: &str) -> Vec<IrInst> {
    let mut lexer = Lexer::new(s);
    let lxms = &lexer.tokenize();
    let parser = Parser::new(lxms, lexer.idtable);
    let program = parser.parse_program();
    let mut emitter = IrEmitter::new();
    emitter.emit_ir(&program)
  }

  #[test]
  fn silly_ir() {
    let s = "x = 1;";
    let ir = vec![IrInst::Assign {
      dest: IrDest::Var(0),
      src: IrVal::ConstInt(1),
    }];
    assert_eq!(emit_ir(s), ir);
    let s = "let x = 1;";
    assert_eq!(emit_ir(s), ir);
    let s = "42;";
    let ir = vec![IrInst::Assign {
      dest: IrDest::Reg(0),
      src: IrVal::ConstInt(42),
    }];
    assert_eq!(emit_ir(s), ir);
  }
  #[test]
  fn silly_exprs_ir() {
    let s = "x = 1 + 2;";
    let ir = vec![
      IrInst::Binary {
        dest: IrDest::Reg(0),
        op: BinaryOp::Add,
        left: IrVal::ConstInt(1),
        right: IrVal::ConstInt(2),
      },
      IrInst::Assign {
        dest: IrDest::Var(0),
        src: IrVal::Reg(0),
      },
    ];
    assert_eq!(emit_ir(s), ir);
    let s = "x = y && 42;";
    let ir = vec![
      IrInst::Binary {
        dest: IrDest::Reg(0),
        op: BinaryOp::And,
        left: IrVal::Var(1),
        right: IrVal::ConstInt(42),
      },
      IrInst::Assign {
        dest: IrDest::Var(0),
        src: IrVal::Reg(0),
      },
    ];
    assert_eq!(emit_ir(s), ir);
  }
  // TODO:
  // - IR generation for simple assignments [x]
  // - IR generation for exprs [x]
}
