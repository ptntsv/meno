use meno::{
  ast::{Expr, Program, Stmt},
  lexer::Lexer,
  parser::Parser,
  type_checker::Type,
};

fn parse(s: &str) -> Program {
  let mut lexer = Lexer::new(s);
  let lxms = &lexer.tokenize();
  let parser = Parser::new(lxms, lexer.idtable);
  parser.parse_program()
}
#[test]
fn two_decls() {
  let program = "let x = 1; let y:bool = false;";
  let program = parse(program);
  let expect = vec![
    Stmt::Decl {
      name_id: 0,
      decl_type: None,
      rhs: Box::new(Expr::ConstInt(1)),
    },
    Stmt::Decl {
      name_id: 1,
      decl_type: Some(Type::Bool),
      rhs: Box::new(Expr::ConstBool(false)),
    },
  ];
  assert_eq!(program.stmts, expect);
}
