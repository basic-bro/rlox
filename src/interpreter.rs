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

pub fn run_file( path: &str ) {
  let file = fs::read_to_string( path );
  match file {
    Ok( src ) => {
      run( src );
        unsafe { if HAD_ERROR { process::exit( 65 ); } }
    },
    Err( e ) => eprintln!( "Error: {}", e )
  }
}

pub fn run_prompt() {
  let stdin = io::stdin();
    
  loop {
    print!( "> " );
    let _ = io::stdout().flush();
    let mut input = String::new();
    match stdin.lock().read_line( &mut input ) {
      Ok( _ ) => run( input ),
      Err( e ) => eprintln!( "Error: {}", e )
    }
  }
}


////////////////////////////
// private implementation //
////////////////////////////

static mut HAD_ERROR: bool = false;

fn error( line: i32, message: String ) {
  report( line, "".to_string(), message );
}

fn report( line: i32, where_: String, message: String ) {
    eprintln!( "[line {}] Error{}: {}", line, where_, message );
    unsafe{ HAD_ERROR = true; }
}

fn run( src: String ) {
  let mut scanner = Scanner::new( &src );
  let tokens = scanner.scan_tokens().clone();
  let mut parser = Parser::new( tokens );
  parser.parse();
  
}