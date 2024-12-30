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

type ParseResult<'src> = Result<Box<Expr<'src>>, String>;

#[derive(Debug)]
enum Eval {
  Number( f64 ),
  StringLiteral( String )
}
  
type EvalResult = Result<Eval, String>;

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
      match e {
        Ok( expr ) => {
          println!( "{:#}", expr );
          println!( "\t{:?}", self.eval( &expr ) );
          //self.exprs.push( e );
        },
        Err( msg ) => {
          eprintln!( "{}", msg );
          break;
        }
      }
    }
  }
  

  ////////////////////////////
  // private implementation //
  ////////////////////////////

  fn eval( &self, expr: &Expr<'src>  ) -> EvalResult {
      match expr {
        Expr::Literal( t ) => self.eval_literal( t ),
        Expr::Grouping( expr ) => self.eval_grouping( expr ),
        Expr::Binary( left, op, right ) => self.eval_binary( left, op, right ),
        Expr::Unary( op, expr ) => self.eval_unary( op, expr )
      }
    }
  
  fn eval_binary( &self, left: &Expr<'src>, op: &Token<'src>, right: &Expr<'src> ) -> EvalResult {
    let left_eval = self.eval( left )?;
    let right_eval = self.eval( right )?;
    match ( &left_eval, &right_eval ) {
      ( Eval::Number( x ), Eval::Number( y ) )
        =>  match op.get_token_type() {
              TokenType::Plus => Ok( Eval::Number( x + y ) ),
              TokenType::Minus => Ok( Eval::Number( x - y ) ),
              TokenType::Star => Ok( Eval::Number( x * y ) ),
              TokenType::Slash => Ok( Eval::Number( x / y ) ),
              _ => Err( format!( "Unknown binary operation on numbers: '{}'", op.get_lexeme() ) )
            },
      ( Eval::StringLiteral( x ), Eval::StringLiteral( y ) )
        =>  match op.get_token_type() {
              TokenType::Plus => Ok( Eval::StringLiteral( x.to_owned() + y ) ),
              _ => Err( format!( "Unknown binary operation on strings: '{}'", op.get_lexeme() ) )
            },
      _ => Err( format!( "Unknown or unsupported binary operation '{}' on values {:?} and {:?}", op.get_lexeme(), left_eval, right_eval ) )
    }
  }

  fn eval_literal( &self, literal: &Token<'src> ) -> EvalResult {
    match literal.get_token_type() {
      TokenType::String( s ) => Ok( Eval::StringLiteral( s.to_string() ) ),
      TokenType::Number( s ) => Ok( Eval::Number( s.parse::<f64>().unwrap() ) ),
      TokenType::Identifer( _ ) => Err( format!( "eval() not implemented yet: {:?}", literal ) ),
      _ => Err( format!( "Internal error: this token should not be parsed as an Expr::Literal: {:?}", literal ) )
    }
  }

  fn eval_grouping( &self, inner: &Expr<'src> ) -> EvalResult {
    self.eval( inner )
  }

  fn eval_unary( &self, op: &Token<'src>, expr: &Expr<'src> ) -> EvalResult {
    let expr_eval = self.eval( expr )?;
    match &expr_eval {
      Eval::Number( x )
        => if *op.get_token_type() == TokenType::Minus {
          Ok( Eval::Number( -x ) )
        } else {
          Err( format!( "Unknown unary operation on a number: '{}'", op.get_lexeme() ) )
        },
      _ => Err( format!( "Unary operator '{}' not implemented for {:?}", op.get_lexeme(), &expr_eval ) )
    }
  }
  
  fn parse_expression( &mut self ) -> ParseResult<'src> {
    self.parse_equality()
  }

  fn parse_equality( &mut self ) -> ParseResult<'src> {
    let mut expr = self.parse_comparison()?;
    loop {
       if self.is_equality() {
        let operator = *self.pop();
        let right = self.parse_comparison()?;
        expr = Box::new( Expr::Binary( expr, operator, right ) );
      } else {
        break;
      }
    }
    Ok( expr )
  }

  fn parse_comparison( &mut self ) -> ParseResult<'src> {
    let mut expr = self.parse_term()?;
    loop {
      if self.is_comparison() {
        let operator = *self.pop();
        let right = self.parse_term()?;
        expr = Box::new( Expr::Binary( expr, operator, right ) );
      } else {
        break;
      }
    }
    return Ok( expr );
  }
  
  fn parse_term( &mut self ) -> ParseResult<'src> {
    let mut expr = self.parse_factor()?;
    loop {
      if self.is_term() {
        let operator = *self.pop();
        let right = self.parse_factor()?;
        expr = Box::new( Expr::Binary( expr, operator, right ) );
      } else {
        break;
      }
    }
    return Ok( expr );
  }

  fn parse_factor( &mut self ) -> ParseResult<'src> {
    let mut expr = self.parse_unary()?;
    loop {
      if self.is_factor()  {
        let operator = *self.pop();
        let right = self.parse_unary()?;
        expr = Box::new( Expr::Binary( expr, operator, right ) );
      } else {
        break;
      }
    }
    return Ok( expr );
  }
  
  fn parse_unary( &mut self ) -> ParseResult<'src> {
    if self.is_unary() {
        Ok( Box::new( Expr::Unary( *self.pop(), self.parse_unary()? ) ) )
    } else {
      self.parse_grouping()
    }
  }

  fn parse_grouping( &mut self ) -> ParseResult<'src> {
    if self.is_grouping() {
      self.pop();
      let expr = Box::new( Expr::Grouping( self.parse_expression()? ) );
      if *self.peek().get_token_type() != TokenType::RightParen {
        Err( format!( "Expected ')' but found '{:?}'", *self.peek() ) )
      } else {
        self.pop();
        Ok( expr )
      }
    } else {
      self.parse_primary()
    }
  }

  fn parse_primary( &mut self ) -> ParseResult<'src> {
    if self.is_primary() {
      Ok( Box::new( Expr::Literal( *self.pop() ) ) )
    } else {
      Err( format!( "Expected a primary expression but found '{:?}'", *self.peek() ) )
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

  fn is_grouping( &self ) -> bool {
    *self.peek().get_token_type() == TokenType::LeftParen
  }

  fn is_primary( &self ) -> bool {
    match self.peek().get_token_type() {
      TokenType::False
        | TokenType::True
        | TokenType::Nil
        | TokenType::Number( _ )
        | TokenType::String( _ )
        //| TokenType::Identifer( _ )
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