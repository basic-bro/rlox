////////////////////////////////////////////
// private module rlox::interpreter::expr //
////////////////////////////////////////////


/////////
// use //
/////////

use crate::interpreter::error::*;
use crate::interpreter::token::*;
use crate::util::*;
use crate::interpreter::eval::*;
use crate::interpreter::env::*;


//////////////////////
// public interface //
//////////////////////

#[derive(Debug, Clone)]
pub enum Expr {
  Binary( Box<Expr> /* left */, Token /* operator */, Box<Expr> /* right */ ),
  Grouping( Box<Expr> ),
  Literal( Token /* identifier | string | number | true | false | nil | eof */ ),
  Unary( Token /* operator */, Box<Expr> )
}

pub trait ExprVisitor<T> {
  fn visit_binary( &self, left: T, op: &Token, right: T ) -> Result<T, Error>;
  fn visit_grouping( &self, expr: T ) -> Result<T, Error>;
  fn visit_literal( &self, literal: &Token ) -> Result<T, Error>;
  fn visit_unary( &self, op: &Token, expr: T ) -> Result<T, Error>;
}

impl Expr {

  pub fn visit<T, V: ExprVisitor<T>>( &self, visitor: &V ) -> Result<T, Error>
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
    match self.visit( &ExprFormatter::new( db ) ) {
      Ok( s ) => s,
      Err( error ) => error.msg
    }
  }

  pub fn eval( &self, db: &StringManager, env: &Env ) -> EvalResult {
    self.visit( &ExprEvaluator::new( db, env ) )
  }
  
}

pub struct ExprFormatter<'str> {
  db: &'str StringManager
}

impl<'str> ExprFormatter<'str> {

  pub fn new( db: &'str StringManager ) -> ExprFormatter<'str> {
    ExprFormatter {
      db
    }
  }

}

impl<'str> ExprVisitor<String> for ExprFormatter<'str> {

  fn visit_binary( &self, left: String, op: &Token, right: String ) -> Result<String, Error> {
    Ok( format!( "{} {} {}", left, op.get_lexeme( self.db ), right ) )
  }

  fn visit_grouping( &self, expr: String ) -> Result<String, Error> {
    Ok( format!( "( {} )", expr ) )
  }

  fn visit_literal( &self, literal: &Token ) -> Result<String, Error> {
    Ok(
      match literal.get_token_type() {
        TokenType::String( s ) => format!( "\"{}\"", self.db.gets( *s ) ),
                                            _ => format!( "{}", literal.get_lexeme( self.db ) )
      }
    )
  }

  fn visit_unary( &self, op: &Token, expr: String ) -> Result<String, Error> {
    Ok( format!( "{}{}", op.get_lexeme( self.db ), expr ) )
  }

}