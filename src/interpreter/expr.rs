////////////////////////////////////////////
// private module rlox::interpreter::expr //
////////////////////////////////////////////


/////////
// use //
/////////

use crate::interpreter::token::*;
use crate::interpreter::format::*;
use crate::interpreter::visitor::*;

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

pub trait ExprMapFolder<T, E> {
  fn map_expr( &mut self, expr: &Expr ) -> MapFolderState<T, E>;
  fn fold_expr_assignment( &mut self, var: &Token, right: T ) -> Result<T, E>;
  fn fold_expr_binary( &mut self, left: T, op: &Token, right: T ) -> Result<T, E>;
  fn fold_expr_call( &mut self, callee: T, paren: &Token, args: &Vec<T> ) -> Result<T, E>;
  fn fold_expr_grouping( &mut self, expr: T ) -> Result<T, E>;
  fn fold_expr_literal( &mut self, literal: &Token ) -> Result<T, E>;
  fn fold_expr_unary( &mut self, op: &Token, expr: T ) -> Result<T, E>;
  fn fold_expr_symbol( &mut self, symbol: &Token ) -> Result<T, E>;
}

pub trait ExprMapFolderTgt<T, E> {
  fn map_fold_expr<V: ExprMapFolder<T, E>>( &self, visitor: &mut V ) ->  Result<T, E>;
}

pub trait ExprVisitor<E> {
  fn visit_expr( &mut self, node: &Expr ) -> Result<(), E>;
  fn before_expr_children( &mut self, node: &Expr ) -> Result<VisitorControl, E>;
  fn after_expr_children( &mut self, node: &Expr );
}

pub trait ExprVisitorTgt<E> {
  fn accept<V: ExprVisitor<E>>( &self, visitor: &mut V ) -> Result<(), E>;
}


/////////////////////
// implementations //
/////////////////////

impl Expr {
  pub fn to_string( &self, sc: &StringCache ) -> String {
    match self.map_fold_expr( &mut ExprFormatter::new( sc ) ) {
      Ok( s ) => s,
      Err( error ) => error.msg
    }
  }
}

impl<T, E> ExprMapFolderTgt<T, E> for Expr {
  fn map_fold_expr<V: ExprMapFolder<T, E>>( &self, visitor: &mut V ) ->  Result<T, E> {
    match visitor.map_expr( &self ) {
      MapFolderState::Complete( result ) => result,
      MapFolderState::Incomplete => match self {
        Self::Assignment( var, right ) => {
          let rv = right.map_fold_expr( visitor )?;
          visitor.fold_expr_assignment( var, rv )
        },
        Self::Binary( left, op , right ) => {
          let lv = left.map_fold_expr( visitor )?;
          let rv = right.map_fold_expr( visitor )?;
          visitor.fold_expr_binary( lv, op, rv )
        },
        Self::Call( callee, paren , args ) => {
          let cv = callee.map_fold_expr( visitor )?;
          let mut avs: Vec<T> = Vec::new();
          for arg in args {
            let av = arg.map_fold_expr( visitor )?;
            avs.push( av );
          }
          visitor.fold_expr_call( cv, paren, &avs )
        }
        Self::Grouping( inner ) => {
          let iv = inner.map_fold_expr( visitor )?;
          visitor.fold_expr_grouping( iv )
        },
        Self::Literal( literal ) => {
          visitor.fold_expr_literal( literal )
        },
        Self::Unary( op, expr ) => {
          let ev = expr.map_fold_expr( visitor )?;
          visitor.fold_expr_unary( op, ev )
        },
        Self::Symbol( op ) => {
          visitor.fold_expr_symbol( op )
        }
      }
    }
  }
}

impl<E> ExprVisitorTgt<E> for Expr {
  fn accept<V: ExprVisitor<E>>( &self, visitor: &mut V ) -> Result<(), E> {
    visitor.visit_expr( self )?;
    if visitor.before_expr_children( self )? == VisitorControl::SkipChildren {
      return Ok( () );
    }
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
    visitor.after_expr_children( self );
    Ok( () )
  }
}