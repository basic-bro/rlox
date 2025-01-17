////////////////////////////////////////////
// private module rlox::interpreter::stmt //
////////////////////////////////////////////


/////////
// use //
/////////

use crate::interpreter::expr::*;
use crate::interpreter::decl::*;
use crate::interpreter::visitor::*;


//////////////////
// declarations //
//////////////////

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

pub trait StmtMapFolder<T, E> {
  fn get_decl_map_folder( &mut self ) -> &mut impl DeclMapFolder<T, E>;
  fn get_expr_map_folder( &mut self ) -> &mut impl ExprMapFolder<T, E>;
  fn map_stmt( &mut self, stmt: &Stmt ) -> MapFolderState<T, E>;
  fn fold_stmt_expr( &mut self, expr: T ) -> Result<T, E>;
  fn fold_stmt_print( &mut self, expr: T ) -> Result<T, E>;
  fn fold_stmt_block( &mut self, decls: Vec<T>, line: &i32 ) -> Result<T, E>;
  fn fold_stmt_if( &mut self, init: Option<T>, condition: T, then: T, else_: Option<T> ) -> Result<T, E>;
  fn fold_stmt_while( &mut self, init: Option<T>, condition: T, body: T ) -> Result<T, E>;
  fn fold_stmt_for( &mut self, init: Option<T>, condition: Option<T>, incr: Option<T>, body: T ) -> Result<T, E>;
  fn fold_stmt_return( &mut self, expr: Option<T> ) -> Result<T, E>;
}

pub trait StmtMapFolderTgt<T, E> {
  fn map_fold_stmt<V: StmtMapFolder<T, E>>( &self, visitor: &mut V ) ->  Result<T, E>;
}

pub trait StmtVisitor<E> {
  fn get_expr_visitor( &mut self ) -> &mut impl ExprVisitor<E>;
  fn get_decl_visitor( &mut self ) -> &mut impl DeclVisitor<E>;

  fn visit_stmt( &mut self, node: &Stmt ) -> Result<(), E>;
  fn before_stmt_children( &mut self, stmt: &Stmt ) -> Result<VisitorControl, E>;
  fn after_stmt_children( &mut self, stmt: &Stmt );
}

pub trait StmtVisitorTgt<E> {
  fn accept<V: StmtVisitor<E>>( &self, visitor: &mut V ) -> Result<(), E>;
}


/////////////////////
// implementations //
/////////////////////

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

impl<T, E> StmtMapFolderTgt<T, E> for Stmt {
  fn map_fold_stmt<V: StmtMapFolder<T, E>>( &self, visitor: &mut V ) ->  Result<T, E> {
    match visitor.map_stmt( self ) {
      MapFolderState::Complete( result ) => result,
      MapFolderState::Incomplete => match self {
        Self::Expr( expr ) => {
          let ev = expr.map_fold_expr( visitor.get_expr_map_folder() )?;
          visitor.fold_stmt_expr( ev )
        },
        Self::Print( expr ) => {
          let ev = expr.map_fold_expr( visitor.get_expr_map_folder() )?;
          visitor.fold_stmt_print( ev )
        },
        Self::Block( decls, line ) => {
          let mut dvs: Vec<T> = Vec::new();
          for decl in decls {
            dvs.push( decl.map_fold_decl( visitor.get_decl_map_folder() )? );
          }
          visitor.fold_stmt_block( dvs, line )
        },
        Self::If( init, condition, then, else_ ) => {
          let iv = match init {
            Some( CtrlFlowInit::VarDecl( decl) )
              => Some( decl.map_fold_decl( visitor.get_decl_map_folder() )? ),
            Some( CtrlFlowInit::ExprStmt( stmt) )
              => Some( stmt.map_fold_stmt( visitor )? ),
            None => None
          };
          let cv = condition.map_fold_expr( visitor.get_expr_map_folder() )?;
          let tv = then.map_fold_stmt( visitor )?;
          let ev = match else_ {
            Some( stmt ) => Some( stmt.map_fold_stmt( visitor )? ),
            None => None,
          };
          visitor.fold_stmt_if( iv, cv, tv, ev )
        },
        Self::While( init, condition, body ) => {
          let iv = match init {
            Some( CtrlFlowInit::VarDecl( decl) )
              => Some( decl.map_fold_decl( visitor.get_decl_map_folder() )? ),
            Some( CtrlFlowInit::ExprStmt( stmt) )
              => Some( stmt.map_fold_stmt( visitor )? ),
            None => None
          };
          let cv = condition.map_fold_expr( visitor.get_expr_map_folder() )?;
          let bv = body.map_fold_stmt( visitor )?;
          visitor.fold_stmt_while( iv, cv, bv )
        },
        Self::For( init, condition, incr, body ) => {
          let iv = match init {
            Some( CtrlFlowInit::VarDecl( decl) )
              => Some( decl.map_fold_decl( visitor.get_decl_map_folder() )? ),
            Some( CtrlFlowInit::ExprStmt( stmt) )
              => Some( stmt.map_fold_stmt( visitor )? ),
            None => None
          };
          let cv = match condition {
            Some( expr )
              => Some( expr.map_fold_expr( visitor.get_expr_map_folder() )? ),
            None => None,
          };
          let incv = match incr {
            Some( expr )
              => Some( expr.map_fold_expr( visitor.get_expr_map_folder() )? ),
            None => None,
          };
          let bv = body.map_fold_stmt( visitor )?;
          visitor.fold_stmt_for( iv, cv, incv, bv )
        },
        Self::Return( exp ) => {
          let ev = match exp {
            Some( expr ) => Some( expr.map_fold_expr( visitor.get_expr_map_folder() )? ),
            None => None
          };
          visitor.fold_stmt_return( ev )
        }
      }
    }
  }
}

