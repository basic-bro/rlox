

#[derive(Clone, Copy, PartialEq, Hash, Eq)]
pub enum TokenType {
  LeftParen, RightParen, LeftBrace, RightBrace,
  Comma, Dot, Minus, Plus, Semicolon, Slash, Star,

  Bang, BangEqual, Equal, EqualEqual,
  Greater, GreaterEqual, Less, LessEqual,

  Identifier, String, Number,

  And, Class, Else, False, Fun, For, If, Nil, Or,
  Print, Return, Super, This, True, Var, While,

  Eof
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Token {
  pub token_type: TokenType,
  pub lexeme: String,
  //pub literal: Eval
  pub line: u32
}


impl TokenType {
  pub fn get_lexeme( &self ) -> &str {
    match self {
      TokenType::LeftParen => "(",
      TokenType::RightParen => ")",
      TokenType::LeftBrace => "{",
      TokenType::RightBrace => "}",
      TokenType::Comma => ",",
      TokenType::Dot => ".",
      TokenType::Minus => "-",
      TokenType::Plus => "+",
      TokenType::Semicolon => ";",
      TokenType::Slash => "/",
      TokenType::Star => "*",
      TokenType::Bang => "!",
      TokenType::BangEqual => "!=",
      TokenType::Equal => "=",
      TokenType::EqualEqual => "==",
      TokenType::Greater => ">",
      TokenType::GreaterEqual => ">=",
      TokenType::Less => "<",
      TokenType::LessEqual => "<=",
      TokenType::Identifier => "[id]",
      TokenType::String => "[string]",
      TokenType::Number => "[number]",
      TokenType::And => "and",
      TokenType::Class => "class",
      TokenType::Else => "else",
      TokenType::False => "false",
      TokenType::Fun => "fun",
      TokenType::For => "for",
      TokenType::If => "if",
      TokenType::Nil => "nil",
      TokenType::Or => "or",
      TokenType::Print => "print",
      TokenType::Return => "return",
      TokenType::Super => "super",
      TokenType::This => "this",
      TokenType::True => "true",
      TokenType::Var => "var",
      TokenType::While => "while",
      TokenType::Eof => "[EOF]",
    }
  }
}