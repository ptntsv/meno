use std::{iter::Peekable, str::Chars};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
  IntLit(i32),
  FalseLit,
  TrueLit,

  // chars
  Plus,
  Minus,
  Star,
  Slash,
  Colon,
  Semicolon,
  EQEQ,
  EQ,
  LT,
  LTE,
  GT,
  GTE,
  LParen,
  RParen,

  LogicOr,
  LogicAnd,
  BitOr,
  BitAnd,

  // types
  IntType,
  CharType,
  BoolType,
  Id(usize),

  // keywords
  Let,
  If,
  Else,
  Def,
  Unknown(char),
}

pub struct Lexer<'a> {
  strm: Peekable<Chars<'a>>,
  pub idtable: Vec<String>,
}

impl<'a> Lexer<'a> {
  pub fn new(src: &'a str) -> Self {
    Lexer {
      strm: src.chars().peekable(),
      idtable: Vec::<String>::new(),
    }
  }
  fn next_until(&mut self, predicate: impl Fn(char) -> bool) -> String {
    let mut substr = String::new();
    while let Some(ch) = self.strm.next_if(|&ch| predicate(ch)) {
      substr.push(ch);
    }
    substr
  }
  fn number(&mut self) -> String {
    self.next_until(|c: char| c.is_ascii_digit())
  }
  fn name(&mut self) -> String {
    self.next_until(|c: char| c.is_ascii_alphanumeric() || c == '_')
  }
  pub fn tokenize(&mut self) -> Vec<Token> {
    let kw_to_tok = |s: &str| match s {
      "let" => Some(Token::Let),
      "def" => Some(Token::Def),
      "if" => Some(Token::If),
      "else" => Some(Token::Else),
      "int" => Some(Token::IntType),
      "char" => Some(Token::CharType),
      "bool" => Some(Token::BoolType),
      "false" => Some(Token::FalseLit),
      "true" => Some(Token::TrueLit),
      _ => None,
    };
    let mut tokens: Vec<Token> = Vec::new();
    while let Some(ch) = self.strm.next() {
      if ch.is_whitespace() {
        continue;
      } else if ch.is_ascii_digit() {
        let mut nat = String::from(ch);
        let rest = self.number();
        nat.push_str(&rest);
        let nat: i32 = nat.parse().expect("Can't parse {int_str}");
        tokens.push(Token::IntLit(nat));
      } else if ch.is_alphabetic() {
        let mut name = String::from(ch);
        let rest = self.name();
        name.push_str(&rest);
        if let Some(tok) = kw_to_tok(&name.as_str()) {
          tokens.push(tok);
        } else {
          self.idtable.push(name);
          tokens.push(Token::Id(self.idtable.len() - 1));
        }
      } else {
        let t = match ch {
          '+' => Token::Plus,
          '-' => Token::Minus,
          '*' => Token::Star,
          '/' => Token::Slash,
          '(' => Token::LParen,
          ')' => Token::RParen,
          ';' => Token::Semicolon,
          ':' => Token::Colon,
          '=' => {
            if self.strm.next_if_eq(&'=').is_some() {
              Token::EQEQ
            } else {
              Token::EQ
            }
          }
          '<' => {
            if self.strm.next_if_eq(&'=').is_some() {
              Token::LTE
            } else {
              Token::LT
            }
          }
          '>' => {
            if self.strm.next_if_eq(&'=').is_some() {
              Token::GTE
            } else {
              Token::GT
            }
          }
          '&' => {
            if self.strm.next_if_eq(&'&').is_some() {
              Token::LogicAnd
            } else {
              Token::BitAnd
            }
          }
          '|' => {
            if self.strm.next_if_eq(&'|').is_some() {
              Token::LogicOr
            } else {
              Token::BitOr
            }
          }
          other => Token::Unknown(other),
        };
        tokens.push(t);
      }
    }
    tokens
  }
}
