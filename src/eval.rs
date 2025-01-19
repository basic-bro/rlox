use std::fmt::Display;

use crate::stmt::Function;





#[derive(Clone)]
pub enum Eval {
  Number( f64 ),
  StringLiteral( String ),
  Bool( bool ),
  Nil,
  Fun( Function )
}

impl Display for Eval {
  fn fmt( &self, f: &mut std::fmt::Formatter<'_> ) -> std::fmt::Result {
    match self {
      Eval::Number( x ) => write!( f, "{}", x ),
      Eval::StringLiteral( s ) => write!( f, "{}", s ),
      Eval::Bool( b ) => write!( f, "{}", b ),
      Eval::Nil => write!( f, "nil" ),
      Eval::Fun( function ) => write!( f, "{}<{}>()", function.name.lexeme, function.params.len() ),
    }
  }
}