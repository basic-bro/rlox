//////////////////////////////////////////////
// private module rlox::interpreter::parser //
//////////////////////////////////////////////


/////////
// use //
/////////

use crate::interpreter::token::*;
use crate::interpreter::expr::*;


//////////////////////
// public interface //
//////////////////////

pub struct Parser<'src> {
  tokens: Vec<Token<'src>>,
  exprs: Vec<Box<Expr<'src>>>,
  current: usize
}

impl<'src> Parser<'src> {

  pub fn new( tokens: Vec<Token<'src>> ) -> Parser<'src> {
    Parser{
      tokens,
      exprs: vec![],
      current: 0
    }
  }

  pub fn parse( &mut self ) {
    while !self.is_at_end() {
      let e = self.parse_expression();
      println!( "{:#}", e );
      self.exprs.push(e);
    }
  }
  

  ////////////////////////////
  // private implementation //
  ////////////////////////////
  
  fn parse_expression( &mut self ) -> Box<Expr<'src>> {
    self.parse_equality()
  }

  fn parse_equality( &mut self ) -> Box<Expr<'src>> {
    let mut expr = self.parse_comparison();
    loop {
       if self.is_equality() {
        let operator = *self.pop();
        let right = self.parse_comparison();
        expr = Box::new( Expr::Binary( expr, operator, right ) );
      } else {
        break;
      }
    }
    return expr;
  }

  fn parse_comparison( &mut self ) -> Box<Expr<'src>> {
    let mut expr = self.parse_term();
    loop {
      if self.is_comparison() {
        let operator = *self.pop();
        let right = self.parse_term();
        expr = Box::new( Expr::Binary( expr, operator, right ) );
      } else {
        break;
      }
    }
    return expr;
  }
  
  fn parse_term( &mut self ) -> Box<Expr<'src>> {
    let mut expr = self.parse_factor();
    loop {
      if self.is_term() {
        let operator = *self.pop();
        let right = self.parse_factor();
        expr = Box::new( Expr::Binary( expr, operator, right ) );
      } else {
        break;
      }
    }
    return expr;
  }

  fn parse_factor( &mut self ) -> Box<Expr<'src>> {
    let mut expr = self.parse_unary();
    loop {
      if self.is_factor()  {
        let operator = *self.pop();
        let right = self.parse_unary();
        expr = Box::new( Expr::Binary( expr, operator, right ) );
      } else {
        break;
      }
    }
    return expr;
  }
  
  fn parse_unary( &mut self ) -> Box<Expr<'src>> {
    if self.is_unary() {
        Box::new( Expr::Unary( *self.pop(), self.parse_unary() ) )
    } else {
      self.parse_primary()
    }
  }

  fn parse_primary( &mut self ) -> Box<Expr<'src>> {
    if self.is_primary() {
      Box::new( Expr::Literal( *self.pop() ) )
    } else {
      panic!( "Expected a primary expression but found '{:?}'", *self.pop() )
    }
  }

  fn is_equality( &self ) -> bool {
    match self.peek().get_token_type() {
      TokenType::BangEqual
      | TokenType::EqualEqual
        => true,
      _ => false
    }
  }

  fn is_comparison( &self ) -> bool {
    match self.peek().get_token_type() {
      TokenType::Greater
      | TokenType::GreaterEqual
      | TokenType::Less
      | TokenType::LessEqual
        => true,
      _ => false
    }
  }

  fn is_term( &self ) -> bool {
    match self.peek().get_token_type() {
      TokenType::Minus
      | TokenType::Plus
        => true,
      _ => false
    }
  }

  fn is_factor( &self ) -> bool {
    match self.peek().get_token_type() {
      TokenType::Slash
      | TokenType::Star
        => true,
      _ => false
    }
  }

  fn is_unary( &self ) -> bool {
    match self.peek().get_token_type() {
      TokenType::Bang
        | TokenType::Minus    
        => true,
      _ => false
    }
  }

  fn is_primary( &self ) -> bool {
    match self.peek().get_token_type() {
      TokenType::False
        | TokenType::True
        | TokenType::Nil
        | TokenType::Number( _ )
        | TokenType::String( _ )
        | TokenType::Identifer( _ )
        => true,
      _ => false
    }
  }

  fn pop( &mut self ) -> &Token<'src> {
    if !self.is_at_end() {
      self.current += 1;
    }
    self.previous()
  }

  fn peek( &self ) -> &Token<'src> {
    if self.is_at_end() {
      self.previous()
    }
    else {
      self.tokens.get( self.current ).unwrap()
    }
  }

  fn previous( &self ) -> &Token<'src> {
    assert!( self.current > 0 && self.current - 1 < self.tokens.len() );
    self.tokens.get( self.current - 1 ).unwrap()
  }

  fn is_at_end( &self ) -> bool {
    self.current >= self.tokens.len()
  }

}