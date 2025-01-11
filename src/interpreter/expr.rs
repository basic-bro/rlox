////////////////////////////////////////////
// private module rlox::interpreter::expr //
////////////////////////////////////////////


/////////
// use //
/////////

use std::iter::zip;

use crate::interpreter::error::*;
use crate::interpreter::token::*;
use crate::interpreter::eval::*;
use crate::interpreter::env::*;
use crate::interpreter::executor::*;
use crate::interpreter::stmt::*;
use crate::interpreter::format::*;

use crate::util::*;


//////////////////////
// public interface //
//////////////////////

#[derive(Debug, Clone)]
pub enum Expr {
  Assignment( /* symbol: */ Token , /* rhs: */ Box<Expr> ),
  Binary( /* left: */ Box<Expr>, /* operator: */ Token , /* right: */ Box<Expr> ),
  Call( /* callee: */ Box<Expr>, /* paren: */ Token, /* args: */ Vec<Box<Expr>> ),
  Grouping( /* inner: */ Box<Expr> ),
  Literal( /* value: */ Token /* identifier? | string | number | true | false | nil | eof */ ),
  Unary( /* operator: */ Token , /* rhs: */ Box<Expr> ),
  Symbol( /* symbol: */ Token )
}

impl Expr {
  pub fn to_string( &self, sc: &StringCache ) -> String {
    match self.map_fold( &mut ExprFormatter::new( sc ) ) {
      Ok( s ) => s,
      Err( error ) => error.msg
    }
  }
  pub fn eval( &self, sc: &mut StringCache, envs: &EnvStack ) -> EvalResult {
    self.map_fold( &mut ExprEvaluator::new( sc, envs ) )
  }
}


////////////////////////////
// private implementation //
////////////////////////////

pub trait ExprVisitor<T, E> {
  fn fold_assignment( &self, var: &Token, right: T ) -> Result<T, E>;
  fn fold_binary( &self, left: T, op: &Token, right: T ) -> Result<T, E>;
  fn fold_mut_call( &mut self, callee: T, paren: &Token, args: &Vec<T> ) -> Result<T, E>;
  fn fold_grouping( &self, expr: T ) -> Result<T, E>;
  fn fold_literal( &self, literal: &Token ) -> Result<T, E>;
  fn fold_unary( &self, op: &Token, expr: T ) -> Result<T, E>;
  fn fold_symbol( &self, symbol: &Token ) -> Result<T, E>;
}

pub trait ExprVisitorTgt<T, E> {
  fn map_fold<V: ExprVisitor<T, E>>( &self, visitor: &mut V ) -> Result<T, E>;
}

pub trait ExprVisitorMut<T, E> {
  fn fold_mut_assignment( &mut self, symbol_name: &Token, right: T ) -> Result<T, E>;
  fn fold_mut_binary( &mut self, left: T, op: &Token, right: T ) -> Result<T, E>;
  fn fold_mut_call( &mut self, callee: T, paren: &Token, args: &Vec<T> ) -> Result<T, E>;
  fn fold_mut_grouping( &mut self, expr: T ) -> Result<T, E>;
  fn fold_mut_literal( &mut self, literal: &Token ) -> Result<T, E>;
  fn fold_mut_unary( &mut self, op: &Token, expr: T ) -> Result<T, E>;
  fn fold_mut_symbol( &mut self, symbol_name: &Token ) -> Result<T, E>;
}

pub trait ExprVisitorMutTgt<T, E> {
  fn map_fold_mut<V: ExprVisitorMut<T, E>>( &self, visitor: &mut V ) -> Result<T, E>;
}

