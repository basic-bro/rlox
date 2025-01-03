///////////////////////////////////////////
// private module rlox::interpreter::env //
///////////////////////////////////////////


/////////
// use //
/////////

use std::collections::HashMap;

use crate::util::*;
use crate::interpreter::eval::*;


//////////////////////
// public interface //
//////////////////////

// 'enc == enclosing scope
pub struct Env {
  db: HashMap<StringKey, Eval>,
  parent: Option<Box<Env>>
}

impl Env {

  pub fn create_global() -> Box<Env> {
    Box::new(
      Env {
        db: HashMap::new(),
        parent: None
      }
    )
  }

  pub fn enclose_new( parent: Box<Env> ) -> Box<Env> {
    Box::new(
      Env {
        db: HashMap::new(),
        parent: Some( parent )
      }
    )
  }

  pub fn is_global( &self ) -> bool {
    self.parent.is_none()
  }

  pub fn has_var_here( &self, key: StringKey ) -> bool {
    self.db.contains_key( &key )
  }

  // recursive!
  pub fn has_var( &self, key: StringKey ) -> bool {
    let result = self.has_var_here( key );
    if self.is_global() {
      result
    }
    else {
      result || self.get_parent().has_var( key )
    }
  }

  pub fn read_var( &self, key: StringKey ) -> &Eval {
    if self.has_var_here( key ) {
      self.db.get( &key ).unwrap()
    }
    else if self.is_global() {
      panic!( "Internal error: Unknown key. [ The caller of get_var() assumes responsibility for verifying that the key exists. ]" )
    }
    else {
      self.get_parent().read_var( key )
    }
  }

  pub fn create_var( &mut self, key: StringKey, value: Eval ) {
    assert!( !self.has_var( key ),
      "Internal error: Known/duplicate key. [ The caller of add_var() assumes responsibility for verifying that a key is unique."
    );
    self.db.insert( key, value );
  }

  pub fn write_var( &mut self, key: StringKey, value: Eval ) {
    if self.has_var_here( key ) {
      self.db.insert( key, value );
    }
    else if self.is_global() {
      panic!( "Internal error: Unknown key. [ The caller of set_var() assumes responsibility for verifying that the key exists. ]" )
    }
    else {
      self.get_parent_mut().write_var( key, value );
    }
  }

  fn get_parent( &self ) -> &Box<Env> {
    self.parent.as_ref().expect( "Internal error: No parent. [ The caller of get_parent() assumes responsibility for verifying that a parent exists." )
  }

  fn get_parent_mut( &mut self ) -> &mut Box<Env> {
    self.parent.as_mut().expect( "Internal error: No parent. [ The caller of get_parent_mut() assumes responsibility for verifying that a parent exists." )
  }

}