impl<E> StmtVisitorTgt<E> for Stmt {
  fn accept<V: StmtVisitor<E>>( &self, visitor: &mut V ) -> Result<(), E> {
    visitor.visit_stmt( self )?;
    if visitor.before_stmt_children( self )? == VisitorControl::SkipChildren {
      return Ok( () );
    }
    match self {
      Self::Expr( expr ) => {
        expr.accept( visitor.get_expr_visitor() )?;
      },
      Self::Print( expr ) => {
        expr.accept( visitor.get_expr_visitor() )?;
      },
      Self::Block( decls, _line ) => {
        for decl in decls {
          decl.accept( visitor.get_decl_visitor() )?;
        }
      },
      Self::If( init, condition, then, else_ ) => {
        match init {
          Some( CtrlFlowInit::VarDecl( decl) )
            => decl.accept( visitor.get_decl_visitor() )?,
          Some( CtrlFlowInit::ExprStmt( stmt) )
            => stmt.accept( visitor )?,
          None => {}
        }
        condition.accept( visitor.get_expr_visitor() )?;
        then.accept( visitor )?;
        if let Some( stmt ) = else_ {
          stmt.accept( visitor )?;
        }
      },
      Self::While( init, condition, body ) => {
        match init {
          Some( CtrlFlowInit::VarDecl( decl) )
            => decl.accept( visitor.get_decl_visitor() )?,
          Some( CtrlFlowInit::ExprStmt( stmt) )
            => stmt.accept( visitor )?,
          None => {}
        }
        condition.accept( visitor.get_expr_visitor() )?;
        body.accept( visitor )?;
      },
      Self::For( init, condition, incr, body ) => {
        match init {
          Some( CtrlFlowInit::VarDecl( decl) )
            => decl.accept( visitor.get_decl_visitor() )?,
          Some( CtrlFlowInit::ExprStmt( stmt) )
            => stmt.accept( visitor )?,
          None => {}
        };
        if let Some( cond ) = condition {
          cond.accept( visitor.get_expr_visitor() )?;
        }
        if let Some( inc ) = incr {
          inc.accept( visitor.get_expr_visitor() )?;
        }
        body.accept( visitor )?;
      },
      Self::Return( expr ) => {
        if let Some( exp ) = expr {
          exp.accept( visitor.get_expr_visitor() )?;
        }
      }
    }
    visitor.after_stmt_children( &self );
    Ok( () )
  }
}