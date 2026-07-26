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

pub struct Lexer {
  str: String,
  idx: usize,
}

impl Lexer {
  pub fn new(src: &str) -> Self {
    Lexer {
      str: src.to_owned(),
      idx: 0,
    }
  }
  fn consume(&mut self) -> Option<char> {
    self.idx += 1;
    self.str.chars().nth(self.idx - 1)
  }
  fn refuse(&mut self) {
    self.idx -= 1
  }
  fn number(&mut self) -> Option<u32> {
    let mut val: u32 = 0;
    let mut any: bool = false;
    while let Some(ch) = self.consume() {
      if let Some(digit) = ch.to_digit(10) {
        val = val * 10 + digit;
        any = true;
      } else {
        self.refuse();
        break
      }
    }
    any.then_some(val)
  }
  fn word(&mut self) -> Option<String> {
    None
  }
  pub fn lex(&mut self) -> Vec<Lexeme> {
    let mut tokens: Vec<Lexeme> = Vec::new();
    while let Some(ch) = self.consume() {
      if ch.is_whitespace() {
        continue;
      } else if ch.is_ascii_digit() {
        self.refuse();
        if let Some(n) = self.number() {
          tokens.push(Lexeme::Int(n));
        }
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
