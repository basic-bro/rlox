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
  sc: &'str mut StringCache,
  envs: EnvStack,
  had_error: bool
}

impl<'str> Executor<'str> {

  pub fn new( sc: &'str mut StringCache ) -> Executor<'str> {
    let mut executor = Executor{
      sc,
      envs: EnvStack::new(),
      had_error: false,
    };

    executor.envs.enclose_new( 0 );
    executor
  }

  pub fn with_envs( sc: &'str mut StringCache, envs: EnvStack ) -> Executor<'str> {
    Executor {
      sc,
      envs,
      had_error: false
    }
  }

  pub fn exec( &mut self, decls: Vec<Decl> ) -> ( Eval, bool ) {
    self.restart();
    let mut retval = Eval::Nil;
    for decl in decls {
      match self.exec_decl( &decl ) {
        Ok( val ) => retval = val,
        Err( EvalError::Error( error) ) => self.emit_error( &error ),
        Err( EvalError::Return( _ ) ) => panic!( "EvalError::Abort should not make its way to here." )
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
    // println!( "Executing declaration: {:?}", decl );
    match decl {
      Decl::Stmt( stmt ) => self.exec_stmt( stmt ),
      Decl::Var( var, init ) => self.exec_var_decl( var, init ),
      Decl::Fun( fun_name, params, body) => self.exec_fun_decl( fun_name, params, body )
    }
  }

  fn exec_stmt( &mut self, stmt: &Stmt ) -> EvalResult {
    match stmt {
      Stmt::Print( expr )
        => self.exec_print_stmt( expr ),
      Stmt::Expr( expr )
        => self.exec_expr_stmt( expr ),
      Stmt::Block( decls, line )
        => self.exec_block_stmt( decls, line, true ),
      Stmt::If( init, condition , then, else_)
        => self.exec_if_stmt( init, condition, then, else_ ),
      Stmt::While( init, condition , loop_ )
        => self.exec_while_stmt( init, condition, loop_ ),
      Stmt::For( init, condition , incr, body )
        => self.exec_for_stmt( init, condition, incr, body ),
      Stmt::Return( expr )
        => self.exec_return_stmt( expr )
    }
  }

  fn exec_var_decl( &mut self, var: &Token, init: &Option<Expr> ) -> EvalResult {
    let key = var.get_key();

    // error on redefinition
    if self.envs.has_symbol_here( key ) {
      return Err( self.make_error( var, "This symbol is already in use.".to_string() ) );
    }

    // evaluate initialiser
    let result = match init {
      Some( expr ) => expr.eval( self.sc, &self.envs ),
      None => Ok( Eval::Nil )
    }?;

    // create variable
    self.envs.create_symbol( key, result.clone() );
    Ok( result )
  }

  fn exec_fun_decl( &mut self, fun_name: &Token, params: &Vec<Token>, body: &Stmt ) -> EvalResult {
    let key = fun_name.get_key();

    // error on redefinition
    if self.envs.has_symbol_here( key ) {
      return Err( self.make_error( fun_name, "This name is already in use.".to_string() ) );
    }

    // parameter keys
    let mut param_keys: Vec<StringKey> = vec![];
    for t in params {
      param_keys.push( t.get_key() );
    }

    // result
    let result = Eval::Fun( param_keys, body.clone() );

    // create function entry
    self.envs.create_symbol( key, result.clone() );
    Ok( result )
  }

  fn exec_print_stmt( &mut self, expr: &Expr ) -> EvalResult {
    let result = expr.eval( self.sc, &self.envs )?;
    print!( "{}\n", result );
    Ok( result )
  }

  fn exec_expr_stmt( &mut self, expr: &Expr ) -> EvalResult {
    let result = expr.eval( self.sc, &self.envs )?;
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
    if self.envs.has_symbol( key ) {
      self.envs.write_symbol( key, result.clone() );
      Ok( result )
    }
    else {
      Err( self.make_error( var, "Undeclared variable.".to_string() ) )
    }
  }

  pub fn exec_block_stmt( &mut self, decls: &Vec<Decl>, line: &i32, new_scope: bool ) -> EvalResult {
    let mut result = Eval::Nil;
    if new_scope {
      self.enclose_new_scope( *line );
    }
    for decl in decls {
      result = self.exec_decl( decl )?;
    }
    if new_scope {
      self.drop_enclosed_scope();
    }
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
    let result = if condition.eval( self.sc, &self.envs )?.is_truthy() {
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
    while condition.eval( self.sc, &self.envs )?.is_truthy() {
      result = self.exec_stmt( body )?;
    }

    // tidy-up from declaration if required
    if has_scope {
      self.drop_enclosed_scope();
    }

    Ok( result )
  }

  fn exec_for_stmt(
    &mut self,
    init: &Option<CtrlFlowInit>,
    condition: &Option<Expr>,
    incr: &Option<Expr>,
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
        if !expr.eval( self.sc, &self.envs )?.is_truthy() {
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

  fn exec_return_stmt( &mut self, expr: &Option<Expr> ) -> EvalResult {
    if expr.is_some() {
      Err( EvalError::Return( expr.as_ref().unwrap().eval( self.sc, &self.envs )? ) )
    } else {
      Err( EvalError::Return( Eval::Nil ) )
    }
  }

  fn enclose_new_scope( &mut self, line: i32 ) {
    self.envs.enclose_new( line );
  }

  fn drop_enclosed_scope( &mut self ) {
    self.envs.drop_enclosed();
  }

  fn make_error( &self, t: &Token, msg: String ) -> EvalError {
    EvalError::Error( Error::from_token( t, msg, self.sc ) )
  }

  fn emit_error( &mut self, error: &Error ) {
    eprintln!( "[line {}] Runtime error{}: {}", error.line, error.loc, error.msg );
    self.had_error = true;
  }

}