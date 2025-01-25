use std::fmt::Display;

use crate::{env::Env, stmt::Function, util::RcMut};

#[derive(Clone)]
pub enum Eval {
  Number( f64 ),
  StringLiteral( String ),
  Bool( bool ),
  Nil,
  Fun( Function, RcMut<Env> )
}

impl Eval {
  pub fn is_truthy( &self ) -> bool {
    match self {
      // "nil" and "false" are falsey
      Eval::Nil => false,
      Eval::Bool( false ) => false,
      // everything else is truthy
      _ => true
    }
  }
  pub fn get_type_name( &self ) -> String {
    match self {
      Eval::Number( _ ) => "Number".to_string(),
      Eval::StringLiteral( _ ) => "String".to_string(),
      Eval::Bool( _ ) => "Bool".to_string(),
      Eval::Nil => "Nil".to_string(),
      Eval::Fun( f, _ ) => format!( "fun<{}>", f.params.len() )
    }
  }
}

impl Display for Eval {
  fn fmt( &self, f: &mut std::fmt::Formatter<'_> ) -> std::fmt::Result {
    match self {
      Eval::Number( x ) => write!( f, "{}", x ),
      Eval::StringLiteral( s ) => write!( f, "{}", s ),
      Eval::Bool( b ) => write!( f, "{}", b ),
      Eval::Nil => write!( f, "nil" ),
      Eval::Fun( function, _ ) => write!( f, "{}<{}>()", function.name.lexeme, function.params.len() ),
    }
  }
}