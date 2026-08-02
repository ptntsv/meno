use crate::ast::Program;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
  Int, Bool, Char
}

#[derive(Debug, Clone)]
pub struct TypeChecker {
  program: Program
}