impl<T, E> ExprVisitorTgt<T, E> for Expr {
  fn map_fold<V: ExprVisitor<T, E>>( &self, visitor: &mut V ) -> Result<T, E> {
    match self {
      Self::Assignment( var, right ) => {
        let rv = right.map_fold( visitor )?;
        visitor.fold_assignment( var, rv )
      },
      Self::Binary( left, op , right ) => {
        let lv = left.map_fold( visitor )?;
        let rv = right.map_fold( visitor )?;
        visitor.fold_binary( lv, op, rv )
      },
      Self::Call( callee, paren , args ) => {
        let cv = callee.map_fold( visitor )?;
        let mut avs: Vec<T> = Vec::new();
        for arg in args {
          let av = arg.map_fold( visitor )?;
          avs.push( av );
        }
        visitor.fold_mut_call( cv, paren, &avs )
      }
      Self::Grouping( inner ) => {
        let iv = inner.map_fold( visitor )?;
        visitor.fold_grouping( iv )
      },
      Self::Literal( literal ) => {
        visitor.fold_literal( literal )
      },
      Self::Unary( op, expr ) => {
        let ev = expr.map_fold( visitor )?;
        visitor.fold_unary( op, ev )
      },
      Self::Symbol( op ) => {
        visitor.fold_symbol( op )
      }
    }
  }
}

impl<T, E> ExprVisitorMutTgt<T, E> for Expr {
  fn map_fold_mut<V: ExprVisitorMut<T, E>>( &self, visitor: &mut V ) -> Result<T, E> {
    match self {
      Self::Assignment( var, right ) => {
        let rv = right.map_fold_mut( visitor )?;
        visitor.fold_mut_assignment( var, rv )
      },
      Self::Binary( left, op , right ) => {
        let lv = left.map_fold_mut( visitor )?;
        let rv = right.map_fold_mut( visitor )?;
        visitor.fold_mut_binary( lv, op, rv )
      },
      Self::Call( callee, paren , args ) => {
        let cv = callee.map_fold_mut( visitor )?;
        let mut avs: Vec<T> = Vec::new();
        for arg in args {
          let av = arg.map_fold_mut( visitor )?;
          avs.push( av );
        }
        visitor.fold_mut_call( cv, paren, &avs )
      }
      Self::Grouping( inner ) => {
        let iv = inner.map_fold_mut( visitor )?;
        visitor.fold_mut_grouping( iv )
      },
      Self::Literal( literal ) => {
        visitor.fold_mut_literal( literal )
      },
      Self::Unary( op, expr ) => {
        let ev = expr.map_fold_mut( visitor )?;
        visitor.fold_mut_unary( op, ev )
      },
      Self::Symbol( op ) => {
        visitor.fold_mut_symbol( op )
      }
    }
  }
}



pub struct ExprEvaluator<'str, 'env> {
  sc: &'str mut StringCache,
  envs: &'env EnvStack
}

pub enum EvalError {
  Error( Error ),
  Return( Eval )
}
  
pub type EvalResult = Result<Eval, EvalError>;

impl<'str, 'env> ExprEvaluator<'str, 'env> {
  pub fn new( sc: &'str mut StringCache, envs: &'env EnvStack ) -> ExprEvaluator<'str, 'env> {
    ExprEvaluator {
      sc,
      envs
    }
  }
}

impl<'str, 'env> ExprVisitor<Eval, EvalError> for ExprEvaluator<'str, 'env> {

  fn fold_assignment( &self, _: &Token, right: Eval ) -> Result<Eval, EvalError> {
    Ok( right )
  }

