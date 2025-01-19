

/////////
// use //
/////////

use std::collections::HashMap;

use crate::{expr::{self, Expr, Variable}, interpreter::Interpreter, stmt::{self, Stmt}, token::Token, util::Stack};
use crate::error::Error;

/////////////////
// declaration //
/////////////////

pub struct Resolver {
  interpreter: Interpreter,
  scopes: Stack<HashMap<String, NameStatus>>
}

#[derive(PartialEq)]
enum NameStatus {
  Declared,
  Defined
}


/////////////////////
// implementations //
/////////////////////

impl Resolver {
  pub fn new( interpreter: Interpreter ) -> Resolver {
    Resolver {
      interpreter,
      scopes: Stack::new()
    }
  }
  fn begin_scope( &mut self ) {
    self.scopes.push( HashMap::new() );
  }
  fn end_scope( &mut self ) {
    self.scopes.pop();
  }
  fn declare_name( &mut self, name: &Token ) -> Result<(), Error> {
    if self.scopes.is_empty() {
      return Ok( () )
    }
    let scope = self.scopes.peek_mut( 0 );
    if scope.contains_key( &name.lexeme ) {
      return Err( Error::from_token( &name, "Name already in use.".into() ) );
    }
    scope.insert( name.lexeme.clone(), NameStatus::Declared );
    Ok( () )
  }
  fn define_name( &mut self, name: &Token ) {
    if self.scopes.is_empty() {
      return;
    }
    self.scopes.peek_mut( 0 ).insert( name.lexeme.clone(), NameStatus::Defined );
  }
  fn declare_define( &mut self, name: &Token ) -> Result<(), Error> {
    self.declare_name( name )?;
    self.define_name( name );
    Ok( () )
  }
  fn resolve_local( &mut self, expr: Expr, name: &Token ) {
    for depth in 0..self.scopes.depth() {
      if self.scopes.peek( depth ).contains_key( &name.lexeme ) {
        self.interpreter.add_local( expr, depth );
        return;
      }
    }
  }
  fn resolve_expr( &mut self, expr: &Expr ) -> Result<(), Error> {
    expr.accept( self )
  }
  fn resolve_stmt( &mut self, stmt: &Stmt ) -> Result<(), Error> {
    stmt.accept( self )
  }
  fn resolve_stmts( &mut self, stmts: &Vec<Stmt> ) -> Result<(), Error> {
    for stmt in stmts {
      self.resolve_stmt( stmt )?;
    }
    Ok( () )
  }
}

impl expr::Visitor<Result<(), Error>> for Resolver {
  fn visit_assign_expr( &mut self, assign: &expr::Assign ) -> Result<(), Error> {
    self.resolve_expr( &assign.value )?;
    self.resolve_local( Expr::Assign( assign.clone() ), &assign.name );
    Ok( () )
  }
  fn visit_binary_expr( &mut self, binary: &expr::Binary ) -> Result<(), Error> {
    self.resolve_expr( &binary.left )?;
    self.resolve_expr( &binary.right )
  }
  fn visit_call_expr( &mut self, call: &expr::Call ) -> Result<(), Error> {
    self.resolve_expr( &call.callee )?;
    for argument in &call.arguments {
      self.resolve_expr( &argument )?;
    }
    Ok( () )
  }
  fn visit_grouping_expr( &mut self, grouping: &expr::Grouping ) -> Result<(), Error> {
    self.resolve_expr( &grouping.expression )
  }
  fn visit_literal_expr( &mut self, _literal: &expr::Literal ) -> Result<(), Error> {
    Ok( () )
  }
  fn visit_logical_expr( &mut self, logical: &expr::Logical ) -> Result<(), Error> {
    self.resolve_expr( &logical.left )?;
    self.resolve_expr( &logical.right )
  }
  fn visit_unary_expr( &mut self, unary: &expr::Unary ) -> Result<(), Error> {
    self.resolve_expr( &unary.right )
  }
  fn visit_variable_expr( &mut self, variable: &expr::Variable ) -> Result<(), Error> {
    if !self.scopes.is_empty() {
      if let Some( status ) = self.scopes.peek( 0 ).get( &variable.name.lexeme ) {
        if *status != NameStatus::Defined {
          return Err( Error::from_token( &variable.name,
            "Can't read local variable in its own initialiser.".into() ) );
        }
      }
    }
    self.resolve_local( Expr::Variable( Variable { name: variable.name.clone() } ), &variable.name );
    Ok( () )
  }
}

impl stmt::Visitor<Result<(), Error>> for Resolver {
  fn visit_block_stmt( &mut self, block: &stmt::Block ) -> Result<(), Error> {
    self.begin_scope();
    self.resolve_stmts( &block.statements )?;
    self.end_scope();
    Ok( () )
  }
  fn visit_expression_stmt( &mut self, expression: &stmt::Expression ) -> Result<(), Error> {
    self.resolve_expr( &expression.expression )
  }
  fn visit_function_stmt( &mut self, function: &stmt::Function ) -> Result<(), Error> {
    self.declare_define( &function.name )?;
    self.begin_scope();
    for param in &function.params {
      self.declare_define( param )?;
    }
    self.resolve_stmts( &function.body )?;
    self.end_scope();
    Ok( () )
  }
  fn visit_if_stmt( &mut self, if_: &stmt::If ) -> Result<(), Error> {
    self.resolve_expr( &if_.condition )?;
    self.resolve_stmt( &if_.then_branch )?;
    if let Some( stmt ) = &if_.else_branch {
      self.resolve_stmt( stmt )?;
    }
    Ok( () )
  }
  fn visit_print_stmt( &mut self, print: &stmt::Print ) -> Result<(), Error> {
    self.resolve_expr( &print.expression )
  }
  fn visit_return_stmt( &mut self, return_: &stmt::Return ) -> Result<(), Error> {
    if let Some( expr ) = &return_.value {
      self.resolve_expr( expr )?;
    }
    Ok( () )
  }
  fn visit_var_stmt( &mut self, var: &stmt::Var ) -> Result<(), Error> {
    self.declare_name( &var.name )?;
    if let Some( expr ) = &var.init {
      self.resolve_expr( expr )?;
    }
    self.define_name( &var.name );
    Ok( () )
  }
  fn visit_while_stmt( &mut self, while_: &stmt::While ) -> Result<(), Error> {
    self.resolve_expr( &while_.condition )?;
    self.resolve_stmt( &while_.body )
  }
}