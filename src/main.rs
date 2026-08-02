use meno::parser::Parser;
use meno::lexer::Lexer;
use std::env;
use std::fs;


fn main() {
  let args: Vec<String> = env::args().collect();
  let src_path = args.get(1).expect("Provide source");
  let content = fs::read_to_string(src_path).expect("Should have been able to read the file");
  println!("With text:\n{content}");
  let mut lexer = Lexer::new(&content);
  let lxms = lexer.tokenize();
  let parser = Parser::new(&lxms, lexer.idtable);
  let ast = parser.parse_program();
  println!("{:?}", ast);
}
