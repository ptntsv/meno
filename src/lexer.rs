use std::collections::{HashMap, HashSet};
use std::{iter::Peekable, str::Chars};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
  IntLit(i32),
  // chars
  Plus,
  Minus,
  Star,
  Slash,
  Colon,
  Semicolon,
  EQ,
  LT,
  LTE,
  GT,
  GTE,
  // types
  IntType,
  CharType,
  BoolType,
  Id(usize),
  // Keyword(String),
  // keywords
  Let,
  If,
  Else,
  Def,
  LParen,
  RParen,
  Unknown(char),
}

pub struct Lexer<'a> {
  strm: Peekable<Chars<'a>>,
  symtable: Vec<String>,
}

impl<'a> Lexer<'a> {
  pub fn new(src: &'a str) -> Self {
    Lexer {
      strm: src.chars().peekable(),
      symtable: Vec::<String>::new(),
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
      other => None,
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
          self.symtable.push(name);
          tokens.push(Token::Id(self.symtable.len() - 1));
        }
      } else if ch == '+' {
        tokens.push(Token::Plus);
      } else if ch == '-' {
        tokens.push(Token::Minus);
      } else if ch == '*' {
        tokens.push(Token::Star);
      } else if ch == '/' {
        tokens.push(Token::Slash);
      } else if ch == '(' {
        tokens.push(Token::LParen);
      } else if ch == ')' {
        tokens.push(Token::RParen);
      } else if ch == '=' {
        tokens.push(Token::EQ);
      } else if ch == ';' {
        tokens.push(Token::Semicolon);
      } else if ch == ':' {
        tokens.push(Token::Colon);
      } else {
        tokens.push(Token::Unknown(ch));
      }
    }
    tokens
  }
}
