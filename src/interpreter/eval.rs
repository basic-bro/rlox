////////////////////////////////////////////
// private module rlox::interpreter::eval //
////////////////////////////////////////////


/////////
// use //
/////////

use std::fmt::Display;

use crate::interpreter::stmt::*;

use crate::util::*;


//////////////////
// declarations //
//////////////////

#[derive(Debug, Clone)]
pub enum Eval {
  Number( f64 ),
  StringLiteral( String ),
  Bool( bool ),
  Nil,
  Fun( /* name: */ StringKey, /* args: */ Vec<StringKey>, /* body: */ Stmt )
}


/////////////////////
// implementations //
/////////////////////

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
      Eval::Fun( _, args, _ ) => format!( "Fun<{}>", args.len() )
    }
  }
}

impl Display for Eval {
  fn fmt( &self, f: &mut std::fmt::Formatter<'_> ) -> std::fmt::Result {
    match self {
      Self::Number( x ) => write!( f, "{}", x ),
      Self::StringLiteral( s ) => write!( f, "{}", s ),
      Self::Bool( b ) => write!( f, "{}", b ),
      Self::Nil => write!( f, "nil" ),
      Self::Fun( _, args, _ ) => write!( f, "fun<{}>", args.len() )
    }
  }
}