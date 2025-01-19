///////////////////////////////////////////////
// private module rlox::interpreter::scanner //
///////////////////////////////////////////////


/////////
// use //
/////////

use crate::util::{is_alpha, is_alphanumeric, is_digit, ifte, char_at, substring};
use crate::token::{TokenType, Token};


//////////////////
// declarations //
//////////////////

pub struct Scanner {
  src: String,
  tokens: Vec<Token>,
  start: usize,
  current: usize,
  line: u32,
  had_error: bool
}

/////////////////////
// implementations //
/////////////////////

impl Scanner {
  pub fn new() -> Scanner {
    Scanner {
      src: "".into(),
      tokens: Vec::new(),
      start: 0,
      current: 0,
      line: 1,
      had_error: false
    }
  }
  pub fn scan( &mut self, src: String ) -> ( Vec<Token>, bool ) {
    self.restart( src );
    while !self.is_at_end() {
      self.start = self.current;
      self.scan_token();
    }
    self.tokens.push( Token{ token_type: TokenType::Eof, lexeme: "[EOF]".into(), line: self.line } );
    let tokens = self.tokens.clone();
    self.tokens.clear();
    ( tokens, self.had_error )
  }
  fn restart( &mut self, src: String ) {
    self.src = src;
    self.tokens.clear();
    self.start = 0;
    self.current = 0;
    self.line = 1;
    self.had_error = false
  }
  fn scan_token( &mut self ) {
    match self.advance() {
      '(' => self.add_token( TokenType::LeftParen, "(" ),
      ')' => self.add_token( TokenType::RightParen, ")" ),
      '{' => self.add_token( TokenType::LeftBrace, "{" ),
      '}' => self.add_token( TokenType::RightBrace, "}" ),
      ',' => self.add_token( TokenType::Comma, "," ),
      '.' => self.add_token( TokenType::Dot, "." ),
      '-' => self.add_token( TokenType::Minus, "-" ),
      '+' => self.add_token( TokenType::Plus, "+" ),
      ';' => self.add_token( TokenType::Semicolon, ";" ),
      '*' => self.add_token( TokenType::Star, "*" ),
      '!' => self.double_char_token( '!', '=', TokenType::BangEqual, TokenType::Bang ),
      '=' => self.double_char_token( '=', '=', TokenType::EqualEqual, TokenType::Equal ),
      '<' => self.double_char_token( '<', '=', TokenType::LessEqual, TokenType::Less ),
      '>' => self.double_char_token( '>', '=', TokenType::GreaterEqual, TokenType::Greater ),
      '/' => if self.advance_if( '/' ) {
               while self.peek() != '\n' && !self.is_at_end() {
                 self.advance();
               }
             } else {
               self.add_token( TokenType::Slash, "/" );
             },
      ' ' => {},
      '\r' => {},
      '\t' => {},
      '\n' => self.line += 1,
      '"' => self.string(),
      c => if is_digit( c ) {
                   self.number();
                 } else if is_alpha( c ) {
                   self.identifer();
                 } else {
                   self.emit_error( &format!( " at '{}'", c ), "Unexpected character." )
                 }          
    }
  }
  fn double_char_token( &mut self, first_char: char, second_char: char, double_token: TokenType, single_token: TokenType ) {
    let did_advance = self.advance_if( second_char );
    self.add_token(
      ifte( did_advance, double_token, single_token ),
      ifte( did_advance, format!( "{}{}", first_char, second_char ).as_str(), format!( "{}", first_char ).as_str() )
    );
  }
  fn string( &mut self ) {
    let begin = self.line;
    while self.peek() != '"' && !self.is_at_end() {
      if self.peek() == '\n' {
        self.line += 1;
      }
      self.advance();
    }
    if self.is_at_end() {
      self.emit_error( " at end of file", &format!( "Unterminated string. (The string started on line {}.)", begin ) );
      return;
    }
    self.advance();
    let value = substring( &self.src, self.start + 1, self.current - self.start - 2 ).unwrap();
    self.add_token( TokenType::String, value.to_owned().as_str() );
  }
  fn number( &mut self ) {
    while is_digit( self.peek() ) {
      self.advance();
    }
    if self.peek() == '.' && is_digit( self.peek_next() ) {
      self.advance();
      while is_digit( self.peek() ) {
        self.advance();
      }
    }
    let value = substring( &self.src, self.start, self.current - self.start ).unwrap();
    self.add_token( TokenType::Number, value.to_owned().as_str() );
  }
  fn identifer( &mut self ) {
    while is_alphanumeric( self.peek() ) {
      self.advance();
    }
    let value = substring( &self.src, self.start, self.current - self.start ).unwrap();
    match Scanner::keyword( value ) {
      Some( tt ) => self.add_token( tt, value.to_owned().as_str() ),
      None => self.add_token( TokenType::Identifier, value.to_owned().as_str() ),
    }
  }
  fn keyword( value: &str ) -> Option<TokenType> {
    match value {
      "and" => Some( TokenType::And ),
      "class" => Some( TokenType::Class ),
      "else" => Some( TokenType::Else ),
      "false" => Some( TokenType::False ),
      "for" => Some( TokenType::For ),
      "fun" => Some( TokenType::Fun ),
      "if" => Some( TokenType::If ),
      "nil" => Some( TokenType::Nil ),
      "or" => Some( TokenType::Or ),
      "print" => Some( TokenType::Print ),
      "return" => Some( TokenType::Return ),
      "super" => Some( TokenType::Super ),
      "this" => Some( TokenType::This ),
      "true" => Some( TokenType::True ),
      "var" => Some( TokenType::Var ),
      "while" => Some( TokenType::While ),
      _ => None
    }
  }
  fn advance( &mut self ) -> char {
    self.current += 1;
    char_at( &self.src, self.current - 1 ).unwrap()
  }
  fn advance_if( &mut self, expected: char ) -> bool {
    if self.is_at_end() {
      return false;
    }
    if char_at( &self.src, self.current ).unwrap() != expected {
      return false;
    }
    self.current += 1;
    true
  }
  fn add_token( &mut self, token_type: TokenType, lexeme: &str ) {
    self.tokens.push(
      Token {
        token_type,
        lexeme: lexeme.into(),
        line: self.line
      }
    );
  }
  fn peek( &self ) -> char {
    if self.is_at_end() {
      '\0'
    } else {
      char_at( &self.src, self.current ).unwrap()
    }
  }
  fn peek_next( &self ) -> char {
    if self.current + 1 >= self.src.len() {
      '\0'
    } else {
      char_at( &self.src, self.current + 1 ).unwrap()
    }
  }
  fn is_at_end( &self ) -> bool {
    self.current >= self.src.len()
  }
  fn emit_error( &mut self, loc: &str, message: &str ) {
    eprintln!( "[line {}] Error{}: {}", self.line, loc, message );
    self.had_error = true;
  }
}