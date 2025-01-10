/////////////////////////////////////////////
// private module rlox::interpreter::scope //
/////////////////////////////////////////////


/////////
// use //
/////////

use std::collections::HashMap;

use crate::util::*;


//////////////////////
// public interface //
//////////////////////

#[derive(Clone)]
pub struct Scope {
  rsolns: HashMap<StringKey, i32>,
  line: i32
}

impl Scope {
  pub fn new( line: i32 ) -> Scope {
    Scope {
      rsolns: HashMap::new(),
      line
    }
  }
  pub fn add_local_symbol( &mut self, symbol_key: StringKey ) {
    self.rsolns.insert( symbol_key, 0 );
  }
  pub fn add_symbol( &mut self, symbol_key: StringKey ) {
    if !self.rsolns.contains_key( &symbol_key ) {
      self.rsolns.insert( symbol_key, -1 );
    }
  }
  pub fn has_local( &self, symbol_key: StringKey ) -> bool {
    if let Some( entry ) = self.rsolns.get( &symbol_key ) {
      *entry == 0
    }
    else {
      false
    }
  }
}

pub type ScopeTree = Tree<Scope>;

pub struct ScopeTreeFormatter<'str> {
  sc: &'str StringCache
}

impl<'str> ScopeTreeFormatter<'str> {
  pub fn new( sc: &'str StringCache ) -> ScopeTreeFormatter<'str> {
    ScopeTreeFormatter {
      sc
    }
  }
}

impl<'str> TreeVisitor<Scope, String, String> for ScopeTreeFormatter<'str> {
  fn visit_node( &self, db: &Tree<Scope>, node_key: u64, depth: u32 ) -> Result<String, String> {
    let indent = " ".repeat( depth as usize );
    let mut rsolns = String::new();
    let node = db.read_node( node_key );
    for extern_ in &node.rsolns {
      rsolns.push_str( self.sc.gets( *extern_.0 ) );
      rsolns.push_str( format!( " [{}] ", *extern_.1 ).as_str() );
    }
    Ok( format!( "{}Scope {} begins on line {} and has symbols: {}", indent, node_key, node.line, rsolns ) )
  }
  fn fold( &self, parent_result: &String, children_results: &Vec<String> ) -> String {
    parent_result.to_owned() + "\n" + &children_results.join( "\n" )
  }
}

// finalises the scope tree
pub struct ScopeTreeResolver { }

impl ScopeTreeResolver {
  pub fn new() -> ScopeTreeResolver {
    ScopeTreeResolver { }
  }
}

impl TreeVisitorMut<Scope, (), String> for ScopeTreeResolver {
  fn visit_node( &self, tree: &mut Tree<Scope>, node_key: u64, _depth: u32 ) -> Result<(), String> {

    let mut parent_key_stack = tree.get_parent_key_stack( node_key );

    let mut curr_jumps = 1;
    loop {

      // exit if no more ancestors to check
      if parent_key_stack.is_empty() {
        break;
      }

      // init current parent
      let parent_key = *parent_key_stack.peek( 0 );
      let parent_scope = tree.read_node( parent_key ).clone();

      // go through symbols in my scope
      for ( &symbol_key, &mut ref mut jumps ) in tree.write_node( node_key ).rsolns.iter_mut() {

        // ignore my local symbols;
        // these have been resolved
        if *jumps == 0 {
          continue;
        }

        // check for my unresolved symbols
        // in the parent's locals
        if parent_scope.has_local( symbol_key ) {
          *jumps = curr_jumps;
        }
      }

      // init next loop
      curr_jumps += 1;
      parent_key_stack.pop();
    }
    Ok( () )
  }

  fn fold( &self, _parent_result: &(), _children_results: &Vec<()> ) -> () { }
}