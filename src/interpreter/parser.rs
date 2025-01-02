//////////////////////////////////////////////
// private module rlox::interpreter::parser //
//////////////////////////////////////////////


/////////
// use //
/////////

use crate::interpreter::decl::*;
use crate::interpreter::error::*;
use crate::interpreter::token::*;
use crate::interpreter::expr::*;
use crate::interpreter::stmt::*;

use crate::util::StringManager;


//////////////////////
// public interface //
//////////////////////

pub struct Parser<'str> {
  db: &'str StringManager,
  tokens: Vec<Token>,
  decls: Vec<Decl>,
  current: usize,
  had_error: bool
}

type ParseExprResult = Result<Expr, Error>;
type ParseStmtResult = Result<Stmt, Error>;
type ParseDeclResult = Result<Decl, Error>;

impl<'str> Parser<'str> {

  pub fn new( db: &'str StringManager ) -> Parser<'str> {
    Parser{
      db,
      tokens: vec![],
      decls: vec![],
      current: 0,
      had_error: false
    }  
  }

  pub fn parse( &mut self, tokens: Vec<Token> ) -> ( Vec<Decl>, bool ) {
    self.restart( tokens );
    while !self.is_at_end() {
      if *self.peek().get_token_type() == TokenType::Eof {
        break;
      }
      let e = self.parse_declaration();
      match e {
        Ok( decl ) => {
          self.decls.push( decl );
        },
        Err( error ) => {
          if error.line > 0 {
            self.emit_error( &error );
          }
          else {
            eprintln!( "Ignoring error: {:?}", error );
          }
          break;
        }
      }
    }
    let decls = self.decls.clone();
    self.decls.clear();
    ( decls, self.had_error )
  }
 

  ////////////////////////////
  // private implementation //
  ////////////////////////////
  
  fn restart( &mut self, tokens: Vec<Token> ) {
    self.tokens = tokens;
    self.decls.clear();
    self.current = 0;
    self.had_error = false;
  }

  // declaration => var_declaration | statement
  // var_declaration => identifer ( "=" expression )? ";"
  fn parse_declaration( &mut self ) -> ParseDeclResult {

    // var_declaration
    if self.is_var_decl() {
      return Ok( self.parse_var_decl()? );
    }

    // statement
    let stmt = self.parse_statement()?;
    Ok( Decl::Stmt( stmt ) )
  }

  fn parse_var_decl( &mut self ) -> ParseDeclResult {

    // consume "var"
    self.pop();

    // identifier
    let identifier = self.parse_identifier()?;

    // ( "=" expression )?    [ aka tail ]
    let tail: Option<Expr> = 
      if *self.peek().get_token_type() == TokenType::Equal {
        self.pop();
        let expr = self.parse_expression()?;
        Some( expr )
      } else {
        None
      };

    // ";"
    if *self.peek().get_token_type() != TokenType::Semicolon {
      return Err( self.make_error( format!( "Expected ';' here." ) ) )
    }
    self.pop();
    
    return Ok( Decl::Var( identifier, tail ) );
  }

  fn parse_identifier( &mut self ) -> Result<Token, Error> {
    match *self.peek().get_token_type() {
      TokenType::Identifer( _ ) => { Ok( *self.pop() ) },
      _ => Err( self.make_error( "Expected an identifier here.".to_string() ) )
    }
  }

  // statement => print_statement | expr_statement
  // print_statement => "print" expression ";"
  // expr_statement => expression ";"
  fn parse_statement( &mut self ) -> ParseStmtResult {

    // print_statement
    if *self.peek().get_token_type() == TokenType::Print {

      // consume "print"
      self.pop();

      // expression
      let expr = self.parse_expression()?;
      
      // ";"
      if *self.peek().get_token_type() != TokenType::Semicolon {
        return Err( self.make_error( format!( "Expected ';' here." ) ) )
      }
      self.pop();

      // success
      Ok( Stmt::Print( expr ) )

    // expr_statement
    } else {

      // expression
      let expr = self.parse_expression()?;
            
      // ";"
      if *self.peek().get_token_type() != TokenType::Semicolon && !self.is_at_end() {
        return Err( self.make_error( format!( "Expected ';' here." ) ) )
      }
      self.pop();

      // success
      Ok( Stmt::Expr( expr ) )
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
        expr = Expr::Binary( Box::new( expr ), operator, Box::new( right ) );
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
        expr = Expr::Binary( Box::new( expr ), operator, Box::new( right ) );
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
        expr = Expr::Binary( Box::new( expr ), operator, Box::new( right ) );
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
        expr = Expr::Binary( Box::new( expr ), operator, Box::new( right ) );
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
        expr = Expr::Binary( Box::new( expr ), operator, Box::new( right ) );
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
        expr = Expr::Binary( Box::new( expr ), operator, Box::new( right ) );
      } else {
        break;
      }
    }
    return Ok( expr );
  }
  
  // unary => ( ( "!" | "-" ) unary ) | grouping
  fn parse_unary( &mut self ) -> ParseExprResult {
    if self.is_unary() {
        Ok( Expr::Unary( *self.pop(), Box::new( self.parse_unary()? ) ) )
    } else {
      self.parse_grouping()
    }
  }
  
  // grouping => ( "(" expression ")" ) | primary
  fn parse_grouping( &mut self ) -> ParseExprResult {
    if self.is_grouping() {
      self.pop();
      let expr = Expr::Grouping( Box::new( self.parse_expression()? ) );
      if *self.peek().get_token_type() != TokenType::RightParen {
        Err( self.make_error( format!( "Expected ')' here." ) ) )
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
      Ok( Expr::Literal( *self.pop() ) )
    // } else if *self.peek().get_token_type() == TokenType::Eof {
    //   Err( Error::new( -1, "".to_string(), "".to_string() ) )
    } else {
      Err( self.make_error( format!( "Expected a primary expression here." ) ) )
    }
  }

  fn is_var_decl( &self ) -> bool {
    *self.peek().get_token_type() == TokenType::Var
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
        | TokenType::Identifer( _ )
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

  fn make_error( &self, msg: String ) -> Error {
    Error::from_token( self.peek(), msg, self.db )
  }

  fn emit_error( &mut self, error: &Error ) {
    eprintln!( "[line {}] Error{}: {}", error.line, error.loc, error.msg );
    self.had_error = true;
  }

}