use std::{any::Any, process::exit};

use crate::{error::Error, eval::Eval, expr::{self}, stmt::{self, Stmt}, token::TokenType, util::{assert, Stack}};

#[derive(Debug, Clone)]
enum Op {
  Add,
  Sub,
  Mul,
  Div,
  Neg,
  Pop,
  PushConstant( u8 ),
  Dup( u8 ),
  Print,
  And,
  Or,
  Return,
  JumpIfFalse( i16 ),
  Jump( i16 ),
  Nop
}

pub struct ByteCode {
  code: Vec<Op>,
  constants: Vec<Value>,
}

pub struct Compiler {
  code: Vec<Op>,
  constants: Vec<Value>,
  locals: Vec<String>,
  stack_size: usize,
  had_error: bool
}

#[derive(Debug, Clone)]
enum Value {
  Number( f64 ),
  Boolean( bool ),
  Nil
}

pub struct Vm {
  bc: ByteCode,
  ip: usize,
  stack: Stack<Value>,
  had_error: bool
}

impl Value {
  pub fn is_truthy( &self ) -> bool {
    match self {
      Value::Number( _ ) => false,
      Value::Boolean( b ) => *b,
      Value::Nil => false,
    }
  }
}

impl Vm {
  pub fn new( bc: ByteCode ) -> Vm {
    Vm {
      bc,
      ip: 0,
      stack: Stack::new(),
      had_error: false
    }
  }
  pub fn exec( &mut self ) -> ( Eval, bool ) {
    let result = Eval::Nil;
    loop {
      let op = self.bc.code.get( self.ip ).unwrap().clone();
      let ( ip_offset, stop ) = self.exec_op( op.clone() );
      println!( "\n\n{:?}\n============", op );
      for i in 0..self.stack.depth() {
        println!( "{:?}", self.stack.peek( i ) );
      }
      if stop {
        // result = Eval::Number( *self.stack.peek( 0 ) );
        break;
      }
      if ip_offset < 0 {
        self.ip -= (-ip_offset) as usize;
      } else {
        self.ip += ip_offset as usize;
      }
    }

    ( result, self.had_error )
  }
  fn exec_op( &mut self, op: Op ) -> ( i16, bool ) {
    let mut ip_offset = 1i16;
    let mut stop = false;
    match op {
      Op::Add => {
        let right = self.stack.pop();
        let left = self.stack.peek( 0 ).clone();
        match ( &left, &right ) {
          ( Value::Number( x ), Value::Number( y ) ) => {
            *self.stack.peek_mut( 0 ) = Value::Number( x + y );
          },
          _ => {
            eprintln!( "\nUnknown operation '+' on types {:?} and {:?}.", left, right );
            stop = true;
          }
        }
      },
      Op::Sub => {
        let right = self.stack.pop();
        let left = self.stack.peek( 0 ).clone();
        match ( &left, &right ) {
          ( Value::Number( x ), Value::Number( y ) ) => {
            *self.stack.peek_mut( 0 ) = Value::Number( x - y );
          },
          _ => {
            eprintln!( "\nUnknown operation '-' on types {:?} and {:?}.", left, right );
            stop = true;
          }
        }
      },
      Op::Mul => {
        let right = self.stack.pop();
        let left = self.stack.peek( 0 ).clone();
        match ( &left, &right ) {
          ( Value::Number( x ), Value::Number( y ) ) => {
            *self.stack.peek_mut( 0 ) = Value::Number( x * y );
          },
          _ => {
            eprintln!( "\nUnknown operation '*' on types {:?} and {:?}.", left, right );
            stop = true;
          }
        }
      },
      Op::Div => {
        let right = self.stack.pop();
        let left = self.stack.peek( 0 ).clone();
        match ( &left, &right ) {
          ( Value::Number( x ), Value::Number( y ) ) => {
            *self.stack.peek_mut( 0 ) = Value::Number( x / y );
          },
          _ => {
            eprintln!( "\nUnknown operation '/' on types {:?} and {:?}.", left, right );
            stop = true;
          }
        }
      },
      Op::Neg => {
        let right = self.stack.peek( 0 ).clone();
        if let Value::Number( x ) = right {
          *self.stack.peek_mut( 0 ) = Value::Number( -x );
        } else {
          eprintln!( "\nUnknown operation '-' on type {:?}.", right );
          stop = true;
        }
      }
      Op::PushConstant( idx ) => {
        self.stack.push( self.bc.constants.get( idx as usize ).unwrap().clone() );
      },
      Op::And => {
        let right = self.stack.pop();
        let left = self.stack.peek( 0 ).clone();
        *self.stack.peek_mut( 0 ) = Value::Boolean( left.is_truthy() && right.is_truthy() );
      },
      Op::Or => {
        let right = self.stack.pop();
        let left = self.stack.peek( 0 ).clone();
        *self.stack.peek_mut( 0 ) = Value::Boolean( left.is_truthy() || right.is_truthy() );
      },
      Op::Return => {
        stop = true;
      },
      Op::Pop => {
        self.stack.pop();
      },
      Op::Dup( depth ) => {
        let local = self.stack.peek( depth as usize );
        self.stack.push( local.clone() );
      },
      Op::Print => {
        println!( "[Execution output: '{:?}']", self.stack.peek( 0 ) );
      },
      Op::Nop => {},
      Op::JumpIfFalse( delta ) => {
        if !self.stack.peek( 0 ).is_truthy() {
          ip_offset = delta;
        }
      },
      Op::Jump( delta ) => {
        ip_offset = delta;
      },
    }
    ( ip_offset, stop )
  }
}

