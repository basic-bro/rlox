/////////////////////////////////////
// public module rlox::interpreter //
/////////////////////////////////////


////////////////
// submodules //
////////////////

mod token;
mod scanner;
mod expr;
mod parser;


/////////
// use //
/////////

use std::process;
use std::fs;
use std::{io, io::BufRead, io::Write};

use crate::interpreter::scanner::*;
use crate::interpreter::parser::*;


//////////////////////
// public interface //
//////////////////////

pub struct Interpreter {
  had_error: bool
}

impl Interpreter {

  pub fn new() -> Interpreter {
    Interpreter {
      had_error: false
    }
  }

  pub fn run_file( &mut self, path: &str ) {
    let file = fs::read_to_string( path );
    match file {
      Ok( src ) => {
        self.run( src );
        if self.had_error {
          process::exit( 65 );
        }
      },
      Err( e ) => eprintln!( "Error: {}", e )
    }
  }
  
  pub fn run_prompt( &mut self ) {
    let stdin = io::stdin();
      
    loop {
      print!( "> " );
      let _ = io::stdout().flush();
      let mut input = String::new();
      match stdin.lock().read_line( &mut input ) {
        Ok( _ ) => self.run( input ),
        Err( e ) => eprintln!( "Error: {}", e )
      }
    }
  }

  fn error( &mut self, line: i32, message: String ) {
    self.report( line, "".to_string(), message );
  }
  
  fn report( &mut self, line: i32, where_: String, message: String ) {
      eprintln!( "[line {}] Error{}: {}", line, where_, message );
      self.had_error = true;
  }
  
  fn run( &self, src: String ) {
    let mut scanner = Scanner::new( &src );
    let tokens = scanner.scan_tokens().clone();
    let mut parser = Parser::new( tokens );
    parser.parse();
    
  }


}



