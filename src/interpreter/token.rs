/////////////////////////////////////////////
// private module rlox::interpreter::token //
/////////////////////////////////////////////


/////////
// use //
/////////

use crate::util::*;

//////////////////////
// public interface //
//////////////////////

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenType {
  LeftParen, RightParen, LeftBrace, RightBrace,
  Comma, Dot, Minus, Plus, Semicolon, Slash, Star,

  Bang, BangEqual, Equal, EqualEqual,
  Greater, GreaterEqual, Less, LessEqual,

  Identifier( StringKey ), String( StringKey ), Number( StringKey ),

  And, Class, Else, False, Fun, For, If, Nil, Or,
  Print, Return, Super, This, True, Var, While,

  Eof
}

impl TokenType {

  pub fn get_lexeme<'str>( &self, sm: &'str StringManager ) -> &'str str {
    match self {
      Self::LeftParen => "(",
      Self::RightParen => ")",
      Self::LeftBrace => "{",
      Self::RightBrace => "}",
      Self::Comma => ",",
      Self::Dot => ".",
      Self::Minus => "-",
      Self::Plus => "+",
      Self::Semicolon => ";",
      Self::Slash => "/",
      Self::Star => "*",
      Self::Bang => "!",
      Self::BangEqual => "!=",
      Self::Equal => "=",
      Self::EqualEqual => "==",
      Self::Greater => ">",
      Self::GreaterEqual => ">=",
      Self::Less => "<",
      Self::LessEqual => "<=",
      Self::Identifier( id ) => sm.gets( *id ),
      Self::String( s ) => sm.gets( *s ),
      Self::Number( f ) => sm.gets( *f ),
      Self::And => "and",
      Self::Class => "class",
      Self::Else => "else",
      Self::False => "false",
      Self::Fun => "fun",
      Self::For => "for",
      Self::If => "if",
      Self::Nil => "nil",
      Self::Or => "or",
      Self::Print => "print",
      Self::Return => "return",
      Self::Super => "super",
      Self::This => "this",
      Self::True => "true",
      Self::Var => "var",
      Self::While => "while",
      Self::Eof => "[end-of-file]"
    }
  }

  pub fn get_key( &self ) -> StringKey {
    match self {
      Self::Identifier( identifier ) => *identifier,
      Self::String( string ) => *string,
      Self::Number( number ) => *number,
      _ => panic!( "The caller of get_key() assumes responsibility for checking that the given token has a key." )
    }
  }

}

#[derive(Debug, Clone, Copy)]
pub struct Token {
  token_type: TokenType,
  line: i32
}

impl Token {

  pub fn new( token_type: TokenType, line: i32 ) -> Token {
    Token {
      token_type,
      line
    }
  }

  pub fn get_lexeme<'str>( &self, sm: &'str StringManager ) -> &'str str {
    self.token_type.get_lexeme( sm )
  }

  pub fn get_type( &self ) -> &TokenType {
    &self.token_type
  }

  pub fn get_line( &self ) -> i32 {
    self.line
  }

  pub fn get_key( &self ) -> StringKey {
    self.token_type.get_key()
  }
  
}