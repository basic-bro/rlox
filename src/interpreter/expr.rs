////////////////////////////////////////////
// private module rlox::interpreter::expr //
////////////////////////////////////////////


/////////
// use //
/////////

use crate::interpreter::token::*;
use crate::util::*;


//////////////////////
// public interface //
//////////////////////

#[derive(Debug)]
pub enum Expr {
  Binary( Box<Expr> /* left */, Token /* operator */, Box<Expr> /* right */ ),
  Grouping( Box<Expr> ),
  Literal( Token /* identifier | string | number */ ),
  Unary( Token /* operator */, Box<Expr> )
}

pub trait ExprVisitor<T, E> {
  fn visit_binary( &self, left: T, op: &Token, right: T ) -> Result<T, E>;
  fn visit_grouping( &self, expr: T ) -> Result<T, E>;
  fn visit_literal( &self, literal: &Token ) -> Result<T, E>;
  fn visit_unary( &self, op: &Token, expr: T ) -> Result<T, E>;
}

impl Expr {

  pub fn visit<T, E, V: ExprVisitor<T, E>>( &self, visitor: &V ) -> Result<T, E>
  {
    match self {
      Self::Binary( left, op , right )
        => visitor.visit_binary( left.visit( visitor )?, op, right.visit( visitor )? ),
      Self::Grouping( inner )
        => visitor.visit_grouping( inner.visit( visitor )? ),
      Self::Literal( literal )
        => visitor.visit_literal( literal ),
      Self::Unary( op, expr )
        => visitor.visit_unary( op, expr.visit( visitor )? )
    }
  }

  pub fn to_string( &self, db: &StringManager ) -> String {
    self.visit( &ExprPrinter::new( db ) ).unwrap()
  }
  
}

pub type PrintResult = Result<String, String>;

pub struct ExprPrinter<'str> {
  db: &'str StringManager
}

impl<'str> ExprPrinter<'str> {

  pub fn new( db: &'str StringManager ) -> ExprPrinter<'str> {
    ExprPrinter {
      db
    }
  }

}

impl<'str> ExprVisitor<String, String> for ExprPrinter<'str> {

  fn visit_binary( &self, left: String, op: &Token, right: String ) -> Result<String, String> {
    Ok( format!( "{} {} {}", left, op.get_lexeme( self.db ), right ) )
  }

  fn visit_grouping( &self, expr: String ) -> Result<String, String> {
    Ok( format!( "( {} )", expr ) )
  }

  fn visit_literal( &self, literal: &Token ) -> Result<String, String> {
    Ok(
      match literal.get_token_type() {
        TokenType::String( s ) => format!( "\"{}\"", self.db.gets( *s ) ),
                                            _ => format!( "{}", literal.get_lexeme( self.db ) )
      }
    )
  }

  fn visit_unary( &self, op: &Token, expr: String ) -> Result<String, String> {
    Ok( format!( "{}{}", op.get_lexeme( self.db ), expr ) )
  }

}