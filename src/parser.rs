//////////////////////////////////////////////
// private module rlox::interpreter::parser //
//////////////////////////////////////////////


/////////
// use //
/////////

use crate::token::{Token, TokenType};
use crate::expr::*;
use crate::stmt::*;
use crate::error::Error;


//////////////////
// declarations //
//////////////////

pub struct Parser {
  tokens: Vec<Token>,
  stmts: Vec<Stmt>,
  current: usize,
  had_error: bool
}

type ParseExprResult = Result<Expr, Error>;
type ParseStmtResult = Result<Stmt, Error>;

/////////////////////
// implementations //
/////////////////////

impl Parser {
  pub fn new() -> Parser {
    Parser{
      tokens: Vec::new(),
      stmts: Vec::new(),
      current: 0,
      had_error: false
    }  
  }
  pub fn parse( &mut self, tokens: Vec<Token> ) -> ( Vec<Stmt>, bool ) {
    self.restart( tokens );
    while !self.is_at_end() {
      if self.peek_type() == TokenType::Eof {
        break;
      }
      let e = self.parse_decl();
      match e {
        Ok( stmt ) => {
          self.stmts.push( stmt );
        },
        Err( error ) => {
          self.emit_error( &error );
          break;
        }
      }
    }
    let stmts = self.stmts.clone();
    self.stmts.clear();
    ( stmts, self.had_error )
  }
  fn restart( &mut self, tokens: Vec<Token> ) {
    self.tokens = tokens;
    self.stmts.clear();
    self.current = 0;
    self.had_error = false;
  }

  // decl => fun_decl | var_decl | stmt
  fn parse_decl( &mut self ) -> ParseStmtResult {
    if self.is_fun_decl() {
      Ok( self.parse_fun_decl()? )
    }
    else if self.is_var_decl() {
      Ok( self.parse_var_decl()? )
    } else {
      Ok( self.parse_stmt()? )
    }
  }

  // fun_decl => "fun" id "(" parameters? ")" block_stmt
  // parameters => id ( "," id )*
  fn parse_fun_decl( &mut self ) -> ParseStmtResult {

    // "fun"
    self.pop();

    // id
    let name = self.parse_id()?;

    // "("
    self.pop_assert( TokenType::LeftParen, " to open function parameter list." )?;

    let mut params: Vec<Token> = Vec::new();
    if !self.pop_if( TokenType::RightParen ) {

      // one or more parameters: first
      params.push( self.parse_id()? );

      // the rest
      while self.pop_if( TokenType::Comma ) {
        params.push( self.parse_id()? );
      }

      // ")"
      self.pop_assert( TokenType::RightParen, " to close function parameter list." )?;
    } 

    // block_stmt
    self.peek_assert( TokenType::LeftBrace, " to begin function body." )?;
    if let Stmt::Block( block ) = self.parse_block_stmt()? {

      // success
      return Ok( Stmt::Function( Function{ name, params, body: block.statements } ) )
    }
    unreachable!()
  }

  // var_decl => id ( "=" expr )? ";"
  fn parse_var_decl( &mut self ) -> ParseStmtResult {

    //  "var"
    self.pop();

    // id
    let name = self.parse_id()?;

    // ( "=" expr )?    [ aka init ]
    let init: Option<Expr> = 
      if self.peek_type() == TokenType::Equal {
        self.pop();
        let expr = self.parse_expr()?;
        Some( expr )
      } else {
        None
      };

    // ";"
    self.pop_assert( TokenType::Semicolon, " to complete the variable declaration." )?;
    
    return Ok( Stmt::Var( Var{ name, init } ) );
  }
  fn parse_id( &mut self ) -> Result<Token, Error> {
    match self.peek_type() {
      TokenType::Identifier => { Ok( self.pop() ) },
      _ => Err( self.make_error( "Expected an identifier here.".to_string() ) )
    }
  }

  /// stmt => print_stmt
  ///       | block_stmt
  ///       | if_stmt
  ///       | expr_stmt
  ///       | while_stmt
  ///       | for_stmt
  ///       | return_stmt
  fn parse_stmt( &mut self ) -> ParseStmtResult {
    match self.peek_type() {
      TokenType::Print => self.parse_print_stmt(),
      TokenType::LeftBrace => self.parse_block_stmt(),
      TokenType::If => self.parse_if_stmt(),
      TokenType::While => self.parse_while_stmt(),
      TokenType::For => self.parse_for_stmt(),
      TokenType::Return => self.parse_return_stmt(),
      _ => Ok( self.parse_expr_stmt()? )
    }
  }

