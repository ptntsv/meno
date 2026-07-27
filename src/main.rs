use crate::lexer::Lexer;
use crate::parser::Parser;
use std::env;
use std::fs;

pub mod ast;
pub mod lexer;
pub mod parser;

fn main() {
  let args: Vec<String> = env::args().collect();
  let src_path = args.get(1).expect("Provide source");
  let content = fs::read_to_string(src_path).expect("Should have been able to read the file");
  println!("With text:\n{content}");
  let mut lexer = Lexer::new(&content);
  let lxms = lexer.lex();
  for l in lxms {
    println!("{:?}", l);
  }
}
