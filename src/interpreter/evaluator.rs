/////////////////////////////////////////////////
// private module rlox::interpreter::evaluator //
/////////////////////////////////////////////////


/////////
// use //
/////////

use crate::interpreter::token::*;
use crate::interpreter::expr::*;


//////////////////////
// public interface //
//////////////////////

#[derive(Debug)]
pub enum Eval {
  Number( f64 ),
  StringLiteral( String )
}
  
pub type EvalResult = Result<Eval, String>;

pub struct Evaluator { }


////////////////////////////
// private implementation //
////////////////////////////

impl<'src> ExprVisitor<'src, Eval, String> for Evaluator {

  fn visit_binary( left: Eval, op: &Token<'src>, right: Eval ) -> Result<Eval, String> {
    match ( &left, &right ) {
      ( Eval::Number( x ), Eval::Number( y ) )
        =>  match op.get_token_type() {
              TokenType::Plus => Ok( Eval::Number( x + y ) ),
              TokenType::Minus => Ok( Eval::Number( x - y ) ),
              TokenType::Star => Ok( Eval::Number( x * y ) ),
              TokenType::Slash => Ok( Eval::Number( x / y ) ),
              _ => Err( format!( "Unknown binary operation on numbers: '{}'", op.get_lexeme() ) )
            },
      ( Eval::StringLiteral( x ), Eval::StringLiteral( y ) )
        =>  match op.get_token_type() {
              TokenType::Plus => Ok( Eval::StringLiteral( x.to_owned() + y ) ),
              _ => Err( format!( "Unknown binary operation on strings: '{}'", op.get_lexeme() ) )
            },
      _ => Err( format!( "Unknown or unsupported binary operation '{}' on values {:?} and {:?}", op.get_lexeme(), left, right ) )
    }
  }

  fn visit_grouping( expr: Eval ) -> Result<Eval, String> {
    Ok( expr )
  }

  fn visit_literal( literal: &Token<'src> ) -> Result<Eval, String> {
    match literal.get_token_type() {
      TokenType::String( s ) => Ok( Eval::StringLiteral( s.to_string() ) ),
      TokenType::Number( s ) => Ok( Eval::Number( s.parse::<f64>().unwrap() ) ),
      TokenType::Identifer( _ ) => Err( format!( "eval() not implemented yet: {:?}", literal ) ),
      _ => Err( format!( "Internal error: this token should not be parsed as an Expr::Literal: {:?}", literal ) )
    }
  }

  fn visit_unary( op: &Token<'src>, expr: Eval ) -> Result<Eval, String> {
    match &expr {
      Eval::Number( x )
        => if *op.get_token_type() == TokenType::Minus {
          Ok( Eval::Number( -x ) )
        } else {
          Err( format!( "Unknown unary operation on a number: '{}'", op.get_lexeme() ) )
        },
      _ => Err( format!( "Unary operator '{}' not implemented for {:?}", op.get_lexeme(), expr ) )
    }
  }
}