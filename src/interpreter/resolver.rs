////////////////////////////////////////////////
// private module rlox::interpreter::resolver //
////////////////////////////////////////////////


/////////
// use //
/////////

use crate::util::*;

use crate::interpreter::decl::*;
use crate::interpreter::scope_tree::*;
use crate::interpreter::format::*;


//////////////////
// declarations //
//////////////////

// traverses the AST and produces a scope tree
pub struct Resolver {
  sc: RcMut<StringCache>,
  scope_tree: RcMut<ScopeTree>,
  had_error: bool
}


/////////////////////
// implementations //
/////////////////////

impl Resolver {

  pub fn new( sc: RcMut<StringCache> ) -> Resolver {
    Resolver {
      sc,
      scope_tree: RcMut::new( Tree::new( Scope::new( 0 ) ) ),
      had_error: false
    }
  }

  pub fn resolve( &mut self, decls: &Vec<Decl> ) -> ( RcMut<ScopeTree>, bool ) {
    self.restart();

    self.resolve_decls( decls, 0 );
    
    let _ = self.scope_tree.view_mut().accept_mut( &mut ScopeTreeResolver::new(), 0, 0 );

    if let Result::Ok( s ) = self.scope_tree.view().map_fold( &ScopeTreeFormatter::new( &self.sc.view() ), 0, 0 ) {
      println!( "{}", s );
    }

    let scopes = self.scope_tree.clone();
    ( scopes, self.had_error )
  }

  fn restart( &mut self ) {
    self.scope_tree = RcMut::new( Tree::new( Scope::new( 0 ) ) );
    self.had_error = false;
  }

  fn resolve_decls( &mut self, decls: &Vec<Decl>, scope_key: u64 ) {
    for decl in decls {
      self.resolve_decl( decl, scope_key );
    }
  }

  fn resolve_decl( &mut self, decl: &Decl, scope_key: u64 ) {
    let _ = decl.accept( &mut ScopeTreeBuilder_Decl::new( self.scope_tree.clone(), scope_key ) );
  }
}