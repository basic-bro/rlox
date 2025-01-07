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
  Fun( /* fun_name: */ Token, /* arg_names: */ Vec<Token>, /* body: */ Stmt ),
  Var( Token /* identifier */, Option<Expr> /* initialiser */ )
}