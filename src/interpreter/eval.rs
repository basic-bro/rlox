////////////////////////////////////////////
// private module rlox::interpreter::eval //
////////////////////////////////////////////


/////////
// use //
/////////

use std::fmt::Display;
use std::iter::zip;

use crate::interpreter::env::*;
use crate::interpreter::error::*;
use crate::interpreter::token::*;
use crate::interpreter::expr::*;
use crate::interpreter::stmt::*;

use crate::util::*;

use super::Executor;


//////////////////////
// public interface //
//////////////////////

#[derive(Debug, Clone)]
pub enum Eval {
  Number( f64 ),
  StringLiteral( String ),
  Bool( bool ),
  Nil,
  Fun( /* args: */ Vec<StringKey>, /* body: */ Stmt )
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

  pub fn get_type_name( &self ) -> String {
    match self {
      Eval::Number( _ ) => "Number".to_string(),
      Eval::StringLiteral( _ ) => "String".to_string(),
      Eval::Bool( _ ) => "Bool".to_string(),
      Eval::Nil => "Nil".to_string(),
      Eval::Fun( args, _ ) => format!( "Fun<{}>", args.len() )
    }
  }

}

impl Display for Eval {

  fn fmt( &self, f: &mut std::fmt::Formatter<'_> ) -> std::fmt::Result {
    match self {
      Self::Number( x ) => write!( f, "{}", x ),
      Self::StringLiteral( s ) => write!( f, "{}", s ),
      Self::Bool( b ) => write!( f, "{}", b ),
      Self::Nil => write!( f, "nil" ),
      Self::Fun( args, _ ) => write!( f, "fun<{}>", args.len() )
    }
  }

}

pub enum EvalError {
  Error( Error ),
  Return( Eval )
}
  
pub type EvalResult = Result<Eval, EvalError>;

pub struct ExprEvaluator<'str, 'env> {
  sm: &'str mut StringManager,
  env: &'env RcMut<Env>
}

impl<'str, 'env> ExprEvaluator<'str, 'env> {

  pub fn new( sm: &'str mut StringManager, env: &'env RcMut<Env> ) -> ExprEvaluator<'str, 'env> {
    ExprEvaluator {
      sm,
      env
    }
  }

}


////////////////////////////
// private implementation //
////////////////////////////

impl<'str, 'env> ExprVisitorMut<Eval, EvalError> for ExprEvaluator<'str, 'env> {

  fn visit_assignment( &self, _: &Token, right: Eval ) -> Result<Eval, EvalError> {
    Ok( right )
  }

  fn visit_binary( &self, left: Eval, op: &Token, right: Eval ) -> Result<Eval, EvalError> {
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
                  _ => Err( EvalError::Error( Error::from_token( op,
                    "Unknown binary operation on type Number.".to_string(), self.sm ) ) )
                },
          
          // binary operations on StringLiterals
          ( Eval::StringLiteral( x ), Eval::StringLiteral( y ) )
            =>  match op.get_type() {

                  // concatenation
                  TokenType::Plus => Ok( Eval::StringLiteral( x.to_owned() + y ) ),

                  // error
                  _ => Err( EvalError::Error( Error::from_token( op,
                    "Unknown binary operation on type String.".to_string(), self.sm ) ) )
                },
          
          // binary operations on Bools
          ( Eval::Bool( x ), Eval::Bool( y ) )
            =>  match op.get_type() {

                  // equality
                  TokenType::EqualEqual => Ok( Eval::Bool( x == y ) ),
                  TokenType::BangEqual  => Ok( Eval::Bool( x != y ) ),

                  // error
                  _ => Err( EvalError::Error( Error::from_token( op,
                    "Unknown binary operation on type Bool.".to_string(), self.sm ) ) )
            },

          // binary operation on Nils
          ( Eval::Nil, Eval::Nil )
            =>  match op.get_type() {

                // equality
                TokenType::EqualEqual => Ok( Eval::Bool( true ) ),
                TokenType::BangEqual  => Ok( Eval::Bool( false ) ),

                // error
                _ => Err( EvalError::Error( Error::from_token( op,
                  "Unknown binary operation on type Nil.".to_string(), self.sm ) ) )
            }