  // print_stmt => "print" expr ";"
  fn parse_print_stmt( &mut self ) -> ParseStmtResult {
    
    // "print"
    self.pop();

    // expr
    let expression = self.parse_expr()?;
    
    // ";"
    self.pop_assert( TokenType::Semicolon, " to complete the print statement." )?;

    // success
    Ok( Stmt::Print( Print{ expression } ) )
  }

  // block_stmt => "{" decl* "}"
  fn parse_block_stmt( &mut self ) -> ParseStmtResult {
    
    // "{"
    let line = self.pop().line;

    // decl*
    let mut stmts: Vec<Stmt> = Vec::new();
    while self.peek_type() != TokenType::RightBrace && !self.is_at_end() {
      stmts.push( self.parse_decl()? );
    }

    // "}"
    self.pop_assert( TokenType::RightBrace, " to complete the block statement." )?;

    // success
    Ok( Stmt::Block( Block{ statements: stmts, line } ) )
  }

  // if_stmt = "if" "(" expr ")" stmt ( "else" stmt )?
  fn parse_if_stmt( &mut self ) -> ParseStmtResult {

    // "if"
    self.pop();

    // "("
    self.pop_assert( TokenType::LeftParen, " to open the if-statement condition-clause" )?;

    // expr
    let condition = self.parse_expr()?;
    
    // ")"
    self.pop_assert( TokenType::RightParen, " to close the if-statement condition-clause." )?;

    // stmt
    let then_branch = Box::new( self.parse_stmt()? );

    // ( "else" stmt )?
    let else_branch = if self.peek_type() == TokenType::Else {
      self.pop();
      Some( Box::new( self.parse_stmt()? ) )
    } else {
      None
    };

    // success
    Ok( Stmt::If( If{ condition, then_branch, else_branch } ) )
    
  }

  // while_stmt = "while" "(" expr ")" stmt
  fn parse_while_stmt( &mut self ) -> ParseStmtResult {

    // "while"
    self.pop();

    // "("
    self.pop_assert( TokenType::LeftParen, " to open the while-statement condition-clause." )?;

    // expr
    let condition = self.parse_expr()?;
    
    // ")"
    self.pop_assert( TokenType::RightParen, " to close the while-statement condition-clause." )?;

    // stmt
    let body = Box::new( self.parse_stmt()? );

    // success
    Ok( Stmt::While( While{ condition, body } ) )

  }

  // for_stmt => "for" "(" ( var_decl | expr_stmt | ";" ) ( expr )? ";" ( expr )? ")" stmt
  fn parse_for_stmt( &mut self ) -> ParseStmtResult {

    // "for"
    let line = self.pop().line;

    // "("
    self.pop_assert( TokenType::LeftParen, " to open the for-statement control-clause." )?;
    
    // ( var_decl | expr_stmt | ";" )
    let init: Option<Stmt> = 
      match self.peek_type() {
        TokenType::Var => Some( self.parse_var_decl()? ),
        TokenType::Semicolon => {
          self.pop();
          None
        },
        _ => Some( self.parse_expr_stmt()? )
      };
    
    // ( expr )?
    let condition = if self.peek_type() != TokenType::Semicolon {
      self.parse_expr()?
    } else {
      // desugar condition
      Expr::Literal( Literal {
        value: Token {
          token_type: TokenType::True,
          line,
          lexeme: "true".into()
        }
      } )
    };

    // ";"
    self.pop_assert( TokenType::Semicolon, " to complete the for-statement condition-expression." )?;

    // ( expr )?
    let incr: Option<Expr> = if self.peek_type() != TokenType::RightParen {
      Some( self.parse_expr() ? )
    } else {
      None
    };

    // ")"
    self.pop_assert( TokenType::RightParen, " to close the for-statement control-clause." )?;

    // stmt
    let mut body = self.parse_stmt()?;
    
    // desugar incr
    if incr.is_some() {
      body = Stmt::Block( Block{
        statements: [
          body,
          Stmt::Expression( Expression{
            expression: incr.unwrap()
          } )
        ].to_vec(), line
      } );
    }

    // desugar condition
    body = Stmt::While( While { condition, body: Box::new( body ) } );

    // desugar init
    if init.is_some() {
      body = Stmt::Block( Block { statements: [ init.unwrap(), body ].to_vec(), line } );
    }

    // success
    Ok( body )

  }

  // expr_stmt => expr ";"
  fn parse_expr_stmt( &mut self ) -> ParseStmtResult {
    
    // expr
    let expression = self.parse_expr()?;
            
    // ";"
    self.pop_assert( TokenType::Semicolon, " to complete the expression statement." )?;
    
    // success
    Ok( Stmt::Expression( Expression{ expression } ) )
    
  }