impl ByteCode {
  fn new( code: Vec<Op>, constants: Vec<Value> ) -> ByteCode {
    ByteCode {
      code,
      constants
    }
  }
}

impl Compiler {
  pub fn new() -> Compiler {
    Compiler {
      code: Vec::new(),
      constants: vec![ Value::Number( 0.0 ) ],
      locals: Vec::new(),
      stack_size: 0,
      had_error: false
    }
  }
  fn restart( &mut self ) {
    self.code.clear();
    self.constants.clear();
    self.had_error = false;
  }
  fn add_constant( &mut self, constant: Value ) -> u8 {
    self.constants.push( constant );
    ( self.constants.len() - 1 ) as u8
  }
  fn emit_op( &mut self, op: Op ) -> usize {
    match op {
      Op::Add => { self.stack_size -= 1 },
      Op::Sub => { self.stack_size -= 1 },
      Op::Mul => { self.stack_size -= 1 },
      Op::Div => { self.stack_size -= 1 },
      Op::Neg => {},
      Op::Pop => { self.stack_size -= 1 },
      Op::PushConstant( _ ) => { self.stack_size += 1 },
      Op::Dup( _ ) => { self.stack_size += 1 },
      Op::Print => {},
      Op::And => { self.stack_size -= 1 },
      Op::Or => { self.stack_size -= 1 },
      Op::Return => {},
      Op::Nop => {},
      Op::JumpIfFalse( _ ) => {},
      Op::Jump( _ ) => {},
    }
    self.code.push( op );
    self.code.len() - 1
  }
  fn compile_expr( &mut self, expr: &expr::Expr ) -> Result<(), Error> {
    expr.accept( self )
  }
  fn compile_stmt( &mut self, stmt: &stmt::Stmt ) -> Result<(), Error> {
    stmt.accept( self )
  }
  pub fn compile( &mut self, stmts: &Vec<Stmt> ) -> ( ByteCode, bool ) {
    // self.restart();
    for stmt in stmts {
      match self.compile_stmt( stmt ) {
        Ok( _ ) => {},
        Err( e ) => {
          self.emit_error( &e );
          break;
        },
      }
    }
    self.emit_op( Op::Return );
    // self.debug_print();
    // exit( 0 );
    ( ByteCode::new( self.code.clone(), self.constants.clone() ), self.had_error )
  }
  fn debug_print( &self ) {
    println!( "Code:" );
    for op in &self.code {
      println!( "{:?}", op );
    }
    println!( "Constants:" );
    for ( idx, constant ) in self.constants.iter().enumerate() {
      println!( "[{}] {:?}", idx, constant );
    }
  }
  fn emit_error( &mut self, error: &Error ) {
    eprintln!( "[line {}] Error{}: {}", error.line, error.loc, error.msg );
    self.had_error = true;
  }
}

impl expr::Visitor<Result<(), Error>> for Compiler {
  fn visit_assign_expr( &mut self, assign: &expr::Assign ) -> Result<(), Error> {
    todo!()
  }

  fn visit_binary_expr( &mut self, binary: &expr::Binary ) -> Result<(), Error> {
    self.compile_expr( &binary.left )?;
    self.compile_expr( &binary.right )?;
    match binary.operator.token_type {
      TokenType::Minus => { self.emit_op( Op::Sub ); },
      TokenType::Plus  => { self.emit_op( Op::Add ); },
      TokenType::Slash => { self.emit_op( Op::Div ); },
      TokenType::Star  => { self.emit_op( Op::Mul ); },
      TokenType::And   => { self.emit_op( Op::And ); },
      TokenType::Or    => { self.emit_op( Op::Or  ); },
      
      _ => {}
    }
    Ok( () )
  }

  fn visit_call_expr( &mut self, call: &expr::Call ) -> Result<(), Error> {
    todo!()
  }

