/////////////////////////////////////
// public module rlox::interpreter //
/////////////////////////////////////


////////////////
// submodules //
////////////////

mod token;
mod expr;
mod stmt;
mod decl;
mod parser;
mod scanner;
mod executor;
mod eval;
mod error;
mod env;


/////////
// use //
/////////

use std::fs;
use std::{io, io::BufRead, io::Write};

use crate::interpreter::eval::*;
use crate::interpreter::scanner::*;
use crate::interpreter::parser::*;
use crate::interpreter::executor::*;
use crate::util::*;


//////////////////////
// public interface //
//////////////////////

pub struct Interpreter {
  str_lookup: StringManager
}

impl Interpreter {

  pub fn new() -> Interpreter {
    Interpreter {
      str_lookup: StringManager::new()
    }
  }

  pub fn run_file( &mut self, path: &str ) {
    let file = fs::read_to_string( path );
    match file {
      Ok( src ) => {
        let ( eval, had_error ) = self.run( src );
        println!( "\n\n----------------------\nExecution finished with return value {}. ", eval );
        if had_error {
          print!( "Runtime errors were detected." );
        }
        else {
          print!( "No runtime errors detected." );
        }
      },
      Err( e ) => eprintln!( "Error reading file: {}", e )
    }
  }
  
  pub fn run_prompt( &mut self ) {
    let stdin = io::stdin();
      
    loop {
      print!( "\n> " );
      let _ = io::stdout().flush();
      let mut input = String::new();
      match stdin.lock().read_line( &mut input ) {
        Ok( _ ) => {
          let ( eval, had_error ) = self.run( input );
          if had_error {
            println!( "\nErr( {} )", eval );
          }
          else {
            println!( "\nOk( {} )", eval );
          }        
        },
        Err( e ) => eprintln!( "Error reading stdin: {}", e )
      }
    }
  }


  ////////////////////////////
  // private implementation //
  ////////////////////////////
  
  fn run( &mut self, src: String ) -> ( Eval, bool ) {

    // scanner / lexer
    let mut scanner = Scanner::new( &mut self.str_lookup );
    let ( tokens, had_error ) = scanner.scan( src );
    if had_error {
      return ( Eval::Nil, true );
    }

    // parser
    let mut parser = Parser::new( &mut self.str_lookup );
    let ( decls, had_error ) = parser.parse( tokens );
    if had_error {
      return ( Eval::Nil, true );
    }

    // executor
    let mut executor = Executor::new( &mut self.str_lookup );
    executor.execute( decls )
  }
  
}



