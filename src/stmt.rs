
use crate::{expr::Expr, token::Token};

#[derive(Clone)]
pub enum Stmt {
  Block( Block ),
  Expression( Expression ),
  Function( Function ),
  If( If ),
  Print( Print ),
  Return( Return ),
  Var( Var ),
  While( While )
}

pub trait Visitor<R> {
  fn visit_block_stmt( &mut self, block: &Block ) -> R;
  fn visit_expression_stmt( &mut self, expression: &Expression ) -> R;
  fn visit_function_stmt( &mut self, function: &Function ) -> R;
  fn visit_if_stmt( &mut self, if_: &If ) -> R;
  fn visit_print_stmt( &mut self, print: &Print ) -> R;
  fn visit_return_stmt( &mut self, return_: &Return ) -> R;
  fn visit_var_stmt( &mut self, var: &Var ) -> R;
  fn visit_while_stmt( &mut self, while_: &While ) -> R;
}

pub trait MutVisitor<R> {
  fn visit_block_stmt_mut( &mut self, block: &mut Block ) -> R;
  fn visit_expression_stmt_mut( &mut self, expression: &mut Expression ) -> R;
  fn visit_function_stmt_mut( &mut self, function: &mut Function ) -> R;
  fn visit_if_stmt_mut( &mut self, if_: &mut If ) -> R;
  fn visit_print_stmt_mut( &mut self, print: &mut Print ) -> R;
  fn visit_return_stmt_mut( &mut self, return_: &mut Return ) -> R;
  fn visit_var_stmt_mut( &mut self, var: &mut Var ) -> R;
  fn visit_while_stmt_mut( &mut self, while_: &mut While ) -> R;
}

#[derive(Clone)]
pub struct Block {
  pub statements: Vec<Stmt>,
  pub line: u32
}

#[derive(Clone)]
pub struct Expression {
  pub expression: Expr
}

#[derive(Clone)]
pub struct Function {
  pub name: Token,
  pub params: Vec<Token>,
  pub body: Vec<Stmt>
}

#[derive(Clone)]
pub struct If {
  pub condition: Expr,
  pub then_branch: Box<Stmt>,
  pub else_branch: Option<Box<Stmt>>
}

#[derive(Clone)]
pub struct Print {
  pub expression: Expr
}

#[derive(Clone)]
pub struct Return {
  pub keyword: Token,
  pub value: Option<Expr>
}

#[derive(Clone)]
pub struct Var {
  pub name: Token,
  pub init: Option<Expr>
}

#[derive(Clone)]
pub struct While {
  pub condition: Expr,
  pub body: Box<Stmt>
}

impl Stmt {
  pub fn accept<R, V: Visitor<R>>( &self, visitor: &mut V ) -> R {
    match self {
      Stmt::Block( block ) => visitor.visit_block_stmt( block ),
      Stmt::Expression( expression ) => visitor.visit_expression_stmt( expression ),
      Stmt::Function( function ) => visitor.visit_function_stmt( function ),
      Stmt::If( if_ ) => visitor.visit_if_stmt( if_ ),
      Stmt::Print( print ) => visitor.visit_print_stmt( print ),
      Stmt::Return( return_ ) => visitor.visit_return_stmt( return_ ),
      Stmt::Var( var ) => visitor.visit_var_stmt( var ),
      Stmt::While( while_ ) => visitor.visit_while_stmt( while_ ),
    }
  }
  pub fn accept_mut<R, V: MutVisitor<R>>( &mut self, visitor: &mut V ) -> R {
    match self {
      Stmt::Block( block ) => visitor.visit_block_stmt_mut( block ),
      Stmt::Expression( expression ) => visitor.visit_expression_stmt_mut( expression ),
      Stmt::Function( function ) => visitor.visit_function_stmt_mut( function ),
      Stmt::If( if_ ) => visitor.visit_if_stmt_mut( if_ ),
      Stmt::Print( print ) => visitor.visit_print_stmt_mut( print ),
      Stmt::Return( return_ ) => visitor.visit_return_stmt_mut( return_ ),
      Stmt::Var( var ) => visitor.visit_var_stmt_mut( var ),
      Stmt::While( while_ ) => visitor.visit_while_stmt_mut( while_ ),
    }
  }
}