  // return_stmt => "return" expr? ";"
  fn parse_return_stmt( &mut self ) -> ParseStmtResult {

    // "return"
    let keyword = self.pop();

    // expr? ";"
    let value: Option<Expr> = if self.pop_if( TokenType::Semicolon ) {
      None
    } else {
      let expr = self.parse_expr()?;
      self.pop_assert( TokenType::Semicolon, " to complete the return statment." )?;
      Some( expr )
    };

    Ok( Stmt::Return( Return{ keyword, value } ) )
  }

  // expr => assign
  fn parse_expr( &mut self ) -> ParseExprResult {
    self.parse_assign()
  }

  // assign  => ( id "=" assign ) | logical_or
  fn parse_assign( &mut self ) -> ParseExprResult {

    let expr = self.parse_or()?;

    if self.peek_type() == TokenType::Equal {
      let equal = self.pop();
      let rhs = self.parse_assign()?;
      match expr {
        Expr::Variable( lhs ) => {
          Ok( Expr::Assign( Assign {
            lhs,
            rhs: Box::new( rhs )
          } ) )
        }
        _ => Err( Error::from_token( &equal, "Cannot assign to the expression on the left hand side.".to_string() ) )
      }
    }
    else {
      Ok( expr )
    }
  }

  // or  => and ( "or" and )*
  fn parse_or( &mut self ) -> ParseExprResult {
    let mut expr = self.parse_and()?;
    loop {
       if self.is_or() {
        let operator = self.pop();
        let right = self.parse_and()?;
        expr = Expr::Binary( Binary {
          left: Box::new( expr ),
          operator,
          right: Box::new( right )
        } );
      } else {
        break;
      }
    }
    Ok( expr )
  }

  // and => eq ( "and" eq )*
  fn parse_and( &mut self ) -> ParseExprResult {
    let mut expr = self.parse_eq()?;
    loop {
       if self.is_and() {
        let operator = self.pop();
        let right = self.parse_eq()?;
        expr = Expr::Binary( Binary {
          left: Box::new( expr ),
          operator,
          right: Box::new( right )
        } );
      } else {
        break;
      }
    }
    Ok( expr )
  }

  // eq => cmp ( ( "==" | "!=" ) cmp )*
  fn parse_eq( &mut self ) -> ParseExprResult {
    let mut expr = self.parse_cmp()?;
    loop {
       if self.is_eq() {
        let operator = self.pop();
        let right = self.parse_cmp()?;
        expr = Expr::Binary( Binary {
          
          left: Box::new( expr ),
          operator,
          right: Box::new( right )
        } );
      } else {
        break;
      }
    }
    Ok( expr )
  }

  // cmp => term ( ( "<" | "<=" | ">" | ">=" ) term )*
  fn parse_cmp( &mut self ) -> ParseExprResult {
    let mut expr = self.parse_term()?;
    loop {
      if self.is_cmp() {
        let operator = self.pop();
        let right = self.parse_term()?;
        expr = Expr::Binary( Binary {
          left: Box::new( expr ),
          operator,
          right: Box::new( right )
        } );
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
        let operator = self.pop();
        let right = self.parse_factor()?;
        expr = Expr::Binary( Binary {
          left: Box::new( expr ),
          operator,
          right: Box::new( right )
        } );
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
        let operator = self.pop();
        let right = self.parse_unary()?;
        expr = Expr::Binary( Binary {
          
          left: Box::new( expr ),
          operator,
          right: Box::new( right )
        } );
      } else {
        break;
      }
    }
    return Ok( expr );
  }
  
  // unary => ( ( "!" | "-" ) unary ) | call
  fn parse_unary( &mut self ) -> ParseExprResult {
    if self.is_unary() {
        Ok( Expr::Unary( Unary {
          operator: self.pop().clone(),
          right: Box::new( self.parse_unary()? )
        } ) )
    } else {
      self.parse_call()
    }
  }

  // call => grouping ( "(" arguments? ")" )* | grouping
  fn parse_call( &mut self ) -> ParseExprResult {

    let mut expr = self.parse_grouping()?;

    // println!( "parse_call() before loop: expr = {}", expr.to_string( self.sm ) );

    loop {
      if self.pop_if( TokenType::LeftParen ) {
        // println!( "parse_call() found '('" );
        expr = self.parse_arguments( expr )?;
        // println!( "parse_call() args = {}", expr.to_string( self.sm ) );
      }
      else {
        break;
      }
    }

    // println!( "parse_call() returning expr = {}", expr.to_string( self.sm ) );
    Ok( expr )
  }
  fn parse_arguments( &mut self, callee: Expr ) -> ParseExprResult {
    let mut args: Vec<Box<Expr>> = Vec::new();
    let paren =
      if !self.pop_if( TokenType::RightParen ) {
        loop {
          args.push( Box::new( self.parse_expr()? ) );
          if !self.pop_if( TokenType::Comma ) {
            break;
          }
        }
        self.pop_assert( TokenType::RightParen, " to finish function call." )?
      } else {
        self.previous().clone()
      };
    Ok( Expr::Call( Call {
      
      callee: Box::new( callee ),
      paren,
      arguments: args
    } ) )
  }
  
