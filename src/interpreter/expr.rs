////////////////////////////////////////////
// private module rlox::interpreter::expr //
////////////////////////////////////////////


/////////
// use //
/////////

use crate::interpreter::error::*;
use crate::interpreter::token::*;
use crate::util::*;
use crate::interpreter::eval::*;
use crate::interpreter::env::*;


//////////////////////
// public interface //
//////////////////////

#[derive(Debug, Clone)]
pub enum Expr {
  Assignment( Token /* identifier */, Box<Expr> /* value */ ),
  Binary( Box<Expr> /* left */, Token /* operator */, Box<Expr> /* right */ ),
  Call( /* callee: */ Box<Expr>, /* paren: */ Token, /* args: */ Vec<Box<Expr>> ),
  Grouping( Box<Expr> ),
  Literal( Token /* identifier? | string | number | true | false | nil | eof */ ),
  Unary( Token /* operator */, Box<Expr> ),
  Var( Token /* variable */ )
}

pub trait ExprVisitor<T> {
  fn visit_assignment( &self, var: &Token, right: T ) -> Result<T, Error>;
  fn visit_binary( &self, left: T, op: &Token, right: T ) -> Result<T, Error>;
  fn visit_call( &self, callee: T, paren: &Token, args: &Vec<T> ) -> Result<T, Error>;
  fn visit_grouping( &self, expr: T ) -> Result<T, Error>;
  fn visit_literal( &self, literal: &Token ) -> Result<T, Error>;
  fn visit_unary( &self, op: &Token, expr: T ) -> Result<T, Error>;
  fn visit_var( &self, var: &Token ) -> Result<T, Error>;
}

pub trait ExprVisitorMut<T, E> {
  fn visit_assignment( &self, var: &Token, right: T ) -> Result<T, E>;
  fn visit_binary( &self, left: T, op: &Token, right: T ) -> Result<T, E>;
  fn visit_call( &mut self, callee: T, paren: &Token, args: &Vec<T> ) -> Result<T, E>;
  fn visit_grouping( &self, expr: T ) -> Result<T, E>;
  fn visit_literal( &self, literal: &Token ) -> Result<T, E>;
  fn visit_unary( &self, op: &Token, expr: T ) -> Result<T, E>;
  fn visit_var( &self, var: &Token ) -> Result<T, E>;
}

impl Expr {

  pub fn visit<T, V: ExprVisitor<T>>( &self, visitor: &V ) -> Result<T, Error>
  {
    match self {
      Self::Assignment( var, right )
        => visitor.visit_assignment( var, right.visit( visitor )? ),
      Self::Binary( left, op , right )
        => visitor.visit_binary( left.visit( visitor )?, op, right.visit( visitor )? ),
      Self::Call( callee, paren , args )
        => visitor.visit_call( callee.visit( visitor )?, paren, &self.visit_args( visitor, args )? ),
      Self::Grouping( inner )
        => visitor.visit_grouping( inner.visit( visitor )? ),
      Self::Literal( literal )
        => visitor.visit_literal( literal ),
      Self::Unary( op, expr )
        => visitor.visit_unary( op, expr.visit( visitor )? ),
      Self::Var( op )
        => visitor.visit_var( op )
    }
  }

  fn visit_args<T, V: ExprVisitor<T>>( &self, visitor: &V, args: &Vec<Box<Expr>> ) -> Result<Vec<T>, Error> {
    let mut results: Vec<T> = vec![];
    for arg in args {
      results.push( arg.visit( visitor )? );
    }
    Ok( results )
  }

  pub fn to_string( &self, sc: &StringCache ) -> String {
    match self.visit( &ExprFormatter::new( sc ) ) {
      Ok( s ) => s,
      Err( error ) => error.msg
    }
  }

  pub fn visit_mut<T, E, V: ExprVisitorMut<T, E>>( &self, visitor: &mut V ) -> Result<T, E>
  {
    match self {
      Self::Assignment( var, right )
        => { let rv = right.visit_mut( visitor )?; visitor.visit_assignment( var, rv ) },
      Self::Binary( left, op , right )
        => { let lv = left.visit_mut( visitor )?; let rv = right.visit_mut( visitor )?;
          visitor.visit_binary( lv, op, rv ) },
      Self::Call( callee, paren , args )
        => { let cv = callee.visit_mut( visitor )?; let va = self.visit_mut_args( visitor, args )?; visitor.visit_call( cv, paren, &va ) },
      Self::Grouping( inner )
        => { let iv = inner.visit_mut( visitor )?; visitor.visit_grouping( iv ) },
      Self::Literal( literal )
        => visitor.visit_literal( literal ),
      Self::Unary( op, expr )
        => { let ev = expr.visit_mut( visitor )?; visitor.visit_unary( op, ev ) },
      Self::Var( op )
        => visitor.visit_var( op )
    }
  }

  fn visit_mut_args<T, E, V: ExprVisitorMut<T, E>>( &self, visitor: &mut V, args: &Vec<Box<Expr>> ) -> Result<Vec<T>, E> {
    let mut results: Vec<T> = vec![];
    for arg in args {
      results.push( arg.visit_mut( visitor )? );
    }
    Ok( results )
  }

  pub fn eval( &self, sc: &mut StringCache, envs: &EnvStack ) -> EvalResult {
    self.visit_mut( &mut ExprEvaluator::new( sc, envs ) )
  }
  
}

pub struct ExprFormatter<'str> {
  sc: &'str StringCache
}

impl<'str> ExprFormatter<'str> {

  pub fn new( sc: &'str StringCache ) -> ExprFormatter<'str> {
    ExprFormatter {
      sc
    }
  }

}

impl<'str> ExprVisitor<String> for ExprFormatter<'str> {

  fn visit_assignment( &self, var: &Token, right: String ) -> Result<String, Error> {
      Ok( format!( "{} = {}", var.get_lexeme( self.sc ), right ) )
  }

  fn visit_binary( &self, left: String, op: &Token, right: String ) -> Result<String, Error> {
    Ok( format!( "{} {} {}", left, op.get_lexeme( self.sc ), right ) )
  }

  fn visit_call( &self, callee: String, _paren: &Token, args: &Vec<String> ) -> Result<String, Error> {

    let no_args = args.len() == 0;

    let mut params = if no_args {
      "(".to_string()
    } else {
      "( ".to_string()
    };

    for ( idx, arg ) in args.iter().enumerate() {
      if idx == 0 {
        params.push_str( arg.as_str() );
      } else {
        params.push_str( format!( ", {}", arg ).as_str() );
      }
    }

    if no_args {
      params.push_str( ")" );
    } else {
      params.push_str( " )" );
    }

    Ok( format!( "{}{}", callee, params ) )
  }

  fn visit_grouping( &self, expr: String ) -> Result<String, Error> {
    Ok( format!( "( {} )", expr ) )
  }

  fn visit_literal( &self, literal: &Token ) -> Result<String, Error> {
    Ok(
      match literal.get_type() {
        TokenType::String( s )
          => format!( "\"{}\"", self.sc.gets( *s ) ),
        _ => format!( "{}", literal.get_lexeme( self.sc ) )
      }
    )
  }

  fn visit_unary( &self, op: &Token, expr: String ) -> Result<String, Error> {
    Ok( format!( "{}{}", op.get_lexeme( self.sc ), expr ) )
  }

  fn visit_var( &self, var: &Token ) -> Result<String, Error> {
      Ok( format!( "{}", var.get_lexeme( self.sc ) ) )
  }

}