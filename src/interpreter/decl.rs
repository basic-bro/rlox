////////////////////////////////////////////
// private module rlox::interpreter::decl //
////////////////////////////////////////////


/////////
// use //
/////////

use crate::interpreter::token::*;
use crate::interpreter::stmt::*;
use crate::interpreter::expr::*;


//////////////////////
// public interface //
//////////////////////

#[derive(Debug, Clone)]
pub enum Decl {
  Stmt( Stmt ),
  Fun( /* fun_name: */ Token, /* arg_names: */ Vec<Token>, /* body: */ Stmt /* ::Block */ ),
  Var( /* var_name: */ Token, /* init: */ Option<Expr> )
}

pub trait DeclVisitorMutTgt<T, E> {
  fn map_fold_mut<V: DeclVisitorMut<T, E>>( &self, visitor: &mut V ) -> Result<T, E>;
}

impl<T, E> DeclVisitorMutTgt<T, E> for Decl {
  fn map_fold_mut<V: DeclVisitorMut<T, E>>( &self, visitor: &mut V ) -> Result<T, E> {
    visitor.before_children( &self );
    let result = match self {
      Self::Stmt( stmt ) => {
        let sv = stmt.map_fold_mut( &mut visitor.get_stmt_visitor_mut() )?;
        visitor.fold_mut_stmt( sv )
      },
      Self::Fun( fun_name, args, stmt ) => {
        let sv = stmt.map_fold_mut( &mut visitor.get_stmt_visitor_mut() )?;
        visitor.fold_mut_fun( *fun_name, args.clone(), sv )
      },
      Self::Var( var_name, init ) => {
        let iv = if let Some( expr ) = init {
          Some( expr.map_fold_mut( &mut visitor.get_stmt_visitor_mut().get_expr_visitor_mut() )? )
        } else {
          None
        };
        visitor.fold_mut_var( *var_name, iv )
      }
    }?;
    visitor.after_children( &self );
    Ok( result )
  }
}

pub trait DeclVisitorMut<T, E> {
  fn get_stmt_visitor_mut( &mut self ) -> impl StmtVisitorMut<T, E>;

  fn before_children( &mut self, decl: &Decl );
  fn after_children( &mut self, decl: &Decl );

  fn fold_mut_stmt( &mut self, stmt: T ) -> Result<T, E>;
  fn fold_mut_fun( &mut self, fun_name: Token, arg_names: Vec<Token>, body: T ) -> Result<T, E>;
  fn fold_mut_var( &mut self, var_name: Token, init: Option<T> ) -> Result<T, E>;
}

