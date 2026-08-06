use std::fmt::{self};

use crate::ast;

type Reg = usize;
type LabelId = usize;
type Label = String;

#[derive(Debug, Clone, PartialEq)]
pub enum IrVal {
  Reg(Reg),
  ConstInt(i32),
  ConstBool(bool),
  ConstUnit,
  Var(String),
}
impl fmt::Display for IrVal {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      &IrVal::Reg(n) => write!(f, "%{n}"),
      &IrVal::ConstInt(x) => write!(f, "{x}"),
      &IrVal::ConstBool(b) => write!(f, "{b:?}"),
      &IrVal::ConstUnit => write!(f, "()"),
      IrVal::Var(name) => write!(f, "{name}"),
    }
  }
}
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
  Add,
  Mul,
  And,
}
impl fmt::Display for BinaryOp {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      &BinaryOp::Add => write!(f, "add"),
      &BinaryOp::Mul => write!(f, "mul"),
      &BinaryOp::And => write!(f, "and"),
    }
  }
}
#[derive(Debug, Clone, PartialEq)]
pub enum IrDest {
  Reg(Reg),
  Var(String),
}
impl fmt::Display for IrDest {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      &IrDest::Reg(n) => write!(f, "%{n}"),
      IrDest::Var(name) => write!(f, "{name}"),
    }
  }
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
  Label(Label),
  Jmp(Label),
  JmpIfFalse {
    cond: IrDest,
    jmp_to: Label,
  },
}

impl fmt::Display for IrInst {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      IrInst::Label(name) => write!(f, "{name}:\n"),
      IrInst::Assign { dest, src } => write!(f, "  {dest} = {src}\n"),
      IrInst::Binary {
        dest,
        op,
        left,
        right,
      } => write!(f, "  {dest} = {op} {left} {right}\n"),
      IrInst::Jmp(label) => write!(f, "  jmp {label}\n"),
      IrInst::JmpIfFalse { cond, jmp_to } => write!(f, "  jmp_if_false {cond} {jmp_to}\n"),
    }
  }
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
  idtable: Vec<String>,
  next_reg: Reg,
  next_label: LabelId,
}

impl IrEmitter {
  pub fn new(idtable: &Vec<String>) -> IrEmitter {
    IrEmitter {
      next_reg: 0,
      idtable: idtable.clone(),
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
  fn emit_expr(&mut self, expr: &ast::Expr, instrs: &mut Vec<IrInst>) -> IrVal {
    match expr {
      ast::Expr::ConstInt(x) => IrVal::ConstInt(*x),
      ast::Expr::ConstBool(b) => IrVal::ConstBool(*b),
      ast::Expr::Var(id) => IrVal::Var(self.idtable[*id].clone()),
      ast::Expr::Binary { op, left, right } => self.emit_binary(op, left, right, instrs),
      ast::Expr::If { cond, tbr, fbr } => self.emit_if(cond, tbr, fbr, instrs),
      ast::Expr::Block { content } => self.emit_block(content, instrs),
      _ => IrVal::Reg(67),
    }
  }
  fn emit_block(&mut self, stmts: &Vec<ast::Stmt>, instrs: &mut Vec<IrInst>) -> IrVal {
    let ret_r = self.new_reg();
    let init_ret_instr = IrInst::Assign {
      dest: IrDest::Reg(ret_r.clone()),
      src: IrVal::ConstUnit,
    };
    instrs.push(init_ret_instr);
    let init = &stmts[..stmts.len() - 1];
    for stmt in init {
      self.emit_stmt(stmt, instrs);
    }
    if let Some(last) = stmts.last() {
      match last {
        ast::Stmt::Expr(e) => {
          let last_e_r = self.emit_expr(e, instrs);
          instrs.push(IrInst::Assign {
            dest: IrDest::Reg(ret_r.clone()),
            src: last_e_r,
          });
        }
        other => self.emit_stmt(other, instrs),
      }
    }
    IrVal::Reg(ret_r)
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
      src: self.emit_expr(&*cond, instrs),
    };
    instrs.push(cond_instr);
    let label_id = self.new_label();
    let else_label = format!(".else{label_id}");
    let merge_label = format!(".merge{label_id}");
    let jmp_if_false_instr = IrInst::JmpIfFalse {
      cond: cond_r.clone(),
      jmp_to: else_label.clone(),
    };
    instrs.push(jmp_if_false_instr);
    let tb_r = self.emit_expr(&**tbr, instrs);
    instrs.push(IrInst::Assign {
      dest: IrDest::Reg(ret_val_r.clone()),
      src: tb_r,
    });
    let jmp_merge_instr = IrInst::Jmp(merge_label.clone());
    instrs.push(jmp_merge_instr);
    instrs.push(IrInst::Label(else_label.clone()));
    if let Some(_fbr) = fbr {
      let fb_r = self.emit_expr(&**_fbr, instrs);
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
      left: self.emit_expr(&*left, instrs),
      right: self.emit_expr(&*right, instrs),
    };
    instrs.push(aux_inst);
    IrVal::Reg(lhs)
  }
  pub fn emit_stmt(&mut self, stmt: &ast::Stmt, instrs: &mut Vec<IrInst>) {
    let inst = match stmt {
      ast::Stmt::Assignment { name_id, rhs } => IrInst::Assign {
        dest: IrDest::Var(self.idtable[*name_id].clone()),
        src: self.emit_expr(&(**rhs), instrs),
      },
      ast::Stmt::Decl { name_id, rhs, .. } => IrInst::Assign {
        dest: IrDest::Var(self.idtable[*name_id].clone()),
        src: self.emit_expr(&(**rhs), instrs),
      },
      ast::Stmt::Expr(expr) => IrInst::Assign {
        dest: IrDest::Reg(self.new_reg()),
        src: self.emit_expr(expr, instrs),
      },
    };
    instrs.push(inst);
  }
  pub fn emit_ir(&mut self, program: &ast::Program) -> Vec<IrInst> {
    let mut ir: Vec<IrInst> = Vec::new();
    for stmt in &program.stmts {
      self.emit_stmt(stmt, &mut ir);
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
    let mut emitter = IrEmitter::new(&program.idtable);
    emitter.emit_ir(&program)
  }

  // TODO:
  // - IR generation for simple assignments [x]
  #[test]
  fn silly_ir() {
    let s = "x = 1;";
    let ir = vec![IrInst::Assign {
      dest: IrDest::Var("x".to_string()),
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
        dest: IrDest::Var("x".to_string()),
        src: IrVal::Reg(0),
      },
    ];
    assert_eq!(emit_ir(s), ir);
    let s = "x = y && 42;";
    let ir = vec![
      IrInst::Binary {
        dest: IrDest::Reg(0),
        op: BinaryOp::And,
        left: IrVal::Var("y".to_string()),
        right: IrVal::ConstInt(42),
      },
      IrInst::Assign {
        dest: IrDest::Var("x".to_string()),
        src: IrVal::Reg(0),
      },
    ];
    assert_eq!(emit_ir(s), ir);
  }
  // TODO:
  // - IR for simple conditions [x]
  #[test]
  fn cond_ir() {
    let s = "
      let x = false;
      if 1 && 2 {
        x = 2;
      } else {
        x = 3;
      };
    ";
    let ir = emit_ir(s);
    for inst in &ir {
      print!("{inst}");
    }
    assert!(1 == 2);
  }
}