  // grouping => ( "(" expr ")" ) | primary
  fn parse_grouping( &mut self ) -> ParseExprResult {
    if self.is_grouping() {
      self.pop();
      let expr = Expr::Grouping( Grouping {
        expression: Box::new( self.parse_expr()? )
      } );
      self.pop_assert( TokenType::RightParen, " to close the grouping." )?;
      Ok( expr )
    } else {
      self.parse_primary()
    }
  }

  // primary => "true" | "false" | "nil" | IDENTIFIER | NUMBER | STRING
  fn parse_primary( &mut self ) -> ParseExprResult {
    if self.is_primary() {
      if self.is_id() {
        Ok( Expr::Variable( Variable {
          name: self.pop(),
          jump: -1
        } ) )
      } else {
        Ok( Expr::Literal( Literal {
          value: self.pop()
        } ) )
      }
    } else {
      Err( self.make_error( format!( "Expected a primary expression here." ) ) )
    }
  }
  fn is_fun_decl( &self ) -> bool {
    self.peek_type() == TokenType::Fun
  }
  fn is_var_decl( &self ) -> bool {
    self.peek_type() == TokenType::Var
  }
  fn is_or( &self ) -> bool {
    self.peek_type() == TokenType::Or
  }
  fn is_and( &self ) -> bool {
    self.peek_type() == TokenType::And
  }
  fn is_eq( &self ) -> bool {
    match self.peek_type() {
      TokenType::BangEqual
      | TokenType::EqualEqual
        => true,
      _ => false
    }
  }
  fn is_cmp( &self ) -> bool {
    match self.peek_type() {
      TokenType::Greater
      | TokenType::GreaterEqual
      | TokenType::Less
      | TokenType::LessEqual
        => true,
      _ => false
    }
  }
  fn is_term( &self ) -> bool {
    match self.peek_type() {
      TokenType::Minus
      | TokenType::Plus
        => true,
      _ => false
    }
  }
  fn is_factor( &self ) -> bool {
    match self.peek_type() {
      TokenType::Slash
      | TokenType::Star
        => true,
      _ => false
    }
  }
  fn is_id( &self ) -> bool {
    self.peek_type() == TokenType::Identifier
  }
  fn is_unary( &self ) -> bool {
    match self.peek_type() {
      TokenType::Bang
        | TokenType::Minus    
        => true,
      _ => false
    }
  }
  fn is_grouping( &self ) -> bool {
    self.peek_type() == TokenType::LeftParen
  }
  fn is_primary( &self ) -> bool {
    match self.peek_type() {
      TokenType::False
        | TokenType::True
        | TokenType::Nil
        | TokenType::Number
        | TokenType::String
        | TokenType::Identifier
        => true,
      _ => false
    }
  }
  fn pop( &mut self ) -> Token {
    if !self.is_at_end() {
      self.current += 1;
    }
    self.previous().clone()
  }
  fn pop_assert( &mut self, tt: TokenType, loc: &str ) -> Result<Token, Error> {
    if self.peek_type() != tt {
      Err( self.make_error( format!( "Expected '{}'{}", tt.get_lexeme(), loc ) ) )
    }
    else {
      Ok( self.pop() )
    }
  }
  fn pop_if( &mut self, tt: TokenType ) -> bool {
    if self.peek_type() == tt {
      self.pop();
      true
    } else {
      false
    }
  }
  fn peek( &self ) -> &Token {
    if self.is_at_end() {
      self.previous()
    }
    else {
      self.tokens.get( self.current ).unwrap()
    }
  }
  fn peek_type( &self ) -> TokenType {
    self.peek().token_type
  }
  fn peek_assert( &mut self, tt: TokenType, loc: &str ) -> Result<(), Error> {
    if self.peek_type() != tt {
      Err( self.make_error( format!( "Expected '{}'{}", tt.get_lexeme(), loc ) ) )
    }
    else {
      Ok( () )
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
    Error::from_token( self.peek(), msg )
  }
  fn emit_error( &mut self, error: &Error ) {
    eprintln!( "[line {}] Error{}: {}", error.line, error.loc, error.msg );
    self.had_error = true;
  }
}