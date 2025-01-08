

use crate::util::*;
use crate::interpreter::decl::*;





pub struct Resolver<'str> {
  sc: &'str mut StringCache,
  had_error: bool
}


impl<'str> Resolver<'str> {

  pub fn new( sc: &'str mut StringCache ) -> Resolver<'str> {
    Resolver {
      sc,
      had_error: false
    }
  }

  pub fn resolve( &self, decls: Vec<Decl> ) {

  }

}