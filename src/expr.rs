
use crate::token::Token;

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum Expr {
  Assign( Assign ),
  Binary( Binary ),
  Call( Call ),
  Grouping( Grouping ),
  Literal( Literal ),
  Logical( Logical ),
  Unary( Unary ),
  Variable( Variable )
}

pub trait Visitor<R> {
  fn visit_assign_expr( &mut self, assign: &Assign ) -> R;
  fn visit_binary_expr( &mut self, binary: &Binary ) -> R;
  fn visit_call_expr( &mut self, call: &Call ) -> R;
  fn visit_grouping_expr( &mut self, grouping: &Grouping ) -> R;
  fn visit_literal_expr( &mut self, literal: &Literal ) -> R;
  fn visit_logical_expr( &mut self, logical: &Logical ) -> R;
  fn visit_unary_expr( &mut self, unary: &Unary ) -> R;
  fn visit_variable_expr( &mut self, variable: &Variable ) -> R;
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Assign {
  // pub uuid: usize,
  pub name: Token,
  pub value: Box<Expr>
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Binary {
  // pub uuid: usize,
  pub left: Box<Expr>,
  pub operator: Token,
  pub right: Box<Expr>
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Call {
  // pub uuid: usize,
  pub callee: Box<Expr>,
  pub paren: Token,
  pub arguments: Vec<Box<Expr>>
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Grouping {
  // pub uuid: usize,
  pub expression: Box<Expr>
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Literal {
  // pub uuid: usize,
  pub value: Token
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Logical {
  // pub uuid: usize,
  pub left: Box<Expr>,
  pub operator: Token,
  pub right: Box<Expr>
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Unary {
  // pub uuid: usize,
  pub operator: Token,
  pub right: Box<Expr>
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Variable {
  // pub uuid: usize,
  pub name: Token
}

impl Expr {
  pub fn accept<R, V: Visitor<R>>( &self, visitor: &mut V ) -> R {
    match self {
      Expr::Assign( assign ) => visitor.visit_assign_expr( assign ),
      Expr::Binary( binary ) => visitor.visit_binary_expr( binary ),
      Expr::Call( call ) => visitor.visit_call_expr( call ),
      Expr::Grouping( grouping ) => visitor.visit_grouping_expr( grouping ),
      Expr::Literal( literal ) => visitor.visit_literal_expr( literal ),
      Expr::Logical( logical ) => visitor.visit_logical_expr( logical ),
      Expr::Unary( unary ) => visitor.visit_unary_expr( unary ),
      Expr::Variable( variable ) => visitor.visit_variable_expr( variable ),
    }
  }
}