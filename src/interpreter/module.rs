//////////////////////////////////////////////
// private module rlox::interpreter::module //
//////////////////////////////////////////////


/////////
// use //
/////////

use crate::util::*;

use crate::interpreter::scope_tree::*;
use crate::interpreter::eval::*;
use crate::interpreter::ast::*;


//////////////////
// declarations //
//////////////////

pub struct Module {
  ast: AST,
  scope_tree: RcMut<ScopeTree>
}


/////////////////////
// implementations //
/////////////////////

impl Module {
  pub fn new( ast: AST, scope_tree: RcMut<ScopeTree> ) -> Module {
    Module {
      ast,
      scope_tree
    }
  }
  pub fn exec( &self ) -> ( Eval, bool ) {
    self.ast.exec( self.scope_tree.clone(), 0 )
  }
}