use crate::ast;

type Reg = usize;
type LabelId = usize;

#[derive(Debug, Clone, PartialEq)]
pub enum IrVal {
  Reg(Reg),
  ConstInt(i32),
  ConstBool(bool),
  ConstUnit,
  Var(usize),
}
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
  Add,
  Mul,
  And,
}
#[derive(Debug, Clone, PartialEq)]
pub enum IrDest {
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
  Label(LabelId),
  Jmp(LabelId),
  JmpIfFalse {
    cond: IrDest,
    jmp_to: LabelId,
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
  next_label: LabelId,
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
  fn new_label(&mut self) -> LabelId {
    self.next_label += 1;
    self.next_label - 1
  }
  fn emit(&mut self, expr: &ast::Expr, instrs: &mut Vec<IrInst>) -> IrVal {
    match expr {
      ast::Expr::ConstInt(x) => IrVal::ConstInt(*x),
      ast::Expr::ConstBool(b) => IrVal::ConstBool(*b),
      ast::Expr::Var(id) => IrVal::Var(*id),
      ast::Expr::Binary { op, left, right } => self.emit_binary(op, left, right, instrs),
      ast::Expr::If { cond, tbr, fbr } => self.emit_if(cond, tbr, fbr, instrs),
      _ => IrVal::Reg(67),
    }
  }
  fn emit_if(
    &mut self,
    cond: &Box<ast::Expr>,
    tbr: &Box<ast::Expr>,
    fbr: &Option<Box<ast::Expr>>,
    instrs: &mut Vec<IrInst>,
  ) -> IrVal {
    let ret_val_r = self.new_reg();
    let cond_r = IrDest::Reg(self.new_reg());
    let cond_instr = IrInst::Assign {
      dest: cond_r.clone(),
      src: self.emit(&*cond, instrs),
    };
    instrs.push(cond_instr);
    let else_label = self.new_label();
    let merge_label = self.new_label();
    let jmp_if_false_instr = IrInst::JmpIfFalse {
      cond: cond_r.clone(),
      jmp_to: else_label.clone(),
    };
    instrs.push(jmp_if_false_instr);
    let tb_r = self.emit(&**tbr, instrs);
    instrs.push(IrInst::Assign {
      dest: IrDest::Reg(ret_val_r.clone()),
      src: tb_r,
    });
    let jmp_merge_instr = IrInst::Jmp(merge_label);
    instrs.push(jmp_merge_instr);
    instrs.push(IrInst::Label(else_label.clone()));
    if let Some(_fbr) = fbr {
      let fb_r = self.emit(&**_fbr, instrs);
      instrs.push(IrInst::Assign {
        dest: IrDest::Reg(ret_val_r.clone()),
        src: fb_r,
      });
    } else {
    }
    instrs.push(IrInst::Label(merge_label.clone()));
    IrVal::Reg(ret_val_r)
  }
  fn emit_binary(
    &mut self,
    op: &ast::BinaryOp,
    left: &Box<ast::Expr>,
    right: &Box<ast::Expr>,
    instrs: &mut Vec<IrInst>,
  ) -> IrVal {
    let op_map = |tok: &ast::BinaryOp| match tok {
      ast::BinaryOp::Add => BinaryOp::Add,
      ast::BinaryOp::Mul => BinaryOp::Mul,
      ast::BinaryOp::And => BinaryOp::And,
      _ => todo!(),
    };
    let lhs = self.new_reg();
    let aux_inst = IrInst::Binary {
      dest: IrDest::Reg(lhs),
      op: op_map(&op),
      left: self.emit(&*left, instrs),
      right: self.emit(&*right, instrs),
    };
    instrs.push(aux_inst);
    IrVal::Reg(lhs)
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

  // TODO:
  // - IR generation for simple assignments [x]
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
  // TODO:
  // - IR generation for exprs [x]
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
  #[test]
  fn cond_ir() {
    let s = "
      let x = false;
      if x {
        x = 2;
      } else {
        x = 3;
      };
    ";
    let x = emit_ir(s);
    println!("{x:?}");
    assert!(1 == 2);
  }
}
