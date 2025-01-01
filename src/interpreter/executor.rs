////////////////////////////////////////////////
// private module rlox::interpreter::executor //
////////////////////////////////////////////////


use crate::interpreter::error::*;
use crate::interpreter::stmt::*;
use crate::interpreter::eval::*;
use crate::interpreter::expr::*;

use crate::util::StringManager;




pub struct Executor<'str> {
  db: &'str StringManager,
  had_error: bool
}

impl<'str> Executor<'str> {

  pub fn new( db: &'str StringManager ) -> Executor<'str> {
    Executor{
      db,
      had_error: false
    }  
  }

  pub fn execute( &mut self, stmts: Vec<Box<Stmt>> ) -> ( Eval, bool ) {
    self.restart();
    let mut retval = Eval::Nil;
    for stmt in stmts {
      match self.execute_stmt( &stmt ) {
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

  fn eval( &self, expr: &Expr ) -> EvalResult {
    expr.visit( &ExprEvaluator::new( self.db ) )
  }

  fn execute_stmt( &self, stmt: &Stmt ) -> EvalResult {
    let result = self.eval( stmt.get_expr() )?;
    match stmt {
      Stmt::Print( _ ) => {
        print!( "{}", result );
        Ok( result )
      },
      _ => Ok( result )
    }
  }

  fn emit_error( &mut self, error: &Error ) {
    eprintln!( "[line {}] Runtime error{}: {}", error.line, error.loc, error.msg );
    self.had_error = true;
  }


}