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
  sm: &'str mut StringManager,
  env: Box<Env>,
  had_error: bool
}

impl<'str> Executor<'str> {

  pub fn new( sm: &'str mut StringManager ) -> Executor<'str> {
    Executor{
      sm,
      env: Env::create_global(),
      had_error: false,
    }  
  }

  pub fn exec( &mut self, decls: Vec<Decl> ) -> ( Eval, bool ) {
    self.restart();
    let mut retval = Eval::Nil;
    for decl in decls {
      match self.exec_decl( &decl ) {
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

  fn exec_decl( &mut self, decl: &Decl ) -> EvalResult {
    //let result = self.eval_decl( decl )?;
    match decl {

      // statement declaration
      Decl::Stmt( stmt ) => self.exec_stmt( stmt ),
      
      // variable declaration
      Decl::Var( var, init ) => {
        let key = var.get_key();

        // error on redefinition
        if self.env.has_var_here( key ) {
          return Err( self.make_error( var, "This variable is already in use.".to_string() ) );
        }

        // evaluate initialiser
        let result = match init {
          Some( expr ) => expr.eval( self.sm, &self.env ),
          None => Ok( Eval::Nil )
        }?;

        // create variable
        self.env.create_var( key, result.clone() );
        Ok( result )
      }
    }
  }

  fn exec_stmt( &mut self, stmt: &Stmt ) -> EvalResult {
    match stmt {
      Stmt::Print( expr )
        => self.exec_print_stmt( expr ),
      Stmt::Expr( expr )
        => self.exec_expr_stmt( expr ),
      Stmt::Block( decls, line )
        => self.exec_block_stmt( decls, line ),
      Stmt::If( init, condition , then, else_)
        => self.exec_if_stmt( init, condition, then, else_ ),
      Stmt::While( init, condition , loop_ )
        => self.exec_while_stmt( init, condition, loop_ ),
      Stmt::For( init, condition , incr, body )
        => self.exec_for_stmt( init, condition, incr, body )
    }
  }

  fn exec_print_stmt( &mut self, expr: &Expr ) -> EvalResult {
    let result = expr.eval( self.sm, &self.env )?;
    print!( "{}\n", result );
    Ok( result )
  }

  fn exec_expr_stmt( &mut self, expr: &Expr ) -> EvalResult {
    let result = expr.eval( self.sm, &self.env )?;
    if let Expr::Assignment( var, rhs ) = expr {
      self.exec_assign_expr( var, rhs, result )
    } else{
      Ok( result )
    } 
  }

  fn exec_assign_expr( &mut self, var: &Token, rhs: &Expr, result: Eval ) -> EvalResult {

    // rhs might be a nested assignment expression
    if let Expr::Assignment( nested_var, nested_rhs ) = rhs {
        self.exec_assign_expr( nested_var, nested_rhs, result.clone() )?;
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

  fn exec_block_stmt( &mut self, decls: &Vec<Decl>, line: &i32 ) -> EvalResult {
    let mut result = Eval::Nil;
    self.enclose_new_scope( *line );
    for decl in decls {
      result = self.exec_decl( decl )?;
    }
    self.drop_enclosed_scope();
    Ok( result )
  }

  fn exec_if_stmt(
    &mut self,
    init: &Option<CtrlFlowInit>,
    condition: &Expr,
    then: &Box<Stmt>,
    else_: &Option<Box<Stmt>> ) -> EvalResult {

    // initialiser if supplied
    let mut has_scope = false;
    if let Some( flow_init ) = init.as_ref() {
      match flow_init {

        // variable declaration
        CtrlFlowInit::VarDecl( var_decl ) => {
          let decl = var_decl.as_ref();
          if let Decl::Var( var, _ ) = decl {
            has_scope = true;
            self.enclose_new_scope( var.get_line() );
            self.exec_decl( decl )?;
          } else {
            panic!( "Internal error. The declaration in this if-statement initialiser is not a variable declaration." );
          }
        },

        // expression statement
        CtrlFlowInit::ExprStmt( expr_stmt ) => {
          let stmt = expr_stmt.as_ref();
          if let Stmt::Expr( expr ) = stmt {
            self.exec_expr_stmt( expr )?;
          } else {
            panic!( "Internal error. The statement in this if-statement initialiser is not an expression statement." );
          }
        }
      }
    }

    // run if-then-else
    let result = if condition.eval( self.sm, &self.env )?.is_truthy() {
      self.exec_stmt( then )?
    } else if else_.is_some() {
      self.exec_stmt( else_.as_ref().unwrap() )?
    }
    else {
      Eval::Bool( false )
    };

    // tidy-up from declaration if required
    if has_scope {
      self.drop_enclosed_scope();
    }

    // success
    Ok( result )

  }

  fn exec_while_stmt(
    &mut self,
    init: &Option<CtrlFlowInit>,
    condition: &Expr,
    body: &Box<Stmt> ) -> EvalResult {
    
    // init
    let mut result = Eval::Nil;
    
    // initialiser if supplied
    let mut has_scope = false;
    if let Some( flow_init ) = init.as_ref() {
      match flow_init {

        // variable declaration
        CtrlFlowInit::VarDecl( var_decl ) => {
          let decl = var_decl.as_ref();
          if let Decl::Var( var, _ ) = decl {
            has_scope = true;
            self.enclose_new_scope( var.get_line() );
            result = self.exec_decl( decl )?;
          } else {
            panic!( "Internal error. The declaration in this while-statement initialiser is not a variable declaration." );
          }
        },

        // expression statement
        CtrlFlowInit::ExprStmt( expr_stmt ) => {
          let stmt = expr_stmt.as_ref();
          if let Stmt::Expr( expr ) = stmt {
            result = self.exec_expr_stmt( expr )?;
          } else {
            panic!( "Internal error. The statement in this while-statement initialiser is not an expression statement." );
          }
        }
      }
    }

    // run loop
    while condition.eval( self.sm, &self.env )?.is_truthy() {
      result = self.exec_stmt( body )?;
    }

    // tidy-up from declaration if required
    if has_scope {
      self.drop_enclosed_scope();
    }

    Ok( result )
  }

  fn exec_for_stmt( &mut self, init: &Option<CtrlFlowInit>, condition: &Option<Expr>, incr: &Option<Expr>, body: &Box<Stmt> ) -> EvalResult {

    // init
    let mut result = Eval::Nil;
    
    // initialiser if supplied
    let mut has_scope = false;
    if let Some( flow_init ) = init.as_ref() {
      match flow_init {

        // variable declaration
        CtrlFlowInit::VarDecl( var_decl ) => {
          let decl = var_decl.as_ref();
          if let Decl::Var( var, _ ) = decl {
            has_scope = true;
            self.enclose_new_scope( var.get_line() );
            result = self.exec_decl( decl )?;
          } else {
            panic!( "Internal error. The declaration in this for-statement initialiser is not a variable declaration." );
          }
        },

        // expression statement
        CtrlFlowInit::ExprStmt( expr_stmt ) => {
          let stmt = expr_stmt.as_ref();
          if let Stmt::Expr( expr ) = stmt {
            result = self.exec_expr_stmt( expr )?;
          } else {
            panic!( "Internal error. The statement in this for-statement initialiser is not an expression statement." );
          }
        }
      }
    }

    // run loop
    loop {

      // condition
      if let Some( expr ) = condition {
        if !expr.eval( self.sm, &self.env )?.is_truthy() {
          break;
        }
      }

      // body
      result = self.exec_stmt( body.as_ref() )?;

      // incr
      if let Some( expr ) = incr {
        result = self.exec_expr_stmt( expr )?;
      }
    }

    // tidy-up from init if required
    if has_scope {
      self.drop_enclosed_scope();
    }

    Ok( result )
  }


  fn enclose_new_scope( &mut self, line: i32 ) {
    self.env = Env::enclose_new( &self.env, line );
  }

  fn drop_enclosed_scope( &mut self ) {
    self.env = Env::drop_enclosed( &self.env );
  }

  fn make_error( &self, t: &Token, msg: String ) -> Error {
    Error::from_token( t, msg, self.sm )
  }

  fn emit_error( &mut self, error: &Error ) {
    eprintln!( "[line {}] Runtime error{}: {}", error.line, error.loc, error.msg );
    self.had_error = true;
  }

}