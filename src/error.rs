/////////////////////////////////////////////
// private module rlox::interpreter::error //
/////////////////////////////////////////////


/////////
// use //
/////////

use crate::token::Token;



//////////////////
// declarations //
//////////////////

#[derive(Debug)]
pub struct Error {
  pub line: u32,
  pub loc: String,
  pub msg: String
}


/////////////////////
// implementations //
/////////////////////

impl Error {
  // pub fn new( line: i32, loc: String, msg: String ) -> Error {
  //   Error {
  //     line,
  //     loc,
  //     msg
  //   }
  // }
  pub fn from_token( t: &Token, msg: String ) -> Error {
    Error {
      line: t.line,
      loc: format!( " at '{}'", t.lexeme ),
      msg
    }
  }
}