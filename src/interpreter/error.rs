/////////////////////////////////////////////
// private module rlox::interpreter::error //
/////////////////////////////////////////////


/////////
// use //
/////////

use crate::interpreter::token::*;

use crate::util::*;


//////////////////
// declarations //
//////////////////

#[derive(Debug)]
pub struct Error {
  pub line: i32,
  pub loc: String,
  pub msg: String
}


/////////////////////
// implementations //
/////////////////////

impl Error {
  pub fn new( line: i32, loc: String, msg: String ) -> Error {
    Error {
      line,
      loc,
      msg
    }
  }
  pub fn from_token( t: &Token, msg: String, sc: &StringCache ) -> Error {
    Error {
      line: t.get_line(),
      loc: format!( " at '{}'", t.get_lexeme( sc ) ),
      msg
    }
  }
}