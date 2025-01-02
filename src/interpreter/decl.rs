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

#[derive(Clone)]
pub enum Decl {
  Var( Token /* identifier */, Option<Expr> /* initialiser */ ),
  Stmt( Stmt )
}