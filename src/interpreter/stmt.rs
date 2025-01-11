////////////////////////////////////////////
// private module rlox::interpreter::stmt //
////////////////////////////////////////////


/////////
// use //
/////////

use crate::interpreter::expr::*;
use crate::interpreter::decl::*;


//////////////////////
// public interface //
//////////////////////

#[derive(Debug, Clone)]
pub enum CtrlFlowInit {
  VarDecl( Box<Decl> ),
  ExprStmt( Box<Stmt> )
}

#[derive(Debug, Clone)]
pub enum Stmt {
  Expr( Expr ),
  Print( Expr ),
  Block( /* decls: */ Vec<Decl>, /* line: */ i32 ),
  If( /* init: */ Option<CtrlFlowInit>, /* condition: */ Expr, /* then: */ Box<Stmt>, /* else: */ Option<Box<Stmt>> ),
  While( /* init: */ Option<CtrlFlowInit>, /* condition: */ Expr, /* body: */ Box<Stmt> ),
  For( /* init: */ Option<CtrlFlowInit>, /* condition: */ Option<Expr>, /* incr: */ Option<Expr>, /* body: */ Box<Stmt> ),
  Return( /* retval: */ Option<Expr> )
}

impl Stmt {
  pub fn get_type_name( &self ) -> &str {
    match self {
        Stmt::Expr( _ ) => "Stmt::Expr",
        Stmt::Print( _ ) => "Stmt::Print",
        Stmt::Block( _, _) => "Stmt::Block",
        Stmt::If( _ , _, _, _) => "Stmt::If",
        Stmt::While( _, _, _ ) => "Stmt::While",
        Stmt::For( _, _, _, _ ) => "Stmt::For",
        Stmt::Return( _ ) => "Stmt::Return"
    }
  }
}

pub trait StmtVisitorMutTgt<T, E> {
  fn map_fold_mut<V: StmtVisitorMut<T, E>>( &self, visitor: &mut V ) -> Result<T, E>;
}

impl<T, E> StmtVisitorMutTgt<T, E> for Stmt {
  fn map_fold_mut<V: StmtVisitorMut<T, E>>( &self, visitor: &mut V ) -> Result<T, E> {
    visitor.before_children( &self );
    let result = match self {
      Self::Expr( expr ) => {
        let ev = expr.map_fold_mut( &mut visitor.get_expr_visitor_mut() )?;
        visitor.fold_mut_expr( ev )
      },
      Self::Print( expr ) => {
        let ev = expr.map_fold_mut( &mut visitor.get_expr_visitor_mut() )?;
        visitor.fold_mut_print( ev )
      },
      Self::Block( decls, line ) => {
        let mut dvs: Vec<T> = Vec::new();
        for decl in decls {
          dvs.push( decl.map_fold_mut( &mut visitor.get_decl_visitor_mut() )? );
        }
        visitor.fold_mut_block( dvs, *line )
      },
      Self::If( init, condition, then, else_ ) => {
        let iv = match init {
          Some( CtrlFlowInit::VarDecl( decl) ) => Some( decl.map_fold_mut( &mut visitor.get_decl_visitor_mut() )? ),
          Some( CtrlFlowInit::ExprStmt( stmt) ) => Some( stmt.map_fold_mut( visitor )? ),
          None => None
        };
        let cv = condition.map_fold_mut( &mut visitor.get_expr_visitor_mut() )?;
        let tv = then.map_fold_mut( visitor )?;
        let ev = if let Some( stmt ) = else_ {
          Some( stmt.map_fold_mut( visitor )? )
        } else {
          None
        };
        visitor.fold_mut_if( iv, cv, tv, ev )
      },
      Self::While( init, condition, body ) => {
        let iv = match init {
          Some( CtrlFlowInit::VarDecl( decl) ) => Some( decl.map_fold_mut( &mut visitor.get_decl_visitor_mut() )? ),
          Some( CtrlFlowInit::ExprStmt( stmt) ) => Some( stmt.map_fold_mut( visitor )? ),
          None => None
        };
        let cv = condition.map_fold_mut( &mut visitor.get_expr_visitor_mut() )?;
        let bv = body.map_fold_mut( visitor )?;
        visitor.fold_mut_while( iv, cv, bv )
      },
      Self::For( init, condition, incr, body ) => {
        let iv = match init {
          Some( CtrlFlowInit::VarDecl( decl) ) => Some( decl.map_fold_mut( &mut visitor.get_decl_visitor_mut() )? ),
          Some( CtrlFlowInit::ExprStmt( stmt) ) => Some( stmt.map_fold_mut( visitor )? ),
          None => None
        };
        let cv = if let Some( cond ) = condition {
          Some( cond.map_fold_mut( &mut visitor.get_expr_visitor_mut() )? )
        } else {
          None
        };
        let incv = if let Some( inc ) = incr {
          Some( inc.map_fold_mut( &mut visitor.get_expr_visitor_mut() )? )
        } else {
          None
        };
        let bv = body.map_fold_mut( visitor )?;
        visitor.fold_mut_for( iv, cv, incv, bv )
      },
      Self::Return( expr ) => {
        let ev = if let Some( exp ) = expr {
          Some( exp.map_fold_mut( &mut visitor.get_expr_visitor_mut() )? )
        } else {
          None
        };
        visitor.fold_mut_return( ev )
      }
    }?;
    visitor.after_children( &self );
    Ok( result )
  }
}

pub trait StmtVisitorMut<T, E> {
  fn get_expr_visitor_mut( &mut self ) -> impl ExprVisitorMut<T, E>;
  fn get_decl_visitor_mut( &mut self ) -> impl DeclVisitorMut<T, E>;

  fn before_children( &mut self, stmt: &Stmt );
  fn after_children( &mut self, stmt: &Stmt );

  fn fold_mut_expr( &mut self, expr: T ) -> Result<T, E>;
  fn fold_mut_print( &mut self, expr: T ) -> Result<T, E>;
  fn fold_mut_block( &mut self, decls: Vec<T>, line: i32 ) -> Result<T, E>;
  fn fold_mut_if( &mut self, init: Option<T>, condition: T, then: T, else_: Option<T> ) -> Result<T, E>;
  fn fold_mut_while( &mut self, init: Option<T>, condition: T, body: T ) -> Result<T, E>;
  fn fold_mut_for( &mut self, init: Option<T>, condition: Option<T>, incr: Option<T>, body: T ) -> Result<T, E>;
  fn fold_mut_return( &mut self, expr: Option<T> ) -> Result<T, E>;
}