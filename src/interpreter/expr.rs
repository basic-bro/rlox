////////////////////////////////////////////
// private module rlox::interpreter::expr //
////////////////////////////////////////////


/////////
// use //
/////////

use std::fmt::Display;

use crate::interpreter::token::*;


//////////////////////
// public interface //
//////////////////////

#[derive(Debug)]
pub enum Expr<'src> {
  Binary( Box<Expr<'src>> /* left */, Token<'src> /* operator */, Box<Expr<'src>> /* right */ ),
  Grouping( Box<Expr<'src>> ),
  Literal( Token<'src> /* identifier | string | number */ ),
  Unary( Token<'src> /* operator */, Box<Expr<'src>> )
}


////////////////////////////
// private implementation //
////////////////////////////

impl<'src> Display for Expr<'src> {
  fn fmt( &self, f: &mut std::fmt::Formatter<'_> ) -> std::fmt::Result {
    match self {
      Self::Binary( left, op , right ) => write!( f, "{} {} {}", left, op.get_lexeme(), right ),
      Self::Grouping( expr ) => write!( f, "( {} )", expr ),
      Self::Literal( token ) => {
        match token.get_token_type() {
          TokenType::String( s ) => write!( f, "\"{}\"", s ),
                                      _ => write!( f, "{}", token.get_lexeme() )
        }
      },
      Self::Unary( op, right ) => write!( f, "{}{}", op.get_lexeme(), right )
    }
  }
}