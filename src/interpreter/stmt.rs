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