///////////////////////////////////////////
// private module rlox::interpreter::env //
///////////////////////////////////////////


/////////
// use //
/////////

use std::borrow::{Borrow, BorrowMut};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;

use crate::util::*;
use crate::interpreter::eval::*;


//////////////////////
// public interface //
//////////////////////

#[derive(Clone)]
pub struct RcMut<T> {
  shared_ptr: Rc<RefCell<T>>
}

impl<T> RcMut<T> {

  pub fn new( t: T ) -> RcMut<T> {
    RcMut {
      shared_ptr: Rc::new( RefCell::new( t ) )
    }
  }

  pub fn view( &self ) -> Ref<T> {
    self.shared_ptr.as_ref().borrow()
  }

  pub fn view_mut( &mut self ) -> RefMut<T> {
    self.shared_ptr.as_ref().borrow_mut()
  }
}

#[derive(Clone)]
pub struct Env {
  db: HashMap<StringKey, Eval>,
  parent: Option<RcMut<Env>>,
  line: i32
}

impl Env {

  pub fn create_global() -> RcMut<Env> {
    RcMut::new(
      Env {
        db: HashMap::new(),
        parent: None,
        line: 0
      }
    )
  }

  pub fn enclose_new( parent: &RcMut<Env>, line: i32 ) -> RcMut<Env> {
    RcMut::new(
      Env {
        db: HashMap::new(),
        parent: Some( parent.clone() ),
        line
      }
    )
  }

  pub fn drop_enclosed( child: &RcMut<Env> ) -> RcMut<Env> {
    let parent = 
      child.view()
        .parent
        .as_ref()
        .expect( "Internal error: No parent. [ Env::drop_enclosed() is called on a child environment, so it should have a parent. Did you forget to call Env::enclose_new() previously?" )
        .clone();
    let _ = child;
    parent
  }

  pub fn clone_global( child: &RcMut<Env> ) -> RcMut<Env> {
    if child.view().is_global() {
      child.clone()
    } else {
      Env::clone_global( child.view().parent.as_ref().unwrap() )
    }
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

  pub fn read_var( &self, key: StringKey ) -> Eval {
    if self.has_var_here( key ) {
      self.db.get( &key ).unwrap().clone()
    }
    else if self.is_global() {
      panic!( "Internal error: Unknown key. [ The caller of get_var() assumes responsibility for verifying that the key exists. ]" )
    }
    else {
      self.get_parent().read_var( key )
    }
  }

  pub fn create_var( &mut self, key: StringKey, value: Eval ) {
    assert!( !self.has_var_here( key ),
      "Internal error: Known/duplicate key. [ The caller of create_var() assumes responsibility for verifying that a key is unique. ]"
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

  fn get_parent( &self ) -> Ref<Env> {    
    self.parent
      .as_ref()
      .expect( "Internal error: No parent. [ The caller of get_parent() assumes responsibility for verifying that a parent exists." )
      .view()    
  }

  fn get_parent_mut( &mut self ) -> RefMut<Env> {
    self.parent
      .as_mut()
      .expect( "Internal error: No parent. [ The caller of get_parent_mut() assumes responsibility for verifying that a parent exists." )
      .view_mut()
  }

  fn debug_print_here( &self, sm: &StringManager ) {
    print!( "\nEnv beginning on line {} has {} entries:", self.line, self.db.len() );
    for ( key, value ) in &self.db {
      print!( "\n  {} = {}", sm.gets( *key ), value );
    }
  }

  pub fn debug_print( &self, sm: &StringManager ) {
    self.debug_print_here( sm );
    if !self.is_global() {
      self.get_parent().debug_print( sm );
    }
  }

}
