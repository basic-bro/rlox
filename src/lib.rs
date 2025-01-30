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
mod byte_code;


use eval::Eval;
use scanner::Scanner;
use parser::Parser;
use resolver::Resolver;
use interpreter::Interpreter;
use byte_code::{Compiler, Vm};
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
  let ( tokens, had_scan_error ) = scanner.scan( src );
  if had_scan_error {
    return ( Eval::Nil, true );
  }

  // parser
  let mut parser = Parser::new();
  let ( mut stmts, had_parse_error ) = parser.parse( tokens );
  if had_parse_error {
    return ( Eval::Nil, true );
  }

  // resolver
  let mut resolver = Resolver::new();
  let had_resolve_error = resolver.resolve( &mut stmts );
  if had_resolve_error {
    return ( Eval::Nil, true );
  }

  let mut codegen = Compiler::new();
  let ( byte_code, had_codegen_error ) = codegen.compile( &stmts );
  if had_codegen_error {
    return ( Eval::Nil, true );
  }

  let mut vm = Vm::new( byte_code );
  vm.exec()

  // interpreter
  // let mut interpreter = Interpreter::new();
  // interpreter.interpret( &stmts )

  

  

//   // module
//   let module = Module::new( ast, scope_tree );
//   module.exec()

//   // executor
//   // let mut executor = Executor::new( &mut self.str_lookup );
//   // executor.exec( decls )
}