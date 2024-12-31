///////////////////////////////////////////////
// private module rlox::interpreter::scanner //
///////////////////////////////////////////////


/////////
// use //
/////////

use crate::util::*;
use crate::interpreter::token::*;


//////////////////////
// public interface //
//////////////////////

pub struct Scanner<'str> {
  db: &'str mut StringManager,
  src: String,
  tokens: Vec<Token>,
  start: usize,
  current: usize,
  line: i32
}

impl<'str> Scanner<'str> {

  pub fn keyword( value: &str ) -> Option<TokenType> {
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

  pub fn new( db: &'str mut StringManager ) -> Scanner<'str> {
    Scanner {
      db,
      src: "".to_string(),
      tokens: vec![],
      start: 0,
      current: 0,
      line: 1
    }
  }

  pub fn scan_tokens( &mut self, src: String ) -> Vec<Token> {
    self.restart( src );
    while !self.is_at_end() {
      self.start = self.current;
      self.scan_token();
    }
    let tokens = self.tokens.clone();
    self.tokens.clear();
    tokens
  }


  ////////////////////////////
  // private implementation //
  ////////////////////////////
   
  fn restart( &mut self, src: String ) {
    self.src = src;
    self.tokens.clear();
    self.start = 0;
    self.current = 0;
    self.line = 1;
  }

  fn scan_token( &mut self ) {
    match self.advance() {
      '(' => self.add_token( TokenType::LeftParen ),
      ')' => self.add_token( TokenType::RightParen ),
      '{' => self.add_token( TokenType::LeftBrace ),
      '}' => self.add_token( TokenType::RightBrace ),
      ',' => self.add_token( TokenType::Comma ),
      '.' => self.add_token( TokenType::Dot ),
      '-' => self.add_token( TokenType::Minus ),
      '+' => self.add_token( TokenType::Plus ),
      ';' => self.add_token( TokenType::Semicolon ),
      '*' => self.add_token( TokenType::Star ),
      '!' => self.double_char_token( '=', TokenType::BangEqual, TokenType::Bang ),
      '=' => self.double_char_token( '=', TokenType::EqualEqual, TokenType::Equal ),
      '<' => self.double_char_token( '=', TokenType::LessEqual, TokenType::Less ),
      '>' => self.double_char_token( '=', TokenType::GreaterEqual, TokenType::Greater ),
      '/' => if self.advance_if( '/' ) {
               while self.peek() != '\n' && !self.is_at_end() {
                 self.advance();
               }
             } else {
               self.add_token( TokenType::Slash );
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
                   Self::error( self.line, String::from( format!( "Unexpected character: '{}'", c ) ) )
                 }          
    }
  }

  fn double_char_token( &mut self, second_char: char, double_token: TokenType, single_token: TokenType ) {
    let did_advance = self.advance_if( second_char );
    self.add_token( ifte( did_advance, double_token, single_token ) );
  }

  fn string( &mut self ) {
    while self.peek() != '"' && !self.is_at_end() {
      if self.peek() == '\n' {
        self.line += 1;
      }
      self.advance();
    }

    if self.is_at_end() {
      Self::error( self.line, String::from( "Unterminated string." ) );
      return;
    }

    self.advance();

    let value = substring( &self.src, self.start + 1, self.current - self.start - 2 ).unwrap();
    let stored_string = self.db.puts( value );
    self.add_token( TokenType::String( stored_string ) );
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
    let stored_string = self.db.puts( value );
    self.add_token( TokenType::Number( stored_string ) );
  }

  fn identifer( &mut self ) {
    while is_alphanumeric( self.peek() ) {
      self.advance();
    }

    let value = substring( &self.src, self.start, self.current - self.start ).unwrap();
    let stored_string = self.db.puts( value );

    match Scanner::keyword( value ) {
      Some( tt ) => self.add_token( tt ),
      None => self.add_token( TokenType::Identifer( stored_string ) ),
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

  fn add_token( &mut self, token_type: TokenType ) {
    self.tokens.push(
      Token::new(
        token_type,
        self.line
      )
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

  fn report( line: i32, where_: String, message: String ) {
    eprintln!( "[line {}] Error{}: {}", line, where_, message );
  }

  fn error( line: i32, message: String ) {
    Self::report( line, "".to_string(), message );
  }
  
}