////////////////////////////////////////////
// private module rlox::interpreter::decl //
////////////////////////////////////////////


/////////
// use //
/////////

use crate::interpreter::token::*;
use crate::interpreter::stmt::*;
use crate::interpreter::expr::*;


//////////////////
// declarations //
//////////////////
#[derive(Debug, Clone)]

pub enum Decl {
  Stmt( Stmt ),
  Fun( /* fun_name: */ Token, /* arg_names: */ Vec<Token>, /* body: */ Stmt /* ::Block */ ),
  Var( /* var_name: */ Token, /* init: */ Option<Expr> )
}

pub trait DeclVisitor<E> {
  fn get_stmt_visitor( &mut self ) -> impl StmtVisitor<E>;

  fn visit( &mut self, node: &Decl ) -> Result<(), E>;
  fn before_children( &mut self, decl: &Decl );
  fn after_children( &mut self, decl: &Decl );
}

pub trait DeclVisitorTgt<E> {
  fn accept<V: DeclVisitor<E>>( &self, visitor: &mut V ) -> Result<(), E>;
}


/////////////////////
// implementations //
/////////////////////

impl<E> DeclVisitorTgt<E> for Decl {
  fn accept<V: DeclVisitor<E>>( &self, visitor: &mut V ) -> Result<(), E> {
    visitor.visit( self )?;
    visitor.before_children( self );
    match self {
      Self::Stmt( stmt ) => {
        stmt.accept( &mut visitor.get_stmt_visitor() )?;
      },
      Self::Fun( _, _, stmt ) => {
        stmt.accept( &mut visitor.get_stmt_visitor() )?;
        
      },
      Self::Var( _, init ) => {
        if let Some( expr ) = init {
          expr.accept( &mut visitor.get_stmt_visitor().get_expr_visitor() )?;
        }
      }
    }
    visitor.after_children( self );
    Ok( () )
  }
}