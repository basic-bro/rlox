////////////////////////////////////////////
// private module rlox::interpreter::eval //
////////////////////////////////////////////


/////////
// use //
/////////

use std::fmt::Display;

use crate::interpreter::token::*;
use crate::interpreter::expr::*;
use crate::util::*;


//////////////////////
// public interface //
//////////////////////

#[derive(Debug)]
pub enum Eval {
  Number( f64 ),
  StringLiteral( String ),
  Bool( bool ),
  Nil
}

impl Eval {

  pub fn is_truthy( &self ) -> bool {
    match self {

      // "nil" and "false" are falsey
      Eval::Nil => false,
      Eval::Bool( false ) => false,

      // everything else is truthy
      _ => true
    }
  }

}

impl Display for Eval {

  fn fmt( &self, f: &mut std::fmt::Formatter<'_> ) -> std::fmt::Result {
    match self {
      Self::Number( x ) => write!( f, "{}", x ),
      Self::StringLiteral( s ) => write!( f, "{}", s ),
      Self::Bool( b ) => write!( f, "{}", b ),
      Self::Nil => write!( f, "nil" )
    }
  }

}
  
pub type EvalResult = Result<Eval, String>;

pub struct ExprEvaluator<'str> {
  db: &'str StringManager
}

impl<'str> ExprEvaluator<'str> {

  pub fn new( db: &'str StringManager ) -> ExprEvaluator<'str> {
    ExprEvaluator {
      db
    }
  }

}


////////////////////////////
// private implementation //
////////////////////////////

impl<'str> ExprVisitor<Eval, String> for ExprEvaluator<'str> {

  fn visit_binary( &self, left: Eval, op: &Token, right: Eval ) -> Result<Eval, String> {
    match op.get_token_type() {

      // first, evaluate any logical operator
      // [ these involve casting to bool => .is_truthy() ]
      TokenType::And => Ok( Eval::Bool( left.is_truthy() && right.is_truthy() ) ),
      TokenType::Or => Ok( Eval::Bool( left.is_truthy() || right.is_truthy() ) ),

      // then, treat according to operand types
      // [ no type conversions required ]
      _ =>  match ( &left, &right ) {

          // binary operations on Numbers
          ( Eval::Number( x ), Eval::Number( y ) )
            =>  match op.get_token_type() {

                  // equality
                  TokenType::EqualEqual => Ok( Eval::Bool( x == y ) ),
                  TokenType::BangEqual  => Ok( Eval::Bool( x != y ) ),

                  // comparison
                  TokenType::Greater      => Ok( Eval::Bool( x > y ) ),
                  TokenType::GreaterEqual => Ok( Eval::Bool( x >= y ) ),
                  TokenType::Less         => Ok( Eval::Bool( x < y ) ),
                  TokenType::LessEqual    => Ok( Eval::Bool( x <= y ) ),

                  // term
                  TokenType::Plus  => Ok( Eval::Number( x + y ) ),
                  TokenType::Minus => Ok( Eval::Number( x - y ) ),

                  // factor
                  TokenType::Star  => Ok( Eval::Number( x * y ) ),
                  TokenType::Slash => Ok( Eval::Number( x / y ) ),
                  
                  // error 
                  _ => Err( format!( "Unknown binary operation on numbers: '{}'", op.get_lexeme( self.db ) ) )
                },
          
          // binary operations on StringLiterals
          ( Eval::StringLiteral( x ), Eval::StringLiteral( y ) )
            =>  match op.get_token_type() {

                  // concatenation
                  TokenType::Plus => Ok( Eval::StringLiteral( x.to_owned() + y ) ),

                  // error
                  _ => Err( format!( "Unknown binary operation on strings: '{}'", op.get_lexeme( self.db ) ) )
                },
          
          // binary operations on Bools
          ( Eval::Bool( x ), Eval::Bool( y ) )
            =>  match op.get_token_type() {

                  // equality
                  TokenType::EqualEqual => Ok( Eval::Bool( x == y ) ),
                  TokenType::BangEqual  => Ok( Eval::Bool( x != y ) ),

                  // error
                  _ => Err( format!( "Unknown binary operation on booleans: '{}'", op.get_lexeme( self.db ) ) )
            },

          // binary operation on Nils
          ( Eval::Nil, Eval::Nil )
            =>  match op.get_token_type() {

                // equality
                TokenType::EqualEqual => Ok( Eval::Bool( true ) ),
                TokenType::BangEqual  => Ok( Eval::Bool( false ) ),

                // error
                _ => Err( format!( "Unknown binary operation on nils: '{}'", op.get_lexeme( self.db ) ) )
            }

          // error
          _ => Err( format!( "Unknown or unsupported binary operation '{}' on values {:?} and {:?}", op.get_lexeme( self.db ), left, right ) )
        }
    }
  }

  fn visit_grouping( &self, expr: Eval ) -> Result<Eval, String> {
    Ok( expr )
  }

  fn visit_literal( &self, literal: &Token ) -> Result<Eval, String> {
    match literal.get_token_type() {
      TokenType::String( s ) => Ok( Eval::StringLiteral( self.db.gets( *s ).to_string() ) ),
      TokenType::Number( s ) => Ok( Eval::Number( self.db.gets( *s ).parse::<f64>().unwrap() ) ),
      TokenType::True => Ok( Eval::Bool( true ) ),
      TokenType::False => Ok( Eval::Bool( false ) ),
      TokenType::Nil => Ok( Eval::Nil ),
      TokenType::Identifer( _ ) => Err( format!( "eval() not implemented yet: {:?}", literal ) ),
      _ => Err( format!( "Internal error: this token should not be parsed as an Expr::Literal: {:?}", literal ) )
    }
  }

  fn visit_unary( &self, op: &Token, expr: Eval ) -> Result<Eval, String> {
    match op.get_token_type() {
      TokenType::Bang => Ok( Eval::Bool( !expr.is_truthy() ) ),
      TokenType::Minus => match expr {
        Eval::Number( x ) => Ok( Eval::Number( -x ) ),
        _ => Err( format!( "Unary '-' cannot be applied to the value '{:?}'", expr ) )
      },
      _ => Err( format!( "Internal error: this token should not be parsed as a unary operation: {}", op.get_lexeme( self.db ) ) )
    }
  }
  
}