////////////////////////////////////////////////
// private module rlox::interpreter::executor //
////////////////////////////////////////////////


/////////
// use //
/////////

use crate::interpreter::error::*;
use crate::interpreter::stmt::*;
use crate::interpreter::eval::*;
use crate::interpreter::decl::*;
use crate::interpreter::token::*;
use crate::interpreter::expr::*;
use crate::interpreter::runtime::*;
use crate::interpreter::visitor::*;

use crate::util::*;


//////////////////
// declarations //
//////////////////

pub struct Executor {
  sc: RcMut<StringCache>,
  rt: RcMut<Runtime>
}

pub enum EvalError {
  Error( Error ),
  Return( Eval )
}


/////////////////////
// implementations //
/////////////////////

impl Executor {
  pub fn new( sc: &RcMut<StringCache>, rt: &RcMut<Runtime> ) -> Executor {
    Executor {
      sc: sc.clone(),
      rt: rt.clone()
    }
  }
  fn env_has_symbol( &self, symbol_key: StringKey ) -> bool {
    self.rt.view().symbol_is_visible( symbol_key )
  }
  fn write_env_symbol( &mut self, symbol_key: StringKey, value: Eval ) {
    assert( self.env_has_symbol( symbol_key ),
      format!( "Internal error: Unknown symbol '{}'", self.sc.view().gets( symbol_key ) ) );
    self.rt.view_mut().write_symbol( symbol_key, value );
  }
  fn push_env( &mut self, decls: &Vec<Decl> ){
    self.rt.view_mut().push_env( decls );
  }
  fn pop_env( &mut self ) {
    self.rt.view_mut().pop_env();
  }
  fn adv_ip( &mut self ) {
    self.rt.view_mut().adv_ip();
  }
  fn rev_ip( &mut self ) {
    self.rt.view_mut().rev_ip();
  }
  fn exec_if_stmt(
    &mut self,
    init: &Option<CtrlFlowInit>,
    condition: &Expr,
    then: &Box<Stmt>,
    else_: &Option<Box<Stmt>> ) -> Result<Eval, EvalError> {

    self.push_env( &Vec::new() );
    if let Some( flow_init ) = init.as_ref() {
      match flow_init {
        CtrlFlowInit::VarDecl( var_decl )
          => var_decl.map_fold_decl( self )?,
        CtrlFlowInit::ExprStmt( expr_stmt )
        => expr_stmt.map_fold_stmt( self )?
      };
    }
    let result = if condition.map_fold_expr( self )?.is_truthy() {
      then.map_fold_stmt( self )?
    } else if else_.is_some() {
      else_.as_ref().unwrap().map_fold_stmt( self )?
    }
    else {
      Eval::Bool( false )
    };
    self.pop_env();
    self.adv_ip();
    Ok( result )
  }
  fn exec_while_stmt(
    &mut self,
    init: &Option<CtrlFlowInit>,
    condition: &Expr,
    body: &Box<Stmt> ) -> Result<Eval, EvalError> {

    self.push_env( &Vec::new() );
    let mut result = if let Some( flow_init ) = init.as_ref() {
      match flow_init {
        CtrlFlowInit::VarDecl( var_decl )
          => var_decl.map_fold_decl( self )?,
        CtrlFlowInit::ExprStmt( expr_stmt )
        => expr_stmt.map_fold_stmt( self )?
      }
    } else {
      Eval::Nil
    };
    while condition.map_fold_expr( self )?.is_truthy() {
      result = body.map_fold_stmt( self )?;
    }
    self.pop_env();
    self.adv_ip();
    Ok( result )
  }
  fn exec_for_stmt(
    &mut self,
    init: &Option<CtrlFlowInit>,
    condition: &Option<Expr>,
    incr: &Option<Expr>,
    body: &Box<Stmt> ) -> Result<Eval, EvalError> {
    
    self.push_env( &Vec::new() );
    if let Some( flow_init ) = init.as_ref() {
      match flow_init {
        CtrlFlowInit::VarDecl( var_decl )
          => var_decl.map_fold_decl( self )?,
        CtrlFlowInit::ExprStmt( expr_stmt )
          => expr_stmt.map_fold_stmt( self )?
      };
    }
    let mut result = Eval::Nil;
    loop {
      if let Some( expr ) = condition {
        if !expr.map_fold_expr( self )?.is_truthy() {
          break;
        }
      }
      result = body.map_fold_stmt( self )?;
      if let Stmt::Block( _, _ ) = body.as_ref() {
        self.rev_ip();
      }
      if let Some( expr ) = incr {
        result = expr.map_fold_expr( self )?;
      }
    }
    self.pop_env();
    self.adv_ip();
    Ok( result )
  }
}

