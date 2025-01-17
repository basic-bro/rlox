///////////////////////////////////////////
// private module rlox::interpreter::ast //
///////////////////////////////////////////


/////////
// use //
/////////

use crate::interpreter::decl::*;
use crate::interpreter::scope_tree::*;
use crate::interpreter::eval::*;
use crate::interpreter::executor::*;
use crate::interpreter::error::*;
use crate::interpreter::runtime::*;

use crate::util::*;


//////////////////
// declarations //
//////////////////

pub struct AST {
  sc: RcMut<StringCache>,
  decls: Vec<Decl>
}


/////////////////////
// implementations //
/////////////////////

impl AST {
  pub fn new( sc: &RcMut<StringCache> ) -> AST {
    AST {
      sc: sc.clone(),
      decls: Vec::new()
    }
  }
  pub fn add_decl( &mut self, decl: Decl ) {
    self.decls.push( decl );
  }
  pub fn add_decls( &mut self, decls: Vec<Decl> ) {
    for decl in decls {
      self.add_decl( decl );
    }
  }
  pub fn build_scope_tree( &self ) -> Option<RcMut<ScopeTree>> {

    // build
    let mut scope_tree = Scope::new_scope_tree();
    for decl in &self.decls {
      decl.add_to_scope_tree( &self.sc, &scope_tree );
    }

    // resolve
    let result = Scope::resolve_scope_tree( &mut scope_tree, &self.sc );
    match result
    {
      Ok( _ ) => Some( scope_tree ),
      Err( e ) => {
        Self::emit_error( &e );
        None
      },
    }
  }
  pub fn exec( &self, scope_tree: RcMut<ScopeTree>, curr_scope: u64 ) -> ( Eval, bool ) {
    let mut result = ( Eval::Nil, false );
    let rt = RcMut::new( Runtime::new( &self.sc, &scope_tree, curr_scope, &self.decls ) );
    let mut executor = Executor::new( &self.sc, &rt );
    for decl in &self.decls {
      result = match decl.map_fold_decl( &mut executor ) {
        Ok( value ) => ( value, false ),
        Err( EvalError::Error( e ) ) => {
          Self::emit_error( &e );
          ( Eval::Nil, true )
        },
        Err( EvalError::Return( retval ) ) => {
          println!( "Internal error: A return statement should not be handled here." );
          ( retval, true )
        }
      };
      if result.1 {
        break;
      }
    }
    result
  }
  fn emit_error( error: &Error ) {
    eprintln!( "[line {}] Error{}: {}", error.line, error.loc, error.msg );
  }
}