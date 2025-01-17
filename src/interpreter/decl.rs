////////////////////////////////////////////
// private module rlox::interpreter::decl //
////////////////////////////////////////////


/////////
// use //
/////////

use crate::util::*;

use crate::interpreter::error::*;
use crate::interpreter::token::*;
use crate::interpreter::stmt::*;
use crate::interpreter::expr::*;
use crate::interpreter::scope_tree::*;
use crate::interpreter::visitor::*;


//////////////////
// declarations //
//////////////////

#[derive(Debug, Clone)]
pub enum Decl {
  Stmt( Stmt ),
  Fun( /* fun_name: */ Token, /* arg_names: */ Vec<Token>, /* body: */ Stmt /* ::Block */ ),
  Var( /* var_name: */ Token, /* init: */ Option<Expr> )
}

pub trait DeclMapFolder<T, E> {
  fn get_stmt_map_folder( &mut self ) -> &mut impl StmtMapFolder<T, E>;
  fn map_decl( &mut self, decl: &Decl ) -> MapFolderState<T, E>;
  fn fold_decl_stmt( &mut self, stmt: T ) -> Result<T, E>;
  fn fold_decl_fun( &mut self, fun_name: &Token, arg_names: &Vec<Token>, body: T ) -> Result<T, E>;
  fn fold_decl_var( &mut self, var_name: &Token, init: Option<T> ) -> Result<T, E>;
}

pub trait DeclMapFolderTgt<T, E> {
  fn map_fold_decl<V: DeclMapFolder<T, E>>( &self, visitor: &mut V ) ->  Result<T, E>;
}

pub trait DeclVisitor<E> {
  fn get_stmt_visitor( &mut self ) -> &mut impl StmtVisitor<E>;

  fn visit_decl( &mut self, node: &Decl ) -> Result<(), E>;
  fn before_decl_children( &mut self, decl: &Decl ) -> Result<VisitorControl, E>;
  fn after_decl_children( &mut self, decl: &Decl );
}

pub trait DeclVisitorTgt<E> {
  fn accept<V: DeclVisitor<E>>( &self, visitor: &mut V ) -> Result<(), E>;
}


/////////////////////
// implementations //
/////////////////////

impl Decl {
  pub fn add_to_scope_tree( &self, sc: &RcMut<StringCache>, scope_tree: &RcMut<ScopeTree> ) {
    match self.accept( &mut ScopeTreeBuilder::new( sc, scope_tree, 0 ) ) {
      Ok( _ ) => {},
      Err( e ) => Self::emit_error( &e ),
    }
  }
  fn emit_error( error: &Error ) {
    eprintln!( "[line {}] Error{}: {}", error.line, error.loc, error.msg );
  }
}

impl<T, E> DeclMapFolderTgt<T, E> for Decl {
  fn map_fold_decl<V: DeclMapFolder<T, E>>( &self, visitor: &mut V ) ->  Result<T, E> {
    match visitor.map_decl( self ) {
      MapFolderState::Complete( result ) => result,
      MapFolderState::Incomplete => match self {
        Decl::Stmt( stmt ) => {
          let sv = stmt.map_fold_stmt( visitor.get_stmt_map_folder() )?;
          visitor.fold_decl_stmt( sv )
        },
        Decl::Fun( fun_name, arg_names, body) => {
          let bv = body.map_fold_stmt( visitor.get_stmt_map_folder() )?;
          visitor.fold_decl_fun( fun_name, arg_names, bv )
        },
        Decl::Var( var_name, init ) => {
          let iv = match init {
            Some( expr )
              => Some( expr.map_fold_expr( visitor.get_stmt_map_folder().get_expr_map_folder() )? ),
            None => None,
          };
          visitor.fold_decl_var( var_name, iv )
        },
      },
    }
  }
}

impl<E> DeclVisitorTgt<E> for Decl {
  fn accept<V: DeclVisitor<E>>( &self, visitor: &mut V ) -> Result<(), E> {
    visitor.visit_decl( self )?;
    if visitor.before_decl_children( self )? == VisitorControl::SkipChildren {
      return Ok( () );
    }
    match self {
      Self::Stmt( stmt ) => {
        stmt.accept( visitor.get_stmt_visitor() )?;
      },
      Self::Fun( _, _, stmt ) => {
        stmt.accept( visitor.get_stmt_visitor() )?;
        
      },
      Self::Var( _, init ) => {
        if let Some( expr ) = init {
          expr.accept( visitor.get_stmt_visitor().get_expr_visitor() )?;
        }
      }
    }
    visitor.after_decl_children( self );
    Ok( () )
  }
}