////////////////////////////////////////////////
// private module rlox::interpreter::executor //
////////////////////////////////////////////////


/////////
// use //
/////////

use std::collections::HashMap;

use crate::interpreter::error::*;
use crate::interpreter::stmt::*;
use crate::interpreter::eval::*;
use crate::interpreter::decl::*;
use crate::interpreter::token::*;
use crate::interpreter::env::*;

use crate::util::*;


//////////////////////
// public interface //
//////////////////////

pub struct Executor<'str> {
  db: &'str mut StringManager,
  env: Env,
  had_error: bool
}

impl<'str> Executor<'str> {

  pub fn new( db: &'str mut StringManager ) -> Executor<'str> {
    Executor{
      db,
      env: Env::new(),
      had_error: false,
    }  
  }

  pub fn execute( &mut self, decls: Vec<Decl> ) -> ( Eval, bool ) {
    self.restart();
    let mut retval = Eval::Nil;
    for decl in decls {
      match self.execute_decl( &decl ) {
        Ok( val ) => retval = val,
        Err( error ) => self.emit_error( &error )
      }
    }
    ( retval, self.had_error )
  }
 

  ////////////////////////////
  // private implementation //
  ////////////////////////////
  
  fn restart( &mut self ) {
    self.had_error = false;
  }

  fn eval( &self, decl: &Decl ) -> EvalResult {
    match decl {
      Decl::Stmt( Stmt::Expr ( expr ) ) => expr.eval( self.db, &self.env ),
      Decl::Stmt( Stmt::Print( expr ) ) => expr.eval( self.db, &self.env ),
      Decl::Var( _, init ) => {
        match init {
          Some( expr ) => expr.eval( self.db, &self.env ),
          None => Ok( Eval::Nil )
        }
      }
    }
  }

  fn execute_decl( &mut self, decl: &Decl ) -> EvalResult {
    let result = self.eval( decl )?;
    match decl {

      // declaration statement
      Decl::Stmt( stmt ) =>
        match stmt {

          // print statement
          Stmt::Print( _ ) => {
            print!( "{}", result );
            Ok( result )
          },

          // expression statement
          Stmt::Expr( _ ) => Ok( result )
        },
      
      // variable declaration
      Decl::Var( t, _ ) => {
        let var_key = self.db.puts( &t.get_lexeme( self.db ).to_string() );

        // error on redefinition
        if self.env.contains_key( &var_key ) {
          return Err( self.make_error( t, "This variable is already in use.".to_string() ) );
        }

        self.env.insert( var_key, result.clone() );
        Ok( result )
      }
    }
  }

  fn make_error( &self, t: &Token, msg: String ) -> Error {
    Error::from_token( t, msg, self.db )
  }

  fn emit_error( &mut self, error: &Error ) {
    eprintln!( "[line {}] Runtime error{}: {}", error.line, error.loc, error.msg );
    self.had_error = true;
  }


}