#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lexeme {
  Char(char),
  Digit(char),
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
  fn end(&self) -> bool {
    self.idx >= self.str.len()
  }
  pub fn new(src: &str) -> Self {
    Lexer { str: src.to_owned(), idx: 0 }
  }
  fn next_char(&mut self) -> Option<char> {
    self.idx += 1;
    self.str.chars().nth(self.idx - 1)
  }
  pub fn tokenize(&mut self) -> Vec<Lexeme> {
    let mut tokens: Vec<Lexeme> = Vec::new();
    while !self.end() {
      let maybe_ch = self.next_char();
      if let Some(ch) = maybe_ch {
        if ch.is_whitespace() {
          continue;
        } else if ch.is_digit(10) {
          tokens.push(Lexeme::Digit(ch));
        } else if ch.is_alphabetic() {
          tokens.push(Lexeme::Char(ch));
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
      } else {
        println!("Something went wrong, str: {}, idx: {}", self.str, self.idx);
        break;
      }
    }
    tokens
  }
}
