//////////////////////////////////////////////
// private module rlox::interpreter::format //
//////////////////////////////////////////////


/////////
// use //
/////////

use crate::util::*;

use crate::interpreter::expr::*;
use crate::interpreter::token::*;
use crate::interpreter::error::*;
use crate::interpreter::scope_tree::*;


//////////////////
// declarations //
//////////////////

pub struct ExprFormatter<'str> {
  sc: &'str StringCache
}

pub struct ScopeTreeFormatter<'str> {
  sc: &'str StringCache
}


/////////////////////
// implementations //
/////////////////////

impl<'str> ExprFormatter<'str> {
  pub fn new( sc: &'str StringCache ) -> ExprFormatter<'str> {
    ExprFormatter {
      sc
    }
  }
}

impl<'str> ExprFolder<String, Error> for ExprFormatter<'str> {
  fn fold_assignment( &mut self, var: &Token, right: String ) -> Result<String, Error> {
      Ok( format!( "{} = {}", var.get_lexeme( self.sc ), right ) )
  }
  fn fold_binary( &mut self, left: String, op: &Token, right: String ) -> Result<String, Error> {
    Ok( format!( "{} {} {}", left, op.get_lexeme( self.sc ), right ) )
  }
  fn fold_call( &mut self, callee: String, _paren: &Token, args: &Vec<String> ) -> Result<String, Error> {

    let no_args = args.len() == 0;

    let mut params = if no_args {
      "(".to_string()
    } else {
      "( ".to_string()
    };

    for ( idx, arg ) in args.iter().enumerate() {
      if idx == 0 {
        params.push_str( arg.as_str() );
      } else {
        params.push_str( format!( ", {}", arg ).as_str() );
      }
    }

    if no_args {
      params.push_str( ")" );
    } else {
      params.push_str( " )" );
    }

    Ok( format!( "{}{}", callee, params ) )
  }
  fn fold_grouping( &mut self, expr: String ) -> Result<String, Error> {
    Ok( format!( "( {} )", expr ) )
  }
  fn fold_literal( &mut self, literal: &Token ) -> Result<String, Error> {
    Ok(
      match literal.get_type() {
        TokenType::String( s )
          => format!( "\"{}\"", self.sc.gets( *s ) ),
        _ => format!( "{}", literal.get_lexeme( self.sc ) )
      }
    )
  }
  fn fold_unary( &mut self, op: &Token, expr: String ) -> Result<String, Error> {
    Ok( format!( "{}{}", op.get_lexeme( self.sc ), expr ) )
  }
  fn fold_symbol( &mut self, var: &Token ) -> Result<String, Error> {
    Ok( format!( "{}", var.get_lexeme( self.sc ) ) )
  }
}

impl<'str> ScopeTreeFormatter<'str> {
  pub fn new( sc: &'str StringCache ) -> ScopeTreeFormatter<'str> {
    ScopeTreeFormatter {
      sc
    }
  }
}

impl<'str> TreeFolder<Scope, String, String> for ScopeTreeFormatter<'str> {
  fn map( &self, db: &Tree<Scope>, node_key: u64, depth: u32 ) -> Result<String, String> {
    let indent = " ".repeat( depth as usize );
    let mut rsolns = String::new();
    let node = db.read_node( node_key );
    for extern_ in node.get_rsolns() {
      rsolns.push_str( self.sc.gets( *extern_.0 ) );
      rsolns.push_str( format!( " [{}] ", *extern_.1 ).as_str() );
    }
    Ok( format!( "{}Scope {} begins on line {} and has symbols: {}", indent, node_key, node.get_line(), rsolns ) )
  }
  fn fold( &self, parent_result: &String, children_results: &Vec<String> ) -> String {
    parent_result.to_owned() + "\n" + &children_results.join( "\n" )
  }
}