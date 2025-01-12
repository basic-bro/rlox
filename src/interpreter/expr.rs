////////////////////////////////////////////
// private module rlox::interpreter::expr //
////////////////////////////////////////////


/////////
// use //
/////////

use crate::interpreter::token::*;
use crate::interpreter::env::*;
use crate::interpreter::executor::*;
use crate::interpreter::format::*;
use crate::util::*;


//////////////////
// declarations //
//////////////////

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

pub trait ExprFolder<T, E> {
  // due to Expr layout, 'map' not required/feasible
  // fn map( &mut self, node: &Expr ) -> Result<T, E>;

  // due to Expr layout, 'fold' required for each enum variant
  // fn fold( &mut self, node_result: T, child_results: Vec<T> ) -> Result<T, E>;

  fn fold_assignment( &mut self, var: &Token, right: T ) -> Result<T, E>;
  fn fold_binary( &mut self, left: T, op: &Token, right: T ) -> Result<T, E>;
  fn fold_call( &mut self, callee: T, paren: &Token, args: &Vec<T> ) -> Result<T, E>;
  fn fold_grouping( &mut self, expr: T ) -> Result<T, E>;
  fn fold_literal( &mut self, literal: &Token ) -> Result<T, E>;
  fn fold_unary( &mut self, op: &Token, expr: T ) -> Result<T, E>;
  fn fold_symbol( &mut self, symbol: &Token ) -> Result<T, E>;
}

pub trait ExprFolderTgt<T, E> {
  fn map_fold<V: ExprFolder<T, E>>( &self, visitor: &mut V ) -> Result<T, E>;
}

pub trait ExprVisitor<E> {
  fn visit( &mut self, node: &Expr ) -> Result<(), E>;
  fn before_children( &mut self, node: &Expr );
  fn after_children( &mut self, node: &Expr );
}

pub trait ExprVisitorTgt<E> {
  fn accept<V: ExprVisitor<E>>( &self, visitor: &mut V ) -> Result<(), E>;
}


/////////////////////
// implementations //
/////////////////////

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

impl<E> ExprVisitorTgt<E> for Expr {
  fn accept<V: ExprVisitor<E>>( &self, visitor: &mut V ) -> Result<(), E> {
    visitor.visit( self )?;
    visitor.before_children( self );
    match self {
      Expr::Assignment( _, expr) => {
        expr.accept( visitor )?;
      },
      Expr::Binary( left, _, right ) => {
        left.accept( visitor )?;
        right.accept( visitor )?;
      },
      Expr::Call( callee, _, args) => {
        callee.accept( visitor )?;
        for arg in args {
          arg.accept( visitor )?;
        }
      },
      Expr::Grouping( inner) => {
        inner.accept( visitor )?;
      },
      Expr::Literal( _ ) => {},
      Expr::Unary( _, right) => {
        right.accept( visitor )?;
      },
      Expr::Symbol( _ ) => {},
    }
    visitor.after_children( self );
    Ok( () )
  }
}

impl<T, E> ExprFolderTgt<T, E> for Expr {
  fn map_fold<V: ExprFolder<T, E>>( &self, visitor: &mut V ) -> Result<T, E> {
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
        visitor.fold_call( cv, paren, &avs )
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