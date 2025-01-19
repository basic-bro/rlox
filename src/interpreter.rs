/////////////////////////////////////
// public module rlox::interpreter //
/////////////////////////////////////


/////////
// use //
/////////

use std::collections::HashMap;

use crate::{env::Env, expr::Expr};


//////////////////
// declarations //
//////////////////

pub struct Interpreter {
  globals: Env,
  env: Env,
  locals: HashMap<Expr, usize>
}


/////////////////////
// implementations //
/////////////////////

impl Interpreter {

  pub fn add_local( &mut self, expr: Expr, depth: usize ) {
    self.locals.insert( expr, depth );
  }
}