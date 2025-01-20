

/////////
// use //
/////////

use std::collections::HashMap;

use crate::{expr::{self, Expr, Variable}, stmt::{self, Stmt}, token::Token, util::Stack};
use crate::error::Error;

/////////////////
// declaration //
/////////////////

pub struct Resolver {
  scopes: Stack<HashMap<String, ResolveStatus>>,
  had_error: bool
}

struct ResolveStatus {
  pub name_status: NameStatus,
  pub is_read: bool
}

#[derive(PartialEq)]
enum NameStatus {
  Declared( /* line: */ u32 ),
  Defined
}


/////////////////////
// implementations //
/////////////////////

impl ResolveStatus {
  pub fn new( name_status: NameStatus ) -> ResolveStatus {
    ResolveStatus {
      name_status,
      is_read: false
    }
  }
}

impl Resolver {
  pub fn new() -> Resolver {
    Resolver {
      scopes: Stack::new(),
      had_error: false
    }
  }
  fn restart( &mut self ) {
    self.scopes = Stack::new();
    self.had_error = false;
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
    scope.insert( name.lexeme.clone(), ResolveStatus::new( NameStatus::Declared( name.line ) ) );
    Ok( () )
  }
  fn define_name( &mut self, name: &Token ) {
    if self.scopes.is_empty() {
      return;
    }
    self.scopes.peek_mut( 0 ).insert( name.lexeme.clone(), ResolveStatus::new( NameStatus::Defined ) );
  }
  fn declare_define( &mut self, name: &Token ) -> Result<(), Error> {
    self.declare_name( name )?;
    self.define_name( name );
    Ok( () )
  }
  fn resolve_variable( &mut self, variable: &mut Variable ) {
    for depth in 0..self.scopes.depth() {
      if self.scopes.peek( depth ).contains_key( &variable.name.lexeme ) {
        variable.jump = depth as i32;
        // println!( "Resolved: '{}' on line {} is defined {} scope(s) back.",
        //   variable.name.lexeme, variable.name.line, variable.jump );
          self.scopes.peek_mut( depth ).get_mut( &variable.name.lexeme ).unwrap().is_read = true;
        return;
      }
    }
    // println!( "Unresolved, assuming global: '{}' on line {}.",
    //   variable.name.lexeme, variable.name.line );
  }
  fn resolve_expr( &mut self, expr: &mut Expr ) -> Result<(), Error> {
    expr.accept_mut( self )
  }
  fn resolve_stmt( &mut self, stmt: &mut Stmt ) -> Result<(), Error> {
    stmt.accept_mut( self )
  }
  fn resolve_stmts( &mut self, stmts: &mut Vec<Stmt> ) -> Result<(), Error> {
    for stmt in stmts {
      self.resolve_stmt( stmt )?;
    }
    Ok( () )
  }
  // fn warn_unused( &self ) {
  // }
  pub fn resolve( &mut self, stmts: &mut Vec<Stmt> ) -> bool {
    self.restart();
    match self.resolve_stmts( stmts ) {
      Ok( _ ) => {},
      Err( e ) => self.emit_error( &e ),
    }
    // self.warn_unused();
    self.had_error
  }
  fn emit_error( &mut self, error: &Error ) {
    eprintln!( "[line {}] Error{}: {}", error.line, error.loc, error.msg );
    self.had_error = true;
  }
}

impl expr::MutVisitor<Result<(), Error>> for Resolver {
  fn visit_assign_expr_mut( &mut self, assign: &mut expr::Assign ) -> Result<(), Error> {
    self.resolve_expr( &mut assign.rhs )?;
    self.resolve_variable( &mut assign.lhs );
    Ok( () )
  }
  fn visit_binary_expr_mut( &mut self, binary: &mut expr::Binary ) -> Result<(), Error> {
    self.resolve_expr( &mut binary.left )?;
    self.resolve_expr( &mut binary.right )
  }
  fn visit_call_expr_mut( &mut self, call: &mut expr::Call ) -> Result<(), Error> {
    self.resolve_expr( &mut call.callee )?;
    for argument in &mut call.arguments {
      self.resolve_expr( argument.as_mut() )?;
    }
    Ok( () )
  }
  fn visit_grouping_expr_mut( &mut self, grouping: &mut expr::Grouping ) -> Result<(), Error> {
    self.resolve_expr( &mut grouping.expression )
  }
  fn visit_literal_expr_mut( &mut self, _literal: &mut expr::Literal ) -> Result<(), Error> {
    Ok( () )
  }
  fn visit_logical_expr_mut( &mut self, logical: &mut expr::Logical ) -> Result<(), Error> {
    self.resolve_expr( &mut logical.left )?;
    self.resolve_expr( &mut logical.right )
  }
  fn visit_unary_expr_mut( &mut self, unary: &mut expr::Unary ) -> Result<(), Error> {
    self.resolve_expr( &mut unary.right )
  }
  fn visit_variable_expr_mut( &mut self, variable: &mut expr::Variable ) -> Result<(), Error> {
    if !self.scopes.is_empty() {
      if let Some( status ) = self.scopes.peek( 0 ).get( &variable.name.lexeme ) {
        if status.name_status != NameStatus::Defined {
          return Err( Error::from_token( &variable.name,
            "Cannot read a local variable in its own initialiser.".into() ) );
        }
      }
    }
    self.resolve_variable( variable );
    Ok( () )
  }
}

impl stmt::MutVisitor<Result<(), Error>> for Resolver {
  fn visit_block_stmt_mut( &mut self, block: &mut stmt::Block ) -> Result<(), Error> {
    self.begin_scope();
    self.resolve_stmts( &mut block.statements )?;
    self.end_scope();
    Ok( () )
  }
  fn visit_expression_stmt_mut( &mut self, expression: &mut stmt::Expression ) -> Result<(), Error> {
    self.resolve_expr( &mut expression.expression )
  }
  fn visit_function_stmt_mut( &mut self, function: &mut stmt::Function ) -> Result<(), Error> {
    self.declare_define( &function.name )?;
    self.begin_scope();
    for param in &function.params {
      self.declare_define( param )?;
    }
    self.resolve_stmts( &mut function.body )?;
    self.end_scope();
    Ok( () )
  }
  fn visit_if_stmt_mut( &mut self, if_: &mut stmt::If ) -> Result<(), Error> {
    self.resolve_expr( &mut if_.condition )?;
    self.resolve_stmt( &mut if_.then_branch )?;
    if let Some( stmt ) = &mut if_.else_branch {
      self.resolve_stmt( stmt )?;
    }
    Ok( () )
  }
  fn visit_print_stmt_mut( &mut self, print: &mut stmt::Print ) -> Result<(), Error> {
    self.resolve_expr( &mut print.expression )
  }
  fn visit_return_stmt_mut( &mut self, return_: &mut stmt::Return ) -> Result<(), Error> {
    if let Some( expr ) = &mut return_.value {
      self.resolve_expr( expr )?;
    }
    Ok( () )
  }
  fn visit_var_stmt_mut( &mut self, var: &mut stmt::Var ) -> Result<(), Error> {
    self.declare_name( &var.name )?;
    if let Some( expr ) = &mut var.init {
      self.resolve_expr( expr )?;
    }
    self.define_name( &var.name );
    Ok( () )
  }
  fn visit_while_stmt_mut( &mut self, while_: &mut stmt::While ) -> Result<(), Error> {
    self.resolve_expr( &mut while_.condition )?;
    self.resolve_stmt( &mut while_.body )
  }
}