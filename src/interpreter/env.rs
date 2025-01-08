///////////////////////////////////////////
// private module rlox::interpreter::env //
///////////////////////////////////////////


/////////
// use //
/////////

use std::cell::{Ref, RefMut};
use std::collections::HashMap;

use crate::util::*;
use crate::interpreter::eval::*;


//////////////////////
// public interface //
//////////////////////

pub struct EnvStack {
  stack: Stack<RcMut<Env>>
}

impl EnvStack {

  pub fn new() -> EnvStack {
    EnvStack {
      stack: Stack::new()
    }
  }

  pub fn enclose_new( &mut self, line: i32 ) {
    self.stack.push( RcMut::new( Env::new( line ) ) );
  }

  pub fn drop_enclosed( &mut self ) {
    self.stack.pop();
  }

  pub fn clone_global( &self ) -> EnvStack {
    let mut global = EnvStack::new();
    global.stack.push( self.stack.peek( self.stack.depth() - 1 ).clone() );
    global
  }

  fn peek( &self, depth: usize ) -> Ref<Env> {
    self.stack.peek( depth ).view()
  }

  fn peek_mut( &mut self, depth: usize ) -> RefMut<Env> {
    self.stack.peek_mut( depth ).view_mut()
  }

  fn curr( &self ) -> Ref<Env> {
    self.peek( 0 )
  }

  fn curr_mut( &mut self ) -> RefMut<Env> {
    self.peek_mut( 0 )
  }

  pub fn has_symbol_here( &self, symbol_key: StringKey ) -> bool {
    self.curr().has_symbol( symbol_key )
  }

  pub fn has_symbol( &self, symbol_key: StringKey ) -> bool {
    for depth in 0..self.stack.depth() {
      if self.peek( depth ).has_symbol(symbol_key ) {
        return true;
      }
    }
    false
  }

  pub fn create_symbol( &mut self, symbol_key: StringKey, eval: Eval ) {
    assert!( !self.has_symbol_here( symbol_key ) );
    self.curr_mut().create_symbol( symbol_key, eval );
  }

  pub fn write_symbol( &mut self, symbol_key: StringKey, eval: Eval ) {
    assert!( self.has_symbol( symbol_key ) );
    for depth in 0..self.stack.depth() {
      if self.peek( depth ).has_symbol( symbol_key ) {
        self.peek_mut( depth ).write_symbol( symbol_key, eval );
        return;
      }
    }
    assert!( false );
  }

  pub fn read_symbol( &self, symbol_key: StringKey ) -> Eval {
    assert!( self.has_symbol( symbol_key ) );
    for depth in 0..self.stack.depth() {
      if self.peek( depth ).has_symbol( symbol_key ) {
        return self.peek( depth ).read_symbol( symbol_key );
      }
    }
    assert!( false );
    Eval::Nil
  }

  pub fn debug_print( &self, sc: &StringCache ) {
    for depth in 0..self.stack.depth() {
      self.peek(depth).debug_print( sc );
    }
  }

}




#[derive(Clone)]
pub struct Env {
  db: HashMap<StringKey, Eval>,
  line: i32
}

impl Env {

  pub fn new( line: i32 ) -> Env {
    Env {
      db: HashMap::new(),
      line
    }
  }

  pub fn has_symbol( &self, key: StringKey ) -> bool {
    self.db.contains_key( &key )
  }

  pub fn read_symbol( &self, key: StringKey ) -> Eval {
    if self.has_symbol( key ) {
      self.db.get( &key ).unwrap().clone()
    }
    else {
      panic!( "Internal error: Unknown key. [ The caller of read_var() assumes responsibility for verifying that the key exists. ]" )
    }
  }

  pub fn create_symbol( &mut self, key: StringKey, value: Eval ) {
    assert!( !self.has_symbol( key ),
      "Internal error: Known/duplicate key. [ The caller of create_var() assumes responsibility for verifying that a key is unique. ]"
    );
    self.db.insert( key, value );
  }

  pub fn write_symbol( &mut self, key: StringKey, value: Eval ) {
    if self.has_symbol( key ) {
      self.db.insert( key, value );
    }
    else {
      panic!( "Internal error: Unknown key. [ The caller of write_var() assumes responsibility for verifying that the key exists. ]" )
    }
  }

  fn debug_print( &self, sc: &StringCache ) {
    print!( "\nEnv beginning on line {} has {} entries:", self.line, self.db.len() );
    for ( key, value ) in &self.db {
      print!( "\n  {} = {}", sc.gets( *key ), value );
    }
  }

}
