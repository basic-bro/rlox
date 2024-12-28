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
      let e = self.expression();
      println!( "{:#}", e );
      self.exprs.push(e);
    }
  }

  ////////////////////////////
  // private implementation //
  ////////////////////////////
  
  fn expression( &mut self ) -> Box<Expr<'src>> {
    self.equality()
  }

  fn equality( &mut self ) -> Box<Expr<'src>> {
    let mut expr = self.comparison();
    loop {
      let tt = *self.peek().get_token_type();
      if tt == TokenType::BangEqual || tt == TokenType::EqualEqual {
        let operator = *self.pop();
        let right = self.comparison();
        expr = Box::new( Expr::Binary( expr, operator, right ) );
      } else {
        break;
      }
    }
    return expr;
  }
  
  fn comparison( &mut self ) -> Box<Expr<'src>> {
    let mut expr = self.term();
    loop {
      let tt = *self.peek().get_token_type();
      if tt == TokenType::Greater || tt == TokenType::GreaterEqual || tt == TokenType::Less || tt == TokenType::LessEqual {
        let operator = *self.pop();
        let right = self.term();
        expr = Box::new( Expr::Binary( expr, operator, right ) );
      } else {
        break;
      }
    }
    return expr;
  }
  
  fn term( &mut self ) -> Box<Expr<'src>> {
    let mut expr = self.factor();
    loop {
      let tt = *self.peek().get_token_type();
      if tt == TokenType::Minus || tt == TokenType::Plus {
        let operator = *self.pop();
        let right = self.factor();
        expr = Box::new( Expr::Binary( expr, operator, right ) );
      } else {
        break;
      }
    }
    return expr;
  }

  fn factor( &mut self ) -> Box<Expr<'src>> {
    let mut expr = self.unary();
    loop {
      let tt = *self.peek().get_token_type();
      if tt == TokenType::Slash || tt == TokenType::Star {
        let operator = *self.pop();
        let right = self.unary();
        expr = Box::new( Expr::Binary( expr, operator, right ) );
      } else {
        break;
      }
    }
    return expr;
  }
  
  fn unary( &mut self ) -> Box<Expr<'src>> {
    match self.peek().get_token_type() {
      TokenType::Bang
        | TokenType::Minus
        => Box::new( Expr::Unary( *self.pop(), self.unary() ) ),
      _ => self.primary()
    }
  }

  fn primary( &mut self ) -> Box<Expr<'src>> {
    match self.peek().get_token_type() {
      TokenType::False
        | TokenType::True
        | TokenType::Nil
        | TokenType::Number( _ )
        | TokenType::String( _ )
        | TokenType::Identifer( _ )
        => Box::new( Expr::Literal( *self.pop() ) ),
      _ => panic!( "Expected a primary expression but found '{:?}'", *self.pop() )
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