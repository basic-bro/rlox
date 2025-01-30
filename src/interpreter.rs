/////////////////////////////////////
// public module rlox::interpreter //
/////////////////////////////////////


/////////
// use //
/////////

use crate::{env::Env, error::Error, eval::Eval, expr::{self, Expr},
stmt::{self, Stmt}, token::TokenType, util::{assert, RcMut}};


//////////////////
// declarations //
//////////////////

pub struct Interpreter {
  envs: RcMut<Env>,
  had_error: bool
}

pub enum EvalError {
  Error( Error ),
  Return( Eval )
}


/////////////////////
// implementations //
/////////////////////

impl Interpreter {
  pub fn new() -> Interpreter {
    Interpreter {
      envs: Env::create_global(),
      had_error: false
    }
  }
  pub fn restart( &mut self ) {
    self.envs = Env::create_global();
    self.had_error = false;
  }
  pub fn interpret( &mut self, stmts: &Vec<Stmt> ) -> ( Eval, bool ) {
    self.restart();
    let mut result = Eval::Nil;
    for stmt in stmts {
      match self.interpret_stmt( stmt ) {
        Ok( eval ) => result = eval,
        Err( EvalError::Error( e ) ) => {
          self.emit_error( &e );
          return ( result, self.had_error )
        },
        Err( EvalError::Return( _ ) ) => {
          panic!( "Internal error: Return values shouldn't make it here." );
        }
      }
    }
    ( result, self.had_error )
  }
  fn emit_error( &mut self, error: &Error ) {
    eprintln!( "[line {}] Error{}: {}", error.line, error.loc, error.msg );
    self.had_error = true;
  }
  fn interpret_expr( &mut self, expr: &Expr ) -> Result<Eval, EvalError> {
    expr.accept( self )
  }
  fn interpret_stmt( &mut self, stmt: &Stmt ) -> Result<Eval, EvalError> {
    stmt.accept( self )
  }
}

impl expr::Visitor<Result<Eval, EvalError>> for Interpreter {
  fn visit_assign_expr( &mut self, assign: &expr::Assign ) -> Result<Eval, EvalError> {
    let result = self.interpret_expr( &assign.rhs )?;
    self.envs.view_mut().write_symbol_at(
      &assign.lhs.name, assign.lhs.jump as usize, &result );
    Ok( result )
  }
  fn visit_binary_expr( &mut self, binary: &expr::Binary ) -> Result<Eval, EvalError> {
    let left = self.interpret_expr( &binary.left )?;
    let right = self.interpret_expr( &binary.right )?;
    let op = &binary.operator;
    let op_t = binary.operator.token_type;
    match op_t {

      // first, evaluate any logical operator
      // [ these involve casting to bool => .is_truthy() ]
      TokenType::And => Ok( Eval::Bool( left.is_truthy() && right.is_truthy() ) ),
      TokenType::Or => Ok( Eval::Bool( left.is_truthy() || right.is_truthy() ) ),

      // then, treat according to operand types
      // [ no type conversions required ]
      _ =>  match ( &left, &right ) {

          // binary operations on Numbers
          ( Eval::Number( x ), Eval::Number( y ) )
            =>  match op_t {

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
                    "Unknown binary operation on type Number.".to_string() ) ) )
                },
          
          // binary operations on StringLiterals
          ( Eval::StringLiteral( x ), Eval::StringLiteral( y ) )
            =>  match op_t {

                  // concatenation
                  TokenType::Plus => Ok( Eval::StringLiteral( x.to_owned() + y ) ),

                  // error
                  _ => Err( EvalError::Error( Error::from_token( op,
                    "Unknown binary operation on type String.".to_string() ) ) )
                },
          
          // binary operations on Bools
          ( Eval::Bool( x ), Eval::Bool( y ) )
            =>  match op_t {

                  // equality
                  TokenType::EqualEqual => Ok( Eval::Bool( x == y ) ),
                  TokenType::BangEqual  => Ok( Eval::Bool( x != y ) ),

                  // error
                  _ => Err( EvalError::Error( Error::from_token( op,
                    "Unknown binary operation on type Bool.".to_string() ) ) )
            },

          // binary operation on Nils
          ( Eval::Nil, Eval::Nil )
            =>  match op_t {

                // equality
                TokenType::EqualEqual => Ok( Eval::Bool( true ) ),
                TokenType::BangEqual  => Ok( Eval::Bool( false ) ),

                // error
                _ => Err( EvalError::Error( Error::from_token( op,
                  "Unknown binary operation on type Nil.".to_string() ) ) )
            }

