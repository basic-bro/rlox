////////////////////////////////////////////
// private module rlox::interpreter::stmt //
////////////////////////////////////////////


/////////
// use //
/////////

use crate::interpreter::expr::*;
use crate::interpreter::decl::*;


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

pub trait StmtVisitor<E> {
  fn get_expr_visitor( &mut self ) -> impl ExprVisitor<E>;
  fn get_decl_visitor( &mut self ) -> impl DeclVisitor<E>;

  fn visit( &self, node: &Stmt ) -> Result<(), E>;
  fn before_children( &mut self, stmt: &Stmt );
  fn after_children( &mut self, stmt: &Stmt );
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

impl<E> StmtVisitorTgt<E> for Stmt {
  fn accept<V: StmtVisitor<E>>( &self, visitor: &mut V ) -> Result<(), E> {
    visitor.visit( self )?;
    visitor.before_children( self );
    match self {
      Self::Expr( expr ) => {
        expr.accept( &mut visitor.get_expr_visitor() )?;
      },
      Self::Print( expr ) => {
        expr.accept( &mut visitor.get_expr_visitor() )?;
      },
      Self::Block( decls, line ) => {
        for decl in decls {
          decl.accept( &mut visitor.get_decl_visitor() )?;
        }
      },
      Self::If( init, condition, then, else_ ) => {
        match init {
          Some( CtrlFlowInit::VarDecl( decl) )
            => decl.accept( &mut visitor.get_decl_visitor() )?,
          Some( CtrlFlowInit::ExprStmt( stmt) )
            => stmt.accept( visitor )?,
          None => {}
        }
        condition.accept( &mut visitor.get_expr_visitor() )?;
        then.accept( visitor )?;
        if let Some( stmt ) = else_ {
          stmt.accept( visitor )?;
        }
      },
      Self::While( init, condition, body ) => {
        match init {
          Some( CtrlFlowInit::VarDecl( decl) )
            => decl.accept( &mut visitor.get_decl_visitor() )?,
          Some( CtrlFlowInit::ExprStmt( stmt) )
            => stmt.accept( visitor )?,
          None => {}
        }
        condition.accept( &mut visitor.get_expr_visitor() )?;
        body.accept( visitor )?;
      },
      Self::For( init, condition, incr, body ) => {
        match init {
          Some( CtrlFlowInit::VarDecl( decl) )
            => decl.accept( &mut visitor.get_decl_visitor() )?,
          Some( CtrlFlowInit::ExprStmt( stmt) )
            => stmt.accept( visitor )?,
          None => {}
        };
        if let Some( cond ) = condition {
          cond.accept( &mut visitor.get_expr_visitor() )?;
        }
        if let Some( inc ) = incr {
          inc.accept( &mut visitor.get_expr_visitor() )?;
        }
        body.accept( visitor )?;
      },
      Self::Return( expr ) => {
        if let Some( exp ) = expr {
          exp.accept( &mut visitor.get_expr_visitor() )?;
        }
      }
    }
    visitor.after_children( &self );
    Ok( () )
  }
}