use std::{fs, io::{self, Write, BufRead}};

mod util;
mod token;
mod expr;
mod stmt;
mod scanner;
mod parser;
mod error;
mod interpreter;
mod env;
mod resolver;
mod eval;

use crate::eval::Eval;
use crate::scanner::Scanner;
use crate::parser::Parser;
// use crate::interpreter::Interpreter;

// pub fn new() -> Interpreter {
//   let sc = RcMut::new( StringCache::new() );
//   Interpreter {
//     sc: sc.clone(),
//     scanner: Scanner::new( sc.clone() ),
//     parser: Parser::new( sc.clone() ),
//   }
// }


pub fn run_file( path: &str ) {
  let file = fs::read_to_string( path );
  match file {
    Ok( src ) => {
      let ( eval, had_error ) = run( src );
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

pub fn run_prompt() {
  let stdin = io::stdin();   
  loop {
    print!( "\n> " );
    let _ = io::stdout().flush();
    let mut input = String::new();
    match stdin.lock().read_line( &mut input ) {
      Ok( _ ) => {
        let ( eval, had_error ) = run( input );
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

fn run( src: String ) -> ( Eval, bool ) {

  // scanner / lexer
  let mut scanner = Scanner::new();
  let ( tokens, had_error ) = scanner.scan( src );
  if had_error {
    return ( Eval::Nil, true );
  }

  // parser
  let mut parser = Parser::new();
  let ( decls, had_error ) = parser.parse( tokens );
  // if had_error {
    // return ( Eval::Nil, true );
  // }

  ( Eval::Nil, had_error )

  

//   // ast
//   let mut ast = AST::new( &self.sc );
//   ast.add_decls( decls );
//   let scope_tree = match ast.build_scope_tree() {
//       Some( tree ) => tree,
//       None => return ( Eval::Nil, true ),
//   };

//   // module
//   let module = Module::new( ast, scope_tree );
//   module.exec()

//   // executor
//   // let mut executor = Executor::new( &mut self.str_lookup );
//   // executor.exec( decls )
}