          // error
          _ => Err( EvalError::Error( Error::from_token( op,
            format!(
              "Unknown binary operation on the types provided. (The types are {} and {}, respectively.)",
              left.get_type_name(), right.get_type_name() ) ) ) )
        }
    }
  }
  fn visit_call_expr( &mut self, call: &expr::Call ) -> Result<Eval, EvalError> {

    let callee = self.interpret_expr( &call.callee )?;

    // if working correctly, callee will be an Eval::Fun
    // from which we can invoke the function call.
    if let Eval::Fun( f, closure ) = callee {
  
      // check arity
      if f.params.len() != call.arguments.len() {
        return Err( EvalError::Error( Error::from_token( &call.paren,
          format!( "Expected {} arguments to function call, but found {}.", f.params.len(),
            call.arguments.len() ) ) ) );
      }

      // prepare function scope
      let mut args: Vec<Eval> = Vec::new();
      for arg in &call.arguments {
        args.push( self.interpret_expr( arg )? );
      }
      let callsite_envs = self.envs.clone();
      self.envs = Env::new_with_enclosing( &closure );
      for ( param, arg ) in std::iter::zip( f.params, args ) {
        self.envs.view_mut().create_symbol( &param, &arg );
      }

      // execute
      let mut exec_result = Eval::Nil;
      for stmt in &f.body {
        match self.interpret_stmt( stmt ) {
          Ok( result ) => {
            exec_result = result;
          },
          Err( EvalError::Return( retval ) ) => {
            self.envs = Env::drop_enclosed( &self.envs );
            self.envs = callsite_envs;
            return Ok( retval );
          },
          actual_error => {
            self.envs = Env::drop_enclosed( &self.envs );
            self.envs = callsite_envs;
            return actual_error;
          }
        }
      };
      self.envs = Env::drop_enclosed( &self.envs );
      self.envs = callsite_envs;
      Ok( exec_result )
    } else {
      Err( EvalError::Error( Error::from_token( &call.paren,
        format!( "Cannot call a {}.", callee.get_type_name() ) ) ) )
    }
  }
  fn visit_grouping_expr( &mut self, grouping: &expr::Grouping ) -> Result<Eval, EvalError> {
    self.interpret_expr( &grouping.expression )
  }
  fn visit_literal_expr( &mut self, literal: &expr::Literal ) -> Result<Eval, EvalError> {
    match literal.value.token_type {
      TokenType::Number => Ok( Eval::Number( literal.value.lexeme.parse::<f64>().unwrap() ) ),
      TokenType::String => Ok( Eval::StringLiteral( literal.value.lexeme.clone() ) ),
      TokenType::True => Ok( Eval::Bool( true ) ),
      TokenType::False => Ok( Eval::Bool( false ) ),
      TokenType::Nil => Ok( Eval::Nil ),
      _ => unreachable!( "Internal error: No other token types can be converted to Eval." )
    }
  }
  // fn visit_logical_expr( &mut self, logical: &expr::Logical ) -> Result<Eval, EvalError> {
  //   todo!()
  // }
  fn visit_unary_expr( &mut self, unary: &expr::Unary ) -> Result<Eval, EvalError> {
    let right = self.interpret_expr( &unary.right )?;
    match unary.operator.token_type {
      TokenType::Bang => Ok( Eval::Bool( !right.is_truthy() ) ),
      TokenType::Minus => match right {
        Eval::Number( x ) => Ok( Eval::Number( -x ) ),
        _ => Err( EvalError::Error( Error::from_token( &unary.operator,
          format!( "Unary '-' cannot be applied to a value of type {}.", right.get_type_name() ) ) ) )
      },
      _ => Err( EvalError::Error( Error::from_token( &unary.operator,
        "Internal error: evaluation of this unary operator is not implemented.".into() ) ) )
    }
  }
  fn visit_variable_expr( &mut self, variable: &expr::Variable ) -> Result<Eval, EvalError> {
    assert( variable.jump >= 0, format!( "Variable '{}' has negative jump value: {}.",
      variable.name.lexeme, variable.jump ) );
    Ok( self.envs.view().read_symbol_at( &variable.name, variable.jump as usize ) )
  }
}

impl stmt::Visitor<Result<Eval, EvalError>> for Interpreter {
  fn visit_block_stmt( &mut self, block: &stmt::Block ) -> Result<Eval, EvalError> {
    self.envs = Env::new_with_enclosing( &self.envs );
    let mut result = Eval::Nil;
    for stmt in &block.statements {
      match self.interpret_stmt( stmt ) {
        Ok( res ) => {
          result = res;
        },
        Err( e ) => {
          self.envs = Env::drop_enclosed( &self.envs );
          return Err( e );      
        },
      }
    }
    self.envs = Env::drop_enclosed( &self.envs );
    Ok( result )
  }
  fn visit_expression_stmt( &mut self, expression: &stmt::Expression ) -> Result<Eval,EvalError> {
    self.interpret_expr( &expression.expression )
  }
  fn visit_function_stmt( &mut self, function: &stmt::Function ) -> Result<Eval, EvalError> {
    let result = Eval::Fun( function.clone(), self.envs.clone() );
    self.envs.view_mut().create_symbol( &function.name, &result );
    Ok( result )
  }
  fn visit_if_stmt( &mut self, if_: &stmt::If ) -> Result<Eval, EvalError> {
    if self.interpret_expr( &if_.condition )?.is_truthy() {
      self.interpret_stmt( &if_.then_branch )
    } else if if_.else_branch.is_some() {
      self.interpret_stmt( if_.else_branch.as_ref().unwrap() )
    } else {
      Ok( Eval::Nil )
    }
  }
  fn visit_print_stmt( &mut self, print: &stmt::Print ) -> Result<Eval, EvalError> {
    let result = self.interpret_expr( &print.expression )?;
    println!( "{}", result );
    Ok( result )
  }
  fn visit_return_stmt( &mut self, return_: &stmt::Return ) -> Result<Eval, EvalError> {
    if let Some( expr ) = &return_.value  {
      Err( EvalError::Return( self.interpret_expr( &expr )? ) )
    } else {
      Err( EvalError::Return( Eval::Nil ) )
    }
  }
  fn visit_var_stmt( &mut self, var: &stmt::Var ) -> Result<Eval, EvalError> {
    let value = match var.init.as_ref() {
      Some( expr ) => self.interpret_expr( expr )?,
      None => Eval::Nil
    };
    self.envs.view_mut().create_symbol( &var.name, &value );
    Ok( value )
  }
  fn visit_while_stmt( &mut self, while_: &stmt::While ) -> Result<Eval, EvalError> {
    let mut result = Eval::Nil;
    loop {
      if !self.interpret_expr( &while_.condition )?.is_truthy() {
        return Ok( result );
      }
      result = self.interpret_stmt( &while_.body )?;
    }
  }
}