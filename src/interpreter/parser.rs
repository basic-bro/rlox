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
  sm: &'str StringManager,
  tokens: Vec<Token>,
  decls: Vec<Decl>,
  current: usize,
  had_error: bool
}

type ParseExprResult = Result<Expr, Error>;
type ParseStmtResult = Result<Stmt, Error>;
type ParseDeclResult = Result<Decl, Error>;

impl<'str> Parser<'str> {

  pub fn new( sm: &'str StringManager ) -> Parser<'str> {
    Parser{
      sm,
      tokens: vec![],
      decls: vec![],
      current: 0,
      had_error: false
    }  
  }

  pub fn parse( &mut self, tokens: Vec<Token> ) -> ( Vec<Decl>, bool ) {
    self.restart( tokens );
    while !self.is_at_end() {
      if self.peek_type() == TokenType::Eof {
        break;
      }
      let e = self.parse_decl();
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

  // decl => var_decl | stmt
  fn parse_decl( &mut self ) -> ParseDeclResult {
    if self.is_var_decl() {
      Ok( self.parse_var_decl()? )
    } else {
      Ok( Decl::Stmt( self.parse_stmt()? ) )
    }
  }

  // var_decl => id ( "=" expr )? ";"
  fn parse_var_decl( &mut self ) -> ParseDeclResult {

    //  "var"
    self.pop();

    // id
    let id = self.parse_id()?;

    // ( "=" expr )?    [ aka tail ]
    let tail: Option<Expr> = 
      if self.peek_type() == TokenType::Equal {
        self.pop();
        let expr = self.parse_expr()?;
        Some( expr )
      } else {
        None
      };

    // ";"
    self.pop_assert( TokenType::Semicolon, " to complete the variable declaration." )?;
    
    return Ok( Decl::Var( id, tail ) );
  }

  fn parse_id( &mut self ) -> Result<Token, Error> {
    match self.peek_type() {
      TokenType::Identifier( _ ) => { Ok( *self.pop() ) },
      _ => Err( self.make_error( "Expected an identifier here.".to_string() ) )
    }
  }

  /// stmt => print_stmt
  ///       | block_stmt
  ///       | if_stmt
  ///       | expr_stmt
  ///       | while_stmt
  ///       | for_stmt
  fn parse_stmt( &mut self ) -> ParseStmtResult {
    match self.peek_type() {
      TokenType::Print => self.parse_print_stmt(),
      TokenType::LeftBrace => self.parse_block_stmt(),
      TokenType::If => self.parse_if_stmt(),
      TokenType::While => self.parse_while_stmt(),
      TokenType::For => self.parse_for_stmt(),
      _ => self.parse_expr_stmt()
    }
  }

  // print_stmt => "print" expr ";"
  fn parse_print_stmt( &mut self ) -> ParseStmtResult {
    
    // "print"
    self.pop();

    // expr
    let expr = self.parse_expr()?;
    
    // ";"
    self.pop_assert( TokenType::Semicolon, " to complete the print statement." )?;

    // success
    Ok( Stmt::Print( expr ) )
  }

  // block_stmt => "{" decl* "}"
  fn parse_block_stmt( &mut self ) -> ParseStmtResult {
    
    // "{"
    let line = self.pop().get_line();

    // decl*
    let mut decls: Vec<Decl> = Vec::new();
    while self.peek_type() != TokenType::RightBrace && !self.is_at_end() {
      decls.push( self.parse_decl()? );
    }

    // "}"
    self.pop_assert( TokenType::RightBrace, " to complete the block statement." )?;

    // success
    Ok( Stmt::Block( decls, line ) )
    
  }

  // if_stmt = "if" "(" ( var_decl | expr_stmt )? expr ")" stmt ( "else" stmt )?
  fn parse_if_stmt( &mut self ) -> ParseStmtResult {

    // "if"
    self.pop();

    // "("
    self.pop_assert( TokenType::LeftParen, " to open the if-statement condition-clause" )?;

    // ( var_decl | expr_stmt )? expr
    let init: Option<CtrlFlowInit>;
    let condition: Expr;
    if self.peek_type() == TokenType::Var {
      init = Some( CtrlFlowInit::VarDecl( Box::new( self.parse_var_decl()? ) ) );
      condition = self.parse_expr()?;
    }
    else {
      let expr_stmt_expr_or_condition = self.parse_expr()?;
      if self.peek_type() == TokenType::Semicolon {
        self.pop();
        init = Some( CtrlFlowInit::ExprStmt( Box::new( Stmt::Expr( expr_stmt_expr_or_condition ) ) ) );
        condition = self.parse_expr()?;
      }
      else {
        init = None;
        condition = expr_stmt_expr_or_condition;
      }
    }

    // ")"
    self.pop_assert( TokenType::RightParen, " to close the if-statement condition-clause." )?;

    // stmt
    let then = Box::new( self.parse_stmt()? );

    // ( "else" stmt )?
    let else_ = if self.peek_type() == TokenType::Else {
      self.pop();
      Some( Box::new( self.parse_stmt()? ) )
    } else {
      None
    };

    // success
    Ok( Stmt::If( init, condition, then, else_ ) )
    
  }

  // while_stmt = "while" "(" ( var_decl | expr_stmt )? expr ")" stmt
  fn parse_while_stmt( &mut self ) -> ParseStmtResult {

    // "while"
    self.pop();

    // "("
    self.pop_assert( TokenType::LeftParen, " to open the while-statement condition-clause." )?;

    // ( var_decl | expr_stmt )? expr
    let init: Option<CtrlFlowInit>;
    let condition: Expr;
    if self.peek_type() == TokenType::Var {
      init = Some( CtrlFlowInit::VarDecl( Box::new( self.parse_var_decl()? ) ) );
      condition = self.parse_expr()?;
    }
    else {
      let expr_stmt_expr_or_condition = self.parse_expr()?;
      if self.peek_type() == TokenType::Semicolon {
        self.pop();
        init = Some( CtrlFlowInit::ExprStmt( Box::new( Stmt::Expr( expr_stmt_expr_or_condition ) ) ) );
        condition = self.parse_expr()?;
      }
      else {
        init = None;
        condition = expr_stmt_expr_or_condition;
      }
    }
    
    // ")"
    self.pop_assert( TokenType::RightParen, " to close the while-statement condition-clause." )?;

    // stmt
    let body = Box::new( self.parse_stmt()? );

    // success
    Ok( Stmt::While( init, condition, body ) )

  }

  // for_stmt => "for" "(" ( var_decl | expr_stmt | ";" ) ( expr )? ";" ( expr )? ")" stmt
  fn parse_for_stmt( &mut self ) -> ParseStmtResult {

    // "for"
    self.pop();

    // "("
    self.pop_assert( TokenType::LeftParen, " to open the for-statement control-clause." )?;
    
    // ( var_decl | expr_stmt | ";" )
    let init: Option<CtrlFlowInit> = 
      match self.peek_type() {

        TokenType::Var => Some( CtrlFlowInit::VarDecl( Box::new( self.parse_var_decl()? ) ) ),

        TokenType::Semicolon => {
          self.pop();
          None
        },

        _ => Some( CtrlFlowInit::ExprStmt( Box::new( self.parse_expr_stmt()? ) ) )
      };
    
    // ( expr )?
    let condition: Option<Expr> = if self.peek_type() != TokenType::Semicolon {
      Some( self.parse_expr() ? )
    } else {
      None
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
    let body = Box::new( self.parse_stmt()? );

    // success
    Ok( Stmt::For( init, condition, incr, body ) )

  }

  // expr_stmt => expr ";"
  fn parse_expr_stmt( &mut self ) -> ParseStmtResult {
    
    // expr
    let expr = self.parse_expr()?;
            
    // ";"
    self.pop_assert( TokenType::Semicolon, " to complete the expression statement." )?;
    
    // success
    Ok( Stmt::Expr( expr ) )
    
  }

  // expr  => assign
  fn parse_expr( &mut self ) -> ParseExprResult {
    self.parse_assign()
  }

  // assign  => ( id "=" assign ) | logical_or
  fn parse_assign( &mut self ) -> ParseExprResult {

    let expr = self.parse_or()?;

    if self.peek_type() == TokenType::Equal {
      let equal = *self.pop();
      let value = self.parse_assign()?;
      match expr {
        Expr::Var( var ) => {
          Ok( Expr::Assignment( var, Box::new( value ) ) )
        }
        _ => Err( Error::from_token( &equal, "Cannot assign to the expression on the left hand side.".to_string(), self.sm ) )
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
        let operator = *self.pop();
        let right = self.parse_and()?;
        expr = Expr::Binary( Box::new( expr ), operator, Box::new( right ) );
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
        let operator = *self.pop();
        let right = self.parse_eq()?;
        expr = Expr::Binary( Box::new( expr ), operator, Box::new( right ) );
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
        let operator = *self.pop();
        let right = self.parse_cmp()?;
        expr = Expr::Binary( Box::new( expr ), operator, Box::new( right ) );
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
  
  // grouping => ( "(" expr ")" ) | primary
  fn parse_grouping( &mut self ) -> ParseExprResult {
    if self.is_grouping() {
      self.pop();
      let expr = Expr::Grouping( Box::new( self.parse_expr()? ) );
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
        Ok( Expr::Var( *self.pop() ) )
      } else {
        Ok( Expr::Literal( *self.pop() ) )
      }
    } else {
      Err( self.make_error( format!( "Expected a primary expression here." ) ) )
    }
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
    if let TokenType::Identifier( _ ) = self.peek_type() {
      true
    }
    else {
      false
    }
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
        | TokenType::Number( _ )
        | TokenType::String( _ )
        | TokenType::Identifier( _ )
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

  fn pop_assert( &mut self, tt: TokenType, loc: &str ) -> Result<&Token, Error> {
    if self.peek_type() != tt {
      Err( self.make_error( format!( "Expected '{}'{}", tt.get_lexeme( self.sm ), loc ) ) )
    }
    else {
      Ok( self.pop() )
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
    *self.peek().get_type()
  }

  fn previous( &self ) -> &Token {
    assert!( self.current > 0 && self.current - 1 < self.tokens.len() );
    self.tokens.get( self.current - 1 ).unwrap()
  }

  fn is_at_end( &self ) -> bool {
    self.current >= self.tokens.len()
  }

  fn make_error( &self, msg: String ) -> Error {
    Error::from_token( self.peek(), msg, self.sm )
  }

  fn emit_error( &mut self, error: &Error ) {
    eprintln!( "[line {}] Error{}: {}", error.line, error.loc, error.msg );
    self.had_error = true;
  }

}