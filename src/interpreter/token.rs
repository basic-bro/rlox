/////////////////////////////////////////////
// private module rlox::interpreter::token //
/////////////////////////////////////////////


//////////////////////
// public interface //
//////////////////////

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenType<'a> {
  LeftParen, RightParen, LeftBrace, RightBrace,
  Comma, Dot, Minus, Plus, Semicolon, Slash, Star,

  Bang, BangEqual, Equal, EqualEqual,
  Greater, GreaterEqual, Less, LessEqual,

  Identifer( &'a str ), String( &'a str ), Number( &'a str ),

  And, Class, Else, False, Fun, For, If, Nil, Or,
  Print, Return, Super, This, True, Var, While
}

impl<'a> TokenType<'a> {

  pub fn get_lexeme( &self ) -> &str {
    match self {
      Self::LeftParen => "(",
      Self::RightParen => ")",
      Self::LeftBrace => "{",
      Self::RightBrace => "}",
      Self::Comma => ",",
      Self::Dot => ".",
      Self::Minus => "-",
      Self::Plus => "+",
      Self::Semicolon => ":",
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
      Self::Identifer( id ) => *id,
      Self::String( s ) => *s,
      Self::Number( f ) => *f,
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
      Self::While => "while"
    }
  }

}

#[derive(Debug, Clone, Copy)]
pub struct Token<'a> {
  token_type: TokenType<'a>,
  line: i32
}

impl<'a> Token<'a> {
  pub fn new( token_type: TokenType<'a>, line: i32 ) -> Token<'a> {
    Token {
      token_type,
      line
    }
  }

  pub fn get_lexeme( &self ) -> &str {
    self.token_type.get_lexeme()
  }

  pub fn get_token_type( &self ) -> &TokenType<'a> {
    &self.token_type
  }
}