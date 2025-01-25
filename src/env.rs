

use std::collections::HashMap;

use crate::{eval::Eval, token::Token, util::{assert, RcMut}};

#[derive(Clone)]
pub struct Env {
  enclosing: Option<RcMut<Env>>,
  values: HashMap<String, Eval>,
  depth: usize
}

impl Env {
  pub fn create_global() -> RcMut<Env> {
    RcMut::new(
      Env {
        enclosing: None,
        values: HashMap::new(),
        depth: 0
      }
    )
  }
  pub fn new_with_enclosing( enclosing: &RcMut<Env> ) -> RcMut<Env> {
    // println!( "Enclosing new scope at depth {}", enclosing.view().depth + 1 );
    RcMut::new(
      Env {
        enclosing: Some( enclosing.clone() ),
        values: HashMap::new(),
        depth: enclosing.view().depth + 1
      }
    )
  }
  pub fn drop_enclosed( env: &RcMut<Env> ) -> RcMut<Env> {
    assert( env.view().depth > 0, "Internal error: Cannot drop global scope.".into() );
    let result = env.view().enclosing.as_ref().unwrap().clone();
    // println!( "Dropping scope at depth {}", env.view().depth );
    let _ = env;
    result
  }
  pub fn read_symbol_at( &self, name: &Token, jump: usize ) -> Eval {
    assert( jump <= self.depth,
      format!( "Cannot jump {} env(s) when my depth is {}!",
        jump, self.depth ) );
    if jump == 0 {
      self.read_symbol( name )
    } else {
      self.enclosing.as_ref().unwrap().view().read_symbol_at( name, jump - 1 )
    }
  }
  pub fn write_symbol_at( &mut self, name: &Token, jump: usize, value: &Eval ) {
    assert( jump <= self.depth,
      format!( "Cannot jump {} env(s) when my depth is {}!",
        jump, self.depth ) );
    if jump == 0 {
      self.write_symbol( name, value )
    } else {
      self.enclosing.as_mut().unwrap().view_mut().write_symbol_at( name, jump - 1, value )
    }
  }
  fn read_symbol( &self, name: &Token ) -> Eval {
    assert( self.values.contains_key( &name.lexeme ), 
      format!( "Internal error: Key '{}' not found at depth {} for reading. Was the symbol created?", name.lexeme, self.depth ) );
    return self.values.get( &name.lexeme ).unwrap().clone()
  }
  fn write_symbol( &mut self, name: &Token, value: &Eval ) {
    assert( self.values.contains_key( &name.lexeme ), 
      format!( "Internal error: Key '{}' not found at depth {} for writing. Was the symbol created?", name.lexeme, self.depth ) );
    self.values.insert( name.lexeme.clone(), value.clone() );
  }
  pub fn create_symbol( &mut self, name: &Token, value: &Eval ) {
    assert( !self.values.contains_key( &name.lexeme ), 
      format!( "Internal error: Creating symbol '{}' at depth {}, but it already exists.", name.lexeme, self.depth ) );
    self.values.insert( name.lexeme.clone(), value.clone() );
  }
}