impl DeclMapFolder<Eval, EvalError> for Executor {
  fn get_stmt_map_folder( &mut self ) -> &mut impl StmtMapFolder<Eval, EvalError> {
    self
  }
  fn map_decl( &mut self, decl: &Decl ) -> MapFolderState<Eval, EvalError> {
    match decl {
      Decl::Stmt( _stmt ) => MapFolderState::Incomplete,
      Decl::Fun( fun_name, param_names, body ) => {
        self.adv_ip();
        let mut param_keys: Vec<StringKey> = Vec::new();
        for param_name in param_names {
          param_keys.push( param_name.get_key() );
        }
        let result = Eval::Fun( fun_name.get_key(), param_keys, body.clone() );
        self.write_env_symbol( fun_name.get_key(), result.clone() );
        MapFolderState::Complete( Ok( result ) )
      },
      Decl::Var( _, _init ) => MapFolderState::Incomplete
    }
  }
  fn fold_decl_stmt( &mut self, stmt: Eval ) -> Result<Eval, EvalError> {
    Ok( stmt )
  }
  fn fold_decl_fun( &mut self, fun_name: &Token, _arg_names: &Vec<Token>, _body: Eval ) -> Result<Eval, EvalError> {
    Err( EvalError::Error( 
      Error::from_token( fun_name,
        "Internal error. Execution of function declaration is already handled.".to_string(), &self.sc.view())
    ))
  }
  fn fold_decl_var( &mut self, var_name: &Token, init: Option<Eval> ) -> Result<Eval, EvalError> {
    let result = match init {
      Some( value ) => value,
      None => Eval::Nil
    };
    self.write_env_symbol( var_name.get_key(), result.clone() );
    Ok( result )
  }
}

impl StmtMapFolder<Eval, EvalError> for Executor {
  fn get_expr_map_folder( &mut self ) -> &mut impl ExprMapFolder<Eval, EvalError> {
    self
  }
  fn get_decl_map_folder( &mut self ) -> &mut impl DeclMapFolder<Eval, EvalError> {
    self
  }
  fn map_stmt( &mut self, stmt: &Stmt ) -> MapFolderState<Eval, EvalError> {
    match stmt {
      Stmt::Print( _expr )
        => MapFolderState::Incomplete,
      Stmt::Expr( _expr )
        => MapFolderState::Incomplete,
      Stmt::Block( decls, _line ) => {
        self.push_env( decls );
        MapFolderState::Incomplete
      },
      Stmt::If( init, condition , then, else_)
        => MapFolderState::Complete( self.exec_if_stmt( init, condition, then, else_ ) ),
      Stmt::While( init, condition , loop_ )
        => MapFolderState::Complete( self.exec_while_stmt( init, condition, loop_ ) ),
      Stmt::For( init, condition , incr, body )
        => MapFolderState::Complete( self.exec_for_stmt( init, condition, incr, body ) ),
      Stmt::Return( _expr )
        => MapFolderState::Incomplete,
    }
  }
  fn fold_stmt_print( &mut self, expr: Eval ) -> Result<Eval, EvalError> {
    println!( "{}", expr.to_string() );
    Ok( expr )
  }
  fn fold_stmt_expr( &mut self, expr: Eval ) -> Result<Eval, EvalError> {
    Ok( expr )
  }
  fn fold_stmt_block( &mut self, decls: Vec<Eval>, _line: &i32 ) -> Result<Eval, EvalError> {
    self.pop_env();
    self.adv_ip();
    if decls.is_empty() {
      Ok( Eval::Nil )
    } else {
      Ok( decls.last().unwrap().clone() )
    }
  }
  fn fold_stmt_if( &mut self, _init: Option<Eval>, _condition: Eval, _then: Eval, _else_: Option<Eval> )
    -> Result<Eval, EvalError> {
      panic!( "Internal error. Execution of if-statement is already handled." );
  }
  fn fold_stmt_while( &mut self, _init: Option<Eval>, _condition: Eval, _body: Eval )
    -> Result<Eval, EvalError> {
      panic!( "Internal error. Execution of while-statement is already handled." );
  }
  fn fold_stmt_for( &mut self, _init: Option<Eval>, _condition: Option<Eval>, _incr: Option<Eval>, _body: Eval )
    -> Result<Eval, EvalError> {
      panic!( "Internal error. Execution of for-statement is already handled." );
  }
  fn fold_stmt_return( &mut self, expr: Option<Eval> ) -> Result<Eval, EvalError> {
    match expr {
      Some( retval ) => Err( EvalError::Return( retval ) ),
      None => Err( EvalError::Return( Eval::Nil ) ),
    }
  }
}

