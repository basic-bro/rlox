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

pub trait ExprVisitor<'src, T, E> {
  fn visit_binary( left: T, op: &Token<'src>, right: T ) -> Result<T, E>;
  fn visit_grouping( expr: T ) -> Result<T, E>;
  fn visit_literal( literal: &Token<'src> ) -> Result<T, E>;
  fn visit_unary( op: &Token<'src>, expr: T ) -> Result<T, E>;
}

impl<'src> Expr<'src> {
  pub fn visit<T, E, V: ExprVisitor<'src, T, E>>( &self, visitor: &V ) -> Result<T, E>
  {
    match self {
      Self::Binary( left, op , right )
        => V::visit_binary( left.visit( visitor )?, op, right.visit( visitor )? ),
      Self::Grouping( inner )
        => V::visit_grouping( inner.visit( visitor )? ),
      Self::Literal( literal )
        => V::visit_literal( literal ),
      Self::Unary( op, expr )
        => V::visit_unary( op, expr.visit( visitor )? )
    }
  }
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