  fn visit_grouping_expr( &mut self, grouping: &expr::Grouping ) -> Result<(), Error> {
    self.compile_expr( &grouping.expression )
  }

  fn visit_literal_expr( &mut self, literal: &expr::Literal ) -> Result<(), Error> {
    if let TokenType::Number = literal.value.token_type {
      let idx = self.add_constant( Value::Number( literal.value.lexeme.parse::<f64>().unwrap() ) );
      self.emit_op( Op::PushConstant( idx ) );
      Ok( () )
    } else {
      Err( Error::from_token( &literal.value, "Only doing numbers presently.".into() ) )
    }
  }

  fn visit_unary_expr( &mut self, unary: &expr::Unary ) -> Result<(), Error> {
    self.compile_expr( &unary.right )?;
    if let TokenType::Minus = unary.operator.token_type {
      self.emit_op( Op::Neg );
      Ok( () )
    }
    else {
      Err( Error::from_token( &unary.operator,
        "Only doing unary minus on constants presently".into() ) )
    }
  }

  fn visit_variable_expr( &mut self, variable: &expr::Variable ) -> Result<(), Error> {
    let mut peek_depth: Option<u8> = None;
    for ( slot, name ) in self.locals.iter().enumerate().rev() {
      if *name == variable.name.lexeme {
        println!( "\nname = '{}', stack_size = {} slot = {}", name, self.stack_size, slot );
        peek_depth = Some( ( self.stack_size - slot - 1 ) as u8 );
        break;
      }
    }
    assert( peek_depth.is_some(), format!( "Could not find local variable '{}'", variable.name.lexeme ) );
    self.emit_op( Op::Dup( peek_depth.unwrap() ) );
    Ok( () )
  }
}

impl stmt::Visitor<Result<(), Error>> for Compiler {
  fn visit_block_stmt( &mut self, block: &stmt::Block ) -> Result<(), Error> {

    // save stack size
    let stack_size = self.stack_size;

    // run block
    for stmt in &block.statements {
      self.compile_stmt( stmt )?;
    }

    // delete variables local to the block just finished
    while self.stack_size > stack_size {
      self.emit_op( Op::Pop );
      self.locals.pop();
    }
    Ok( () )
  }

  fn visit_expression_stmt( &mut self, expression: &stmt::Expression ) -> Result<(), Error> {
    self.compile_expr( &expression.expression )?;
    self.emit_op( Op::Pop );
    Ok( () )
  }

  fn visit_function_stmt( &mut self, function: &stmt::Function ) -> Result<(), Error> {
    todo!()
  }

  fn visit_if_stmt( &mut self, if_: &stmt::If ) -> Result<(), Error> {
    
    // save stack size
    let stack_size = self.stack_size;

    // [ ... byte-code for the condition ... ]
    self.compile_expr( &if_.condition )?;

    // jump if false to #else#
    let __jump_if_false__ = self.emit_op( Op::Nop );

    // pop, [ ... byte-code for the body ... ]
    self.emit_op( Op::Pop );
    self.compile_stmt( &if_.then_branch )?;

    // jump to #done#
    let __jump_to_done__ = self.emit_op( Op::Nop );
    
    // #else#
    // pop, [ ... byte-code for the else clause ... ]
    let __else__ = self.emit_op( Op::Pop );
    self.stack_size = stack_size;  // restore stack to before the if-condition
    if let Some( stmt ) = &if_.else_branch {
      self.compile_stmt( stmt )?;
    }

    // #done#
    let __done__ = self.code.len();

    // stitch up __jump_if_false__
    *self.code.get_mut( __jump_if_false__ ).unwrap()
      = Op::JumpIfFalse( ( __else__ - __jump_if_false__ ) as i16 );

    // stitch up __jump_to_done__
    *self.code.get_mut( __jump_to_done__ ).unwrap()
      = Op::Jump( ( __done__ - __jump_to_done__ ) as i16 );
    
    Ok( () )
  }

  fn visit_print_stmt( &mut self, print: &stmt::Print ) -> Result<(), Error> {
    self.compile_expr( &print.expression )?;
    self.emit_op( Op::Print );
    self.emit_op( Op::Pop );
    Ok( () )
  }

  fn visit_return_stmt( &mut self, return_: &stmt::Return ) -> Result<(), Error> {
    todo!()
  }

  fn visit_var_stmt( &mut self, var: &stmt::Var ) -> Result<(), Error> {
    if let Some( expr ) = var.init.as_ref() {
      self.compile_expr( expr )?;
    } else {
      self.emit_op( Op::PushConstant( 0 ) );
    }
    self.locals.push( var.name.lexeme.clone() );
    Ok( () )
  }

  fn visit_while_stmt( &mut self, while_: &stmt::While ) -> Result<(), Error> {
    todo!()
  }
}