  fn fold_binary( &self, left: Eval, op: &Token, right: Eval ) -> Result<Eval, EvalError> {
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
                    "Unknown binary operation on type Number.".to_string(), self.sc ) ) )
                },
          
          // binary operations on StringLiterals
          ( Eval::StringLiteral( x ), Eval::StringLiteral( y ) )
            =>  match op.get_type() {

                  // concatenation
                  TokenType::Plus => Ok( Eval::StringLiteral( x.to_owned() + y ) ),

                  // error
                  _ => Err( EvalError::Error( Error::from_token( op,
                    "Unknown binary operation on type String.".to_string(), self.sc ) ) )
                },
          
          // binary operations on Bools
          ( Eval::Bool( x ), Eval::Bool( y ) )
            =>  match op.get_type() {

                  // equality
                  TokenType::EqualEqual => Ok( Eval::Bool( x == y ) ),
                  TokenType::BangEqual  => Ok( Eval::Bool( x != y ) ),

                  // error
                  _ => Err( EvalError::Error( Error::from_token( op,
                    "Unknown binary operation on type Bool.".to_string(), self.sc ) ) )
            },

          // binary operation on Nils
          ( Eval::Nil, Eval::Nil )
            =>  match op.get_type() {

                // equality
                TokenType::EqualEqual => Ok( Eval::Bool( true ) ),
                TokenType::BangEqual  => Ok( Eval::Bool( false ) ),

                // error
                _ => Err( EvalError::Error( Error::from_token( op,
                  "Unknown binary operation on type Nil.".to_string(), self.sc ) ) )
            }

          // error
          _ => Err( EvalError::Error( Error::from_token( op,
            format!( "Unknown binary operation on the types provided. (The types are {} and {}, respectively.)",
              left.get_type_name(), right.get_type_name() ), self.sc ) ) )
        }
    }
  }

  fn fold_mut_call( &mut self, callee: Eval, paren: &Token, args: &Vec<Eval> ) -> Result<Eval, EvalError> {

    // if working correctly, callee will be an Eval::Fun
    // from which we can invoke the function call.
    if let Eval::Fun( param_keys, body ) = callee {

      if let Stmt::Block( decls, line ) = body {

        // check arity
        if param_keys.len() != args.len() {
          return Err( EvalError::Error( Error::from_token( paren,
            format!( "Expected {} arguments to function call, but found {}.", param_keys.len(), args.len() ), self.sc ) ) );
        }

        // prepare function scope
        let mut fun_envs = self.envs.clone_global();
        fun_envs.enclose_new( paren.get_line() );

        // add parameters to function scope
        for ( key, value ) in zip( param_keys, args ) {
          fun_envs.create_symbol( key, value.clone() );
        }

        let mut fn_exec = Executor::with_envs( &mut self.sc, fun_envs );

        match fn_exec.exec_block_stmt( &decls, &line, false ) {
          Err( EvalError::Return( retval ) ) => Ok( retval ),
          result => result
        }
      } else {
        panic!( "Internal error: 'body' should have type Stmt::Block, but it has type {} instead.", body.get_type_name() );  
      }
    } else {
      Err( EvalError::Error( Error::from_token( paren, format!( "Cannot call a {}.", callee.get_type_name() ), self.sc ) ) )
    }
  }

  fn fold_grouping( &self, expr: Eval ) -> Result<Eval, EvalError> {
    Ok( expr )
  }

  fn fold_literal( &self, literal: &Token ) -> Result<Eval, EvalError> {
    match literal.get_type() {
      TokenType::String( s ) => Ok( Eval::StringLiteral( self.sc.gets( *s ).to_string() ) ),
      TokenType::Number( s ) => Ok( Eval::Number( self.sc.gets( *s ).parse::<f64>().unwrap() ) ),
      TokenType::True => Ok( Eval::Bool( true ) ),
      TokenType::False => Ok( Eval::Bool( false ) ),
      TokenType::Nil => Ok( Eval::Nil ),
      _ => Err( EvalError::Error( Error::from_token( literal,
        "Internal error: evaluation of this expression is not implemented.".to_string(), self.sc ) ) )
    }
  }

  fn fold_unary( &self, op: &Token, expr: Eval ) -> Result<Eval, EvalError> {
    match op.get_type() {
      TokenType::Bang => Ok( Eval::Bool( !expr.is_truthy() ) ),
      TokenType::Minus => match expr {
        Eval::Number( x ) => Ok( Eval::Number( -x ) ),
        _ => Err( EvalError::Error( Error::from_token( op,
          format!( "Unary '-' cannot be applied to a value of type {}.", expr.get_type_name() ), self.sc ) ) )
      },
      _ => Err( EvalError::Error( Error::from_token( op,
        "Internal error: evaluation of this unary operator is not implemented.".to_string(), self. sc ) ) )
    }
  }

  fn fold_symbol( &self, var: &Token ) -> Result<Eval, EvalError> {
    match var.get_type() {
      TokenType::Identifier( id ) => {

        // error on undeclared variable
        if !self.envs.has_symbol( *id ) {
          Err( EvalError::Error( Error::from_token( var, "Undeclared variable.".to_string(), self.sc ) ) )
        } else {
          Ok( self.envs.read_symbol( *id ).clone() )
        }
      }
      _ => Err( EvalError::Error( Error::from_token( var,
        "Internal error: evaluation of this expression is not implemented.".to_string(), self. sc ) ) )
    }
  }
}