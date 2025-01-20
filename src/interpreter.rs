/////////////////////////////////////
// public module rlox::interpreter //
/////////////////////////////////////


/////////
// use //
/////////

use std::collections::HashMap;

use crate::{env::Env, error::Error, eval::Eval, expr::{self, Expr}, token::Token, util::RcMut};


//////////////////
// declarations //
//////////////////

pub struct Interpreter {
  globals: RcMut<Env>,
  env: RcMut<Env>,
  locals: HashMap<Expr, usize>
}


/////////////////////
// implementations //
/////////////////////

impl Interpreter {
  pub fn new() -> Interpreter {
    let globals = RcMut::new( Env::new() );
    Interpreter {
      globals: globals.clone(),
      env: globals.clone(),
      locals: HashMap::new()
    }
  }
  pub fn add_local( &mut self, expr: Expr, depth: usize ) {
    self.locals.insert( expr, depth );
  }
  pub fn read_symbol( &self, name: Token, expr: &Expr ) -> Eval {
    if let Some( &depth ) = self.locals.get( expr ) {
      Env::read_symbol_at( &self.env, name, depth )
    } else {
      self.globals.view().read_symbol( name )
    }
  }
}

// impl expr::Visitor<Result<Eval, Error>> for Interpreter {
//     fn visit_assign_expr( &mut self, assign: &expr::Assign ) -> Result<Eval, Error> {
      
//     }

//     fn visit_binary_expr( &mut self, binary: &expr::Binary ) -> Result<Eval, Error> {
//         todo!()
//     }

//     fn visit_call_expr( &mut self, call: &expr::Call ) -> Result<Eval, Error> {
//         todo!()
//     }

//     fn visit_grouping_expr( &mut self, grouping: &expr::Grouping ) -> Result<Eval, Error> {
//         todo!()
//     }

//     fn visit_literal_expr( &mut self, literal: &expr::Literal ) -> Result<Eval, Error> {
//         todo!()
//     }

//     fn visit_logical_expr( &mut self, logical: &expr::Logical ) -> Result<Eval, Error> {
//         todo!()
//     }

//     fn visit_unary_expr( &mut self, unary: &expr::Unary ) -> Result<Eval, Error> {
//         todo!()
//     }

//     fn visit_variable_expr( &mut self, variable: &expr::Variable ) -> Result<Eval, Error> {
//       Ok( self.read_symbol( variable.name.clone(), &Expr::Variable( variable.clone() ) ) )
//     }
// }