use std::{iter::Peekable, str::Chars};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lexeme {
  Word(String),
  Int(u32),
  Plus,
  Minus,
  Star,
  Slash,
  Unknown(char),
}

pub struct Lexer<'a> {
  strm: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
  pub fn new(src: &'a str) -> Self {
    Lexer {
      strm: src.chars().peekable(),
    }
  }
  fn number(&mut self) -> String {
    let mut num_str = String::new();
    while let Some(ch) = self.strm.next_if(|ch| ch.is_digit(10)) {
      num_str.push(ch);
    }
    num_str
  }
  fn word(&mut self) -> Option<String> {
    None
  }
  pub fn lex(&mut self) -> Vec<Lexeme> {
    let mut tokens: Vec<Lexeme> = Vec::new();
    while let Some(ch) = self.strm.next() {
      if ch.is_whitespace() {
        continue;
      } else if ch.is_ascii_digit() {
        let mut nat = String::from(ch);
        let rest = self.number();
        nat.push_str(&rest);
        let nat: u32 = nat.parse().expect("Can't parse {int_str}");
        tokens.push(Lexeme::Int(nat));
      } else if ch.is_alphabetic() {
        // tokens.push(self.word());
      } else if ch == '+' {
        tokens.push(Lexeme::Plus);
      } else if ch == '-' {
        tokens.push(Lexeme::Minus);
      } else if ch == '*' {
        tokens.push(Lexeme::Star);
      } else if ch == '/' {
        tokens.push(Lexeme::Slash);
      } else {
        tokens.push(Lexeme::Unknown(ch));
      }
    }
    tokens
  }
}
