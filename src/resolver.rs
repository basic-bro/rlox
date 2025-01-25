

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
  global_init_order: Vec<String>,
  had_error: bool
}

struct ResolveStatus {
  name_status: NameStatus,
  is_read: bool,
  decl_line: u32
}

#[derive(PartialEq)]
enum NameStatus {
  Declared,
  Defined
}


/////////////////////
// implementations //
/////////////////////

impl ResolveStatus {
  pub fn declare( line: u32 ) -> ResolveStatus {
    ResolveStatus {
      name_status: NameStatus::Declared,
      is_read: false,
      decl_line: line
    }
  }
  pub fn define( &mut self ) {
    self.name_status = NameStatus::Defined;
  }
  pub fn mark_as_read( &mut self ) {
    self.is_read = true;
  }
  pub fn is_unread( &self ) -> bool {
    !self.is_read
  }
  pub fn get_line( &self ) -> u32 {
    self.decl_line
  }
}

impl Resolver {
  pub fn new() -> Resolver {
    Resolver {
      scopes: Stack::new(),
      global_init_order: Vec::new(),
      had_error: false
    }
  }
  fn restart( &mut self ) {
    self.scopes.clear();
    self.global_init_order.clear();
    self.had_error = false;
  }
  fn begin_scope( &mut self ) {
    self.scopes.push( HashMap::new() );
  }
  fn end_scope( &mut self ) {
    self.warn_unused();
    self.scopes.pop();
  }
  fn declare_name( &mut self, name: &Token ) -> Result<(), Error> {
    let scope = self.scopes.peek_mut( 0 );
    if scope.contains_key( &name.lexeme ) {
      return Err( Error::from_token( &name, "Name already in use.".into() ) );
    }
    scope.insert( name.lexeme.clone(), ResolveStatus::declare( name.line ) );
    Ok( () )
  }
  fn define_name( &mut self, name: &Token ) {
    self.scopes.peek_mut( 0 ).get_mut( &name.lexeme ).unwrap().define();
  }
  fn declare_define( &mut self, name: &Token ) -> Result<(), Error> {
    self.declare_name( name )?;
    self.define_name( name );
    Ok( () )
  }
  fn resolve_variable( &mut self, variable: &mut Variable ) -> Result<(), Error> {
    for depth in 0..self.scopes.depth() {
      if self.scopes.peek( depth ).contains_key( &variable.name.lexeme ) {
        variable.jump = depth as i32;
        // println!( "Resolved: '{}' on line {} is defined {} scope(s) back.",
        //   variable.name.lexeme, variable.name.line, variable.jump );
        self.scopes.peek_mut( depth ).get_mut( &variable.name.lexeme ).unwrap().mark_as_read();
        return Ok( () );
      }
    }
    Err( Error::from_token( &variable.name, "Undeclared symbol.".into() ) )
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
  fn warn_unused( &self ) {
    for ( name, status ) in self.scopes.peek( 0 ) {
      if status.is_unread() {
        eprintln!( "[line {}] Warning at '{}': Symbol is defined but never used.", status.get_line(), name );
      }
    }
  }
  pub fn resolve( &mut self, stmts: &mut Vec<Stmt> ) -> bool {
    self.restart();
    self.begin_scope();
    match self.resolve_stmts( stmts ) {
      Ok( _ ) => {},
      Err( e ) => self.emit_error( &e ),
    }
    self.end_scope();
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
    self.resolve_variable( &mut assign.lhs )
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
  // fn visit_logical_expr_mut( &mut self, logical: &mut expr::Logical ) -> Result<(), Error> {
  //   self.resolve_expr( &mut logical.left )?;
  //   self.resolve_expr( &mut logical.right )
  // }
  fn visit_unary_expr_mut( &mut self, unary: &mut expr::Unary ) -> Result<(), Error> {
    self.resolve_expr( &mut unary.right )
  }
  fn visit_variable_expr_mut( &mut self, variable: &mut expr::Variable ) -> Result<(), Error> {
    if let Some( status ) = self.scopes.peek( 0 ).get( &variable.name.lexeme ) {
      if status.name_status != NameStatus::Defined {
        return Err( Error::from_token( &variable.name,
          "Cannot read a local variable in its own initialiser.".into() ) );
      }
    }
    self.resolve_variable( variable )
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