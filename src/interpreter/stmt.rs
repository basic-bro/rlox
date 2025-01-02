////////////////////////////////////////////
// private module rlox::interpreter::stmt //
////////////////////////////////////////////


/////////
// use //
/////////

use crate::interpreter::expr::*;


//////////////////////
// public interface //
//////////////////////

#[derive(Clone)]
pub enum Stmt {
  Expr( Expr ),
  Print( Expr )
}

impl Stmt {

  pub fn get_expr( &self ) -> &Expr {
    match self {
      Stmt::Expr( expr ) => expr,
      Stmt::Print( expr ) => expr
    }
  }

}