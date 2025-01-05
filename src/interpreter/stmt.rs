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

#[derive(Clone)]
pub enum CtrlFlowInit {
  VarDecl( Box<Decl> ),
  ExprStmt( Box<Stmt> )
}


#[derive(Clone)]
pub enum Stmt {
  Expr( Expr ),
  Print( Expr ),
  Block( /* decls: */ Vec<Decl>, /* line: */ i32 ),
  If( /* init: */ Option<CtrlFlowInit>, /* condition: */ Expr, /* then: */ Box<Stmt>, /* else: */ Option<Box<Stmt>> ),
  While( /* init: */ Option<CtrlFlowInit>, /* condition: */ Expr, /* body: */ Box<Stmt> ),
  For( /* init: */ Option<CtrlFlowInit>, /* condition: */ Option<Expr>, /* incr: */ Option<Expr>, /* body: */ Box<Stmt> )
}