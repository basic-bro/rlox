////////////////////////////////////////////
// private module rlox::interpreter::eval //
////////////////////////////////////////////


/////////
// use //
/////////

use std::fmt::Display;

use crate::interpreter::env::*;
use crate::interpreter::error::*;
use crate::interpreter::token::*;
use crate::interpreter::expr::*;

use crate::util::*;


//////////////////////
// public interface //
//////////////////////

#[derive(Debug, Clone)]
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

  pub fn get_type_name( &self ) -> &str {
    match self {
      Eval::Number( _ ) => "Number",
      Eval::StringLiteral( _ ) => "String",
      Eval::Bool( _ ) => "Bool",
      Eval::Nil => "Nil"
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
  
pub type EvalResult = Result<Eval, Error>;

pub struct ExprEvaluator<'str, 'env> {
  sm: &'str StringManager,
  env: &'env Env
}

impl<'str, 'env> ExprEvaluator<'str, 'env> {

  pub fn new( sm: &'str StringManager, env: &'env Env ) -> ExprEvaluator<'str, 'env> {
    ExprEvaluator {
      sm,
      env
    }
  }

}


////////////////////////////
// private implementation //
////////////////////////////

impl<'str, 'env> ExprVisitor<Eval> for ExprEvaluator<'str, 'env> {

  fn visit_assignment( &self, _: &Token, right: Eval ) -> Result<Eval, Error> {
    Ok( right )
  }

  fn visit_binary( &self, left: Eval, op: &Token, right: Eval ) -> Result<Eval, Error> {
    match op.get_type() {

      // first, evaluate any logical operator
      // [ these involve casting to bool => .is_truthy() ]
      TokenType::And => Ok( Eval::Bool( left.is_truthy() && right.is_truthy() ) ),
      TokenType::Or => Ok( Eval::Bool( left.is_truthy() || right.is_truthy() ) ),

      // then, treat according to operand types
      // [ no type conversions required ]
      _ =>  match ( &left, &right ) {

          // binary operations on Numbers
          ( Eval::Number( x ), Eval::Number( y ) )
            =>  match op.get_type() {

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
                  _ => Err( Error::from_token( op,
                    "Unknown binary operation on type Number.".to_string(), self.sm ) )
                },
          
          // binary operations on StringLiterals
          ( Eval::StringLiteral( x ), Eval::StringLiteral( y ) )
            =>  match op.get_type() {

                  // concatenation
                  TokenType::Plus => Ok( Eval::StringLiteral( x.to_owned() + y ) ),

                  // error
                  _ => Err( Error::from_token( op,
                    "Unknown binary operation on type String.".to_string(), self.sm ) )
                },
          
          // binary operations on Bools
          ( Eval::Bool( x ), Eval::Bool( y ) )
            =>  match op.get_type() {

                  // equality
                  TokenType::EqualEqual => Ok( Eval::Bool( x == y ) ),
                  TokenType::BangEqual  => Ok( Eval::Bool( x != y ) ),

                  // error
                  _ => Err( Error::from_token( op,
                    "Unknown binary operation on type Bool.".to_string(), self.sm ) )
            },

          // binary operation on Nils
          ( Eval::Nil, Eval::Nil )
            =>  match op.get_type() {

                // equality
                TokenType::EqualEqual => Ok( Eval::Bool( true ) ),
                TokenType::BangEqual  => Ok( Eval::Bool( false ) ),

                // error
                _ => Err( Error::from_token( op,
                  "Unknown binary operation on type Nil.".to_string(), self.sm ) )
            }

          // error
          _ => Err( Error::from_token( op,
            format!( "Unknown binary operation on the types provided. (The types are {} and {}, respectively.)",
              left.get_type_name(), right.get_type_name() ), self.sm ) )
        }
    }
  }

  fn visit_grouping( &self, expr: Eval ) -> Result<Eval, Error> {
    Ok( expr )
  }

  fn visit_literal( &self, literal: &Token ) -> Result<Eval, Error> {
    match literal.get_type() {
      TokenType::String( s ) => Ok( Eval::StringLiteral( self.sm.gets( *s ).to_string() ) ),
      TokenType::Number( s ) => Ok( Eval::Number( self.sm.gets( *s ).parse::<f64>().unwrap() ) ),
      TokenType::True => Ok( Eval::Bool( true ) ),
      TokenType::False => Ok( Eval::Bool( false ) ),
      TokenType::Nil => Ok( Eval::Nil ),
      _ => Err( Error::from_token( literal,
        "Internal error: evaluation of this expression is not implemented.".to_string(), self.sm ) )
    }
  }

  fn visit_unary( &self, op: &Token, expr: Eval ) -> Result<Eval, Error> {
    match op.get_type() {
      TokenType::Bang => Ok( Eval::Bool( !expr.is_truthy() ) ),
      TokenType::Minus => match expr {
        Eval::Number( x ) => Ok( Eval::Number( -x ) ),
        _ => Err( Error::from_token( op,
          format!( "Unary '-' cannot be applied to a value of type {}.", expr.get_type_name() ), self.sm ) )
      },
      _ => Err( Error::from_token( op,
        "Internal error: evaluation of this unary operator is not implemented.".to_string(), self. sm ) )
    }
  }

  fn visit_var( &self, var: &Token ) -> Result<Eval, Error> {
    match var.get_type() {
      TokenType::Identifier( id ) => {

        // error on undeclared variable
        if !self.env.has_var( *id ) {
          Err( Error::from_token( var, "Undeclared variable.".to_string(), self.sm ) )
        } else {
          Ok( self.env.read_var( *id ).clone() )
        }
      }
      _ => Err( Error::from_token( var,
        "Internal error: evaluation of this expression is not implemented.".to_string(), self. sm ) )
    }
  }
  
}