////////////////////////////////////////////////
// private module rlox::interpreter::executor //
////////////////////////////////////////////////


/////////
// use //
/////////

use crate::interpreter::error::*;
use crate::interpreter::stmt::*;
use crate::interpreter::eval::*;
use crate::interpreter::decl::*;
use crate::interpreter::token::*;
use crate::interpreter::env::*;
use crate::interpreter::expr::*;

use crate::util::*;


//////////////////////
// public interface //
//////////////////////

pub struct Executor<'str> {
  db: &'str mut StringManager,
  env: Box<Env>,
  had_error: bool
}

impl<'str> Executor<'str> {

  pub fn new( db: &'str mut StringManager ) -> Executor<'str> {
    Executor{
      db,
      env: Env::create_global(),
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
      Decl::Stmt( Stmt::Block( _decls, _ ) ) => Ok( Eval::Nil ),
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

      // statement declaration
      Decl::Stmt( stmt ) =>
        match stmt {

          // print statement
          Stmt::Print( _ ) => {
            print!( "{}\n", result );
            Ok( result )
          },

          // expression statement
          Stmt::Expr( expr ) =>
            match expr {
              Expr::Assignment( var, rhs ) => self.execute_assignment_expr( var, rhs, result ),
              _ => Ok( result )
            },

          // block statement
          Stmt::Block( decls, line ) => {
            let mut final_result = Eval::Nil;


            // print!( "\nBefore Env::enclose_new()" );
            // self.env.debug_print( self.db );

            self.env = Env::enclose_new( &self.env, *line );

            // print!( "\nAfter Env::enclose_new()" );
            // self.env.debug_print( self.db );


            for decl in decls {
              final_result = self.execute_decl( decl )?;
            }

            // print!( "\nBefore Env::drop_enclosed()" );
            // self.env.debug_print( self.db );

            self.env = Env::drop_enclosed( &self.env );

            // print!( "\nAfter Env::drop_enclosed()" );
            // self.env.debug_print( self.db );

            Ok( final_result )
          }
        },
      
      // variable declaration
      Decl::Var( var, _ ) => {
        let key = var.get_key();

        // error on redefinition
        if self.env.has_var_here( key ) {
          return Err( self.make_error( var, "This variable is already in use.".to_string() ) );
        }

        self.env.create_var( key, result.clone() );
        Ok( result )
      }
    }
  }

  fn execute_assignment_expr( &mut self, var: &Token, rhs: &Expr, result: Eval ) -> EvalResult {

    // rhs might be a nested assignment expression
    match rhs {
      Expr::Assignment( nested_var, nested_rhs ) => {
        self.execute_assignment_expr( nested_var, nested_rhs, result.clone() )?;
      },
      _ => ()
    }

    // check variable has been declared
    let key = var.get_key();
    if self.env.has_var( key ) {
      self.env.write_var( key, result.clone() );
      Ok( result )
    }
    else {
      Err( self.make_error( var, "Undeclared variable.".to_string() ) )
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