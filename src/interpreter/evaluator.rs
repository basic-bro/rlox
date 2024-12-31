/////////////////////////////////////////////////
// private module rlox::interpreter::evaluator //
/////////////////////////////////////////////////


/////////
// use //
/////////

use crate::interpreter::token::*;
use crate::interpreter::expr::*;

use super::StringManager;


//////////////////////
// public interface //
//////////////////////

#[derive(Debug)]
pub enum Eval {
  Number( f64 ),
  StringLiteral( String )
}
  
pub type EvalResult = Result<Eval, String>;

pub struct Evaluator<'str> {
  db: &'str StringManager
}

impl<'str> Evaluator<'str> {

  pub fn new( db: &'str StringManager ) -> Evaluator<'str> {
    Evaluator {
      db
    }
  }

}


////////////////////////////
// private implementation //
////////////////////////////

impl<'str> ExprVisitor<Eval, String> for Evaluator<'str> {

  fn visit_binary( &self, left: Eval, op: &Token, right: Eval ) -> Result<Eval, String> {
    match ( &left, &right ) {
      ( Eval::Number( x ), Eval::Number( y ) )
        =>  match op.get_token_type() {
              TokenType::Plus => Ok( Eval::Number( x + y ) ),
              TokenType::Minus => Ok( Eval::Number( x - y ) ),
              TokenType::Star => Ok( Eval::Number( x * y ) ),
              TokenType::Slash => Ok( Eval::Number( x / y ) ),
              _ => Err( format!( "Unknown binary operation on numbers: '{}'", op.get_lexeme( self.db ) ) )
            },
      ( Eval::StringLiteral( x ), Eval::StringLiteral( y ) )
        =>  match op.get_token_type() {
              TokenType::Plus => Ok( Eval::StringLiteral( x.to_owned() + y ) ),
              _ => Err( format!( "Unknown binary operation on strings: '{}'", op.get_lexeme( self.db ) ) )
            },
      _ => Err( format!( "Unknown or unsupported binary operation '{}' on values {:?} and {:?}", op.get_lexeme( self.db ), left, right ) )
    }
  }

  fn visit_grouping( &self, expr: Eval ) -> Result<Eval, String> {
    Ok( expr )
  }

  fn visit_literal( &self, literal: &Token ) -> Result<Eval, String> {
    match literal.get_token_type() {
      TokenType::String( s ) => Ok( Eval::StringLiteral( self.db.gets( *s ).to_string() ) ),
      TokenType::Number( s ) => Ok( Eval::Number( self.db.gets( *s ).parse::<f64>().unwrap() ) ),
      TokenType::Identifer( _ ) => Err( format!( "eval() not implemented yet: {:?}", literal ) ),
      _ => Err( format!( "Internal error: this token should not be parsed as an Expr::Literal: {:?}", literal ) )
    }
  }

  fn visit_unary( &self, op: &Token, expr: Eval ) -> Result<Eval, String> {
    match &expr {
      Eval::Number( x )
        => if *op.get_token_type() == TokenType::Minus {
          Ok( Eval::Number( -x ) )
        } else {
          Err( format!( "Unknown unary operation on a number: '{}'", op.get_lexeme( self.db ) ) )
        },
      _ => Err( format!( "Unary operator '{}' not implemented for {:?}", op.get_lexeme( self.db ), expr ) )
    }
  }
  
}