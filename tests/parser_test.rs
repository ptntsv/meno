use meno::{
  ast::{Expr, Program, Stmt},
  lexer::Lexer,
  parser::Parser,
};

fn parse(s: &str) -> Program {
  let mut lexer = Lexer::new(s);
  let lxms = &lexer.tokenize();
  let mut parser = Parser::new(lxms);
  parser.parse_program()
}
#[test]
fn two_decls() {
  let program = "let x = 1; let y = 2;";
  let program = parse(program);
  for stmt in &program.stmts {
    println!("{stmt:?}");
  }
  let expect = vec![
    Stmt::Assignment {
      name_id: 0,
      rhs: Box::new(Expr::Int(1)),
    },
    Stmt::Assignment {
      name_id: 1,
      rhs: Box::new(Expr::Int(2)),
    },
  ];
  assert_eq!(program.stmts, expect);
}
