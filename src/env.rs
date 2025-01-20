

use std::collections::HashMap;

use crate::{eval::Eval, util::RcMut, token::Token};

#[derive(Clone)]
pub struct Env {
  enclosing: Option<RcMut<Env>>,
  values: HashMap<String, Eval>
}

impl Env {
  pub fn new() -> Env {
    Env {
      enclosing: None,
      values: HashMap::new()
    }
  }
  pub fn from_enclosing( env: &RcMut<Env> ) -> Env {
    Env {
      enclosing: Some( env.clone() ),
      values: HashMap::new()
    }
  }
  pub fn add_local( &mut self, name: Token, value: Eval ) {
    self.values.insert( name.lexeme, value );
  }
  fn read_env( tgt: &RcMut<Env>, depth: usize ) -> RcMut<Env> {
    if depth == 0 {
      return tgt.clone();
    } else if let Some( env ) = &tgt.view().enclosing {
      return Self::read_env( env, depth - 1 );
    }
    println!( "Runtime error. Depth too big: '{}'", depth );
    panic!()
  }
  fn write_env( tgt: &RcMut<Env>, depth: usize ) -> RcMut<Env> {
    if depth == 0 {
      return tgt.clone();
    } else if let Some( env ) = &tgt.view().enclosing {
      return Self::write_env( env, depth - 1 );
    }
    println!( "Runtime error. Depth too big: '{}'", depth );
    panic!()
  }
  pub fn read_symbol_at( tgt: &RcMut<Env>, name: Token, depth: usize ) -> Eval {
    Self::read_env( tgt, depth ).view().read_symbol( name )
  }
  pub fn write_symbol_at( tgt: &RcMut<Env>, name: Token, depth: usize, value: Eval ) {
    Self::write_env( tgt, depth ).view_mut().write_symbol( name, value );
  }
  pub fn read_symbol( &self, name: Token ) -> Eval {
    if self.values.contains_key( &name.lexeme ) {
      return self.values.get( &name.lexeme ).unwrap().clone()
    } else if let Some( env ) = self.enclosing.as_ref() {
      return env.view().read_symbol( name )
    }
    println!( "Runtime error. Unknown variable: '{}'", name.lexeme );
    panic!()
  }
  pub fn write_symbol( &mut self, name: Token, value: Eval ) {
    if self.values.contains_key( &name.lexeme ) {
      self.values.insert( name.clone().lexeme, value );
    } else if let Some( env ) = self.enclosing.as_mut() {
      env.view_mut().write_symbol( name.clone(), value );
    }
    println!( "Runtime error. Unknown variable: '{}'", name.lexeme );
  }
}