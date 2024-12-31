//////////////////////////////////////////////
// private module rlox::interpreter::parser //
//////////////////////////////////////////////


/////////
// use //
/////////

use crate::interpreter::token::*;
use crate::interpreter::expr::*;
use crate::interpreter::stmt::*;
use crate::interpreter::eval::*;

use crate::util::StringManager;


//////////////////////
// public interface //
//////////////////////

pub struct Parser<'str> {
  db: &'str StringManager,
  tokens: Vec<Token>,
  stmts: Vec<Box<Stmt>>,
  current: usize
}

type ParseExprResult = Result<Box<Expr>, String>;
type ParseStmtResult = Result<Box<Stmt>, String>;

impl<'str> Parser<'str> {

  pub fn new( db: &'str StringManager, tokens: Vec<Token> ) -> Parser<'str> {
    Parser{
      db,
      tokens,
      stmts: vec![],
      current: 0
    }  
  }

  pub fn parse( &mut self ) {
    while !self.is_at_end() {
      let e = self.parse_statement();
      match e {
        Ok( stmt ) => {
          self.exec( &stmt );
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

  fn eval( &self, expr: &Expr ) -> EvalResult {
    expr.visit( &ExprEvaluator::new( self.db ) )
  }

  fn exec( &self, stmt: &Stmt ) {
    match stmt {
      Stmt::Expr( expr ) => {
        match self.eval( expr ) {
          Ok( eval ) => println!( "[ Interpreter: ignoring an expression statement which evaluated to '{}'. ]", eval ),
          Err( msg ) => eprintln!( "[ Interpreter: error while evaluating an expression statement: {}", msg )
        }
      },
      Stmt::Print( expr ) => {
        match self.eval( expr ) {
          Ok( eval ) => print!( "{}", eval ),
          Err( msg ) => eprintln!( "[ Interpreter: error while evaluating a print statement: {}", msg )
        }
      }
    }
  }

  // statement => print_statement | expr_statement
  // print_statement => "print" expression ";"
  // expr_statement => expression ";"
  fn parse_statement( &mut self ) -> ParseStmtResult {
    if *self.peek().get_token_type() == TokenType::Print {

      // consume "print"
      self.pop();

      // expression
      let result = self.parse_expression();
      if result.is_err() {
        return Err( result.err().unwrap() )
      }
      
      // ";"
      if *self.peek().get_token_type() != TokenType::Semicolon {
        return Err( format!( "Expected ';' but found '{}'", self.peek().get_lexeme( self.db ) ) );
      }
      self.pop();

      // success
      Ok( Box::new( Stmt::Print( result.unwrap() ) ) )

    } else {

      // expression
      let result = self.parse_expression();
      if result.is_err() {
        return Err( result.err().unwrap() )
      }
      
      // ";"
      if *self.peek().get_token_type() != TokenType::Semicolon {
        return Err( format!( "Expected ';' but found '{}'", self.peek().get_lexeme( self.db ) ) );
      }
      self.pop();

      // success
      Ok( Box::new( Stmt::Expr( result.unwrap() ) ) )
    }
  }

  // expression  => assignment
  fn parse_expression( &mut self ) -> ParseExprResult {
    self.parse_assignment()
  }

  // assignment  => ( IDENTIFIER "=" assignment ) | logical_or
  fn parse_assignment( &mut self ) -> ParseExprResult {
    self.parse_logical_or()
  }

  // logical_or  => logical_and ( "or" logical_and )*
  fn parse_logical_or( &mut self ) -> ParseExprResult {
    let mut expr = self.parse_logical_and()?;
    loop {
       if self.is_logical_or() {
        let operator = *self.pop();
        let right = self.parse_logical_and()?;
        expr = Box::new( Expr::Binary( expr, operator, right ) );
      } else {
        break;
      }
    }
    Ok( expr )
  }

  // logical_and => equality ( "and" equality )*
  fn parse_logical_and( &mut self ) -> ParseExprResult {
    let mut expr = self.parse_equality()?;
    loop {
       if self.is_logical_and() {
        let operator = *self.pop();
        let right = self.parse_equality()?;
        expr = Box::new( Expr::Binary( expr, operator, right ) );
      } else {
        break;
      }
    }
    Ok( expr )
  }

  // equality => comparison ( ( "==" | "!=" ) comparison )*
  fn parse_equality( &mut self ) -> ParseExprResult {
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

  // equality => term ( ( "<" | "<=" | ">" | ">=" ) term )*
  fn parse_comparison( &mut self ) -> ParseExprResult {
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
  
  // term => factor ( ( "+" | "-" ) factor )*
  fn parse_term( &mut self ) -> ParseExprResult {
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

  // factor => unary ( ( "*" | "/" ) unary )*
  fn parse_factor( &mut self ) -> ParseExprResult {
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
  
  // unary => ( ( "!" | "-" ) unary ) | grouping
  fn parse_unary( &mut self ) -> ParseExprResult {
    if self.is_unary() {
        Ok( Box::new( Expr::Unary( *self.pop(), self.parse_unary()? ) ) )
    } else {
      self.parse_grouping()
    }
  }
  
  // grouping => ( "(" expression ")" ) | primary
  fn parse_grouping( &mut self ) -> ParseExprResult {
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

  // primary => "true" | "false" | "nil" | NUMBER | STRING_LITERAL
  fn parse_primary( &mut self ) -> ParseExprResult {
    if self.is_primary() {
      Ok( Box::new( Expr::Literal( *self.pop() ) ) )
    } else {
      Err( format!( "Expected a primary expression but found '{:?}'", *self.peek() ) )
    }
  }

  fn is_logical_or( &self ) -> bool {
    *self.peek().get_token_type() == TokenType::Or
  }

  fn is_logical_and( &self ) -> bool {
    *self.peek().get_token_type() == TokenType::And
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

  fn pop( &mut self ) -> &Token {
    if !self.is_at_end() {
      self.current += 1;
    }
    self.previous()
  }

  fn peek( &self ) -> &Token {
    if self.is_at_end() {
      self.previous()
    }
    else {
      self.tokens.get( self.current ).unwrap()
    }
  }

  fn previous( &self ) -> &Token {
    assert!( self.current > 0 && self.current - 1 < self.tokens.len() );
    self.tokens.get( self.current - 1 ).unwrap()
  }

  fn is_at_end( &self ) -> bool {
    self.current >= self.tokens.len()
  }

}