          // error
          _ => Err( EvalError::Error( Error::from_token( op,
            format!( "Unknown binary operation on the types provided. (The types are {} and {}, respectively.)",
              left.get_type_name(), right.get_type_name() ), self.sm ) ) )
        }
    }
  }

  fn visit_call( &mut self, callee: Eval, paren: &Token, args: &Vec<Eval> ) -> Result<Eval, EvalError> {

    // if working correctly, callee will be an Eval::Fun
    // from which we can invoke the function call.
    if let Eval::Fun( param_keys, body ) = callee {

      if let Stmt::Block( decls, line ) = body {

        // check arity
        if param_keys.len() != args.len() {
          return Err( EvalError::Error( Error::from_token( paren,
            format!( "Expected {} arguments to function call, but found {}.", param_keys.len(), args.len() ), self.sm ) ) );
        }

        // prepare function scope
        let fun_global = Env::clone_global( &self.env );
        let mut fun_scope = Env::enclose_new( &fun_global, paren.get_line() );

        // add parameters to function scope
        for ( key, value ) in zip( param_keys, args ) {
          fun_scope.view_mut().create_var( key, value.clone() );
        }

        let mut fn_exec = Executor::with_env( &mut self.sm, fun_scope.clone() );
        
        
        // Env::drop_enclosed( &fun_scope ); // needed?

        match fn_exec.exec_block_stmt( &decls, &line, false ) {
          Err( EvalError::Return( retval ) ) => Ok( retval ),
          result => result
        }
      } else {
        panic!( "Internal error: 'body' should have type Stmt::Block, but it has type {} instead.", body.get_type_name() );  
      }
    } else {
      Err( EvalError::Error( Error::from_token( paren, format!( "Cannot call a {}.", callee.get_type_name() ), self.sm ) ) )
    }
  }

  fn visit_grouping( &self, expr: Eval ) -> Result<Eval, EvalError> {
    Ok( expr )
  }

  fn visit_literal( &self, literal: &Token ) -> Result<Eval, EvalError> {
    match literal.get_type() {
      TokenType::String( s ) => Ok( Eval::StringLiteral( self.sm.gets( *s ).to_string() ) ),
      TokenType::Number( s ) => Ok( Eval::Number( self.sm.gets( *s ).parse::<f64>().unwrap() ) ),
      TokenType::True => Ok( Eval::Bool( true ) ),
      TokenType::False => Ok( Eval::Bool( false ) ),
      TokenType::Nil => Ok( Eval::Nil ),
      _ => Err( EvalError::Error( Error::from_token( literal,
        "Internal error: evaluation of this expression is not implemented.".to_string(), self.sm ) ) )
    }
  }

  fn visit_unary( &self, op: &Token, expr: Eval ) -> Result<Eval, EvalError> {
    match op.get_type() {
      TokenType::Bang => Ok( Eval::Bool( !expr.is_truthy() ) ),
      TokenType::Minus => match expr {
        Eval::Number( x ) => Ok( Eval::Number( -x ) ),
        _ => Err( EvalError::Error( Error::from_token( op,
          format!( "Unary '-' cannot be applied to a value of type {}.", expr.get_type_name() ), self.sm ) ) )
      },
      _ => Err( EvalError::Error( Error::from_token( op,
        "Internal error: evaluation of this unary operator is not implemented.".to_string(), self. sm ) ) )
    }
  }

  fn visit_var( &self, var: &Token ) -> Result<Eval, EvalError> {
    match var.get_type() {
      TokenType::Identifier( id ) => {

        // error on undeclared variable
        if !self.env.view().has_var( *id ) {
          Err( EvalError::Error( Error::from_token( var, "Undeclared variable.".to_string(), self.sm ) ) )
        } else {
          Ok( self.env.view().read_var( *id ).clone() )
        }
      }
      _ => Err( EvalError::Error( Error::from_token( var,
        "Internal error: evaluation of this expression is not implemented.".to_string(), self. sm ) ) )
    }
  }
  
}