impl ExprMapFolder<Eval, EvalError> for Executor {
  fn map_expr( &mut self, _expr: &Expr ) -> MapFolderState<Eval, EvalError> {
    MapFolderState::Incomplete
  }
  fn fold_expr_assignment( &mut self, var_name: &Token, right: Eval ) -> Result<Eval, EvalError> {
    self.write_env_symbol( var_name.get_key(), right.clone() );
    Ok( right )
  }
  fn fold_expr_binary( &mut self, left: Eval, op: &Token, right: Eval ) -> Result<Eval, EvalError> {
    match op.get_type() {

      // first, evaluate any logical operator
      // [ these involve casting to bool => .is_truthy() ]
      TokenType::And => Ok( Eval::Bool( left.is_truthy() && right.is_truthy() ) ),
      TokenType::Or => Ok( Eval::Bool( left.is_truthy() || right.is_truthy() ) ),

      // then, treat according to operand types
      // [ no type conversions required ]
      _ =>  match ( &left, &right ) {

          // binary operations on Numbers
          ( Eval::Number( x ), Eval::Number( y ) )
            =>  match op.get_type() {

                  // equality
                  TokenType::EqualEqual => Ok( Eval::Bool( x == y ) ),
                  TokenType::BangEqual  => Ok( Eval::Bool( x != y ) ),

                  // comparison
                  TokenType::Greater      => Ok( Eval::Bool( x > y ) ),
                  TokenType::GreaterEqual => Ok( Eval::Bool( x >= y ) ),
                  TokenType::Less         => Ok( Eval::Bool( x < y ) ),
                  TokenType::LessEqual    => Ok( Eval::Bool( x <= y ) ),

                  // term
                  TokenType::Plus  => Ok( Eval::Number( x + y ) ),
                  TokenType::Minus => Ok( Eval::Number( x - y ) ),

                  // factor
                  TokenType::Star  => Ok( Eval::Number( x * y ) ),
                  TokenType::Slash => Ok( Eval::Number( x / y ) ),
                  
                  // error 
                  _ => Err( EvalError::Error( Error::from_token( op,
                    "Unknown binary operation on type Number.".to_string(), &self.sc.view() ) ) )
                },
          
          // binary operations on StringLiterals
          ( Eval::StringLiteral( x ), Eval::StringLiteral( y ) )
            =>  match op.get_type() {

                  // concatenation
                  TokenType::Plus => Ok( Eval::StringLiteral( x.to_owned() + y ) ),

                  // error
                  _ => Err( EvalError::Error( Error::from_token( op,
                    "Unknown binary operation on type String.".to_string(), &self.sc.view() ) ) )
                },
          
          // binary operations on Bools
          ( Eval::Bool( x ), Eval::Bool( y ) )
            =>  match op.get_type() {

                  // equality
                  TokenType::EqualEqual => Ok( Eval::Bool( x == y ) ),
                  TokenType::BangEqual  => Ok( Eval::Bool( x != y ) ),

                  // error
                  _ => Err( EvalError::Error( Error::from_token( op,
                    "Unknown binary operation on type Bool.".to_string(), &self.sc.view() ) ) )
            },

          // binary operation on Nils
          ( Eval::Nil, Eval::Nil )
            =>  match op.get_type() {

                // equality
                TokenType::EqualEqual => Ok( Eval::Bool( true ) ),
                TokenType::BangEqual  => Ok( Eval::Bool( false ) ),

                // error
                _ => Err( EvalError::Error( Error::from_token( op,
                  "Unknown binary operation on type Nil.".to_string(), &self.sc.view() ) ) )
            }

          // error
          _ => Err( EvalError::Error( Error::from_token( op,
            format!( "Unknown binary operation on the types provided. (The types are {} and {}, respectively.)",
              left.get_type_name(), right.get_type_name() ), &self.sc.view() ) ) )
        }
    }
  }
  fn fold_expr_call( &mut self, callee: Eval, paren: &Token, args: &Vec<Eval> ) -> Result<Eval, EvalError> {

    // if working correctly, callee will be an Eval::Fun
    // from which we can invoke the function call.
    if let Eval::Fun( fun_name_key, param_keys, body ) = callee {

      //if let Stmt::Block( decls, line ) = body {
        
        // check arity
        if param_keys.len() != args.len() {
          return Err( EvalError::Error( Error::from_token( paren,
            format!( "Expected {} arguments to function call, but found {}.", param_keys.len(), args.len() ), &self.sc.view() ) ) );
        }

        // prepare function scope
        self.rt.view_mut().init_fun_call( fun_name_key, args );
        let exec_result = body.map_fold_stmt( self );
        self.rt.view_mut().finish_fun_call();
        match exec_result {
          Ok( result ) => Ok( result ),
          Err( EvalError::Return( retval ) ) => Ok( retval ),
          actual_error => Err( actual_error.unwrap_err() )
        }
      //} else {
      //  panic!( "Internal error: 'body' should have type Stmt::Block, but it has type {} instead.", body.get_type_name() );  
      //}
    } else {
      Err( EvalError::Error( Error::from_token( paren, format!( "Cannot call a {}.", callee.get_type_name() ), &self.sc.view() ) ) )
    }
  }
  fn fold_expr_grouping( &mut self, expr: Eval ) -> Result<Eval, EvalError> {
    Ok( expr )
  }
  fn fold_expr_literal( &mut self, literal: &Token ) -> Result<Eval, EvalError> {
    match literal.get_type() {
      TokenType::String( s ) => Ok( Eval::StringLiteral( self.sc.view().gets( *s ).to_string() ) ),
      TokenType::Number( s ) => Ok( Eval::Number( self.sc.view().gets( *s ).parse::<f64>().unwrap() ) ),
      TokenType::True => Ok( Eval::Bool( true ) ),
      TokenType::False => Ok( Eval::Bool( false ) ),
      TokenType::Nil => Ok( Eval::Nil ),
      _ => Err( EvalError::Error( Error::from_token( literal,
        "Internal error: evaluation of this expression is not implemented.".to_string(), &self.sc.view() ) ) )
    }
  }
  fn fold_expr_unary( &mut self, op: &Token, expr: Eval ) -> Result<Eval, EvalError> {
    match op.get_type() {
      TokenType::Bang => Ok( Eval::Bool( !expr.is_truthy() ) ),
      TokenType::Minus => match expr {
        Eval::Number( x ) => Ok( Eval::Number( -x ) ),
        _ => Err( EvalError::Error( Error::from_token( op,
          format!( "Unary '-' cannot be applied to a value of type {}.", expr.get_type_name() ), &self.sc.view() ) ) )
      },
      _ => Err( EvalError::Error( Error::from_token( op,
        "Internal error: evaluation of this unary operator is not implemented.".to_string(), &self.sc.view() ) ) )
    }
  }
  fn fold_expr_symbol( &mut self, var: &Token ) -> Result<Eval, EvalError> {
    match var.get_type() {
      TokenType::Identifier( id ) => {
        if self.rt.view().symbol_is_visible( *id ) {
          Ok( self.rt.view().read_symbol( *id ) )
        } else {
          panic!( "Internal error: Undeclared variable should have been caught long before this!" );
        }
      }
      _ => Err( EvalError::Error( Error::from_token( var,
        "Internal error: evaluation of this expression is not implemented.".to_string(), &self.sc.view() ) ) )
    }
  }
}