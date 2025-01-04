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
pub enum Stmt {
  Expr( Expr ),
  Print( Expr ),
  Block( Vec<Decl>, i32 )
}

impl Stmt {

  pub fn get_expr( &self ) -> &Expr {
    match self {
      Stmt::Expr( expr ) => expr,
      Stmt::Print( expr ) => expr,
      Stmt::Block( _, _ ) => panic!( "Internal error: No Expr available. [ The caller of get_expr() assumes responsibility of checking that an Expr exists. ]" )
    }
  }

}