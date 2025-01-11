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
  pub fn get_rsolns( &self ) -> &HashMap<StringKey, i32> {
    &self.rsolns
  }
  pub fn get_line( &self ) -> i32 {
    self.line
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
    if let Some( &entry ) = self.rsolns.get( &symbol_key ) {
      entry == 0
    }
    else {
      false
    }
  }
}

pub type ScopeTree = Tree<Scope>;

// finalises the scope tree
pub struct ScopeTreeResolver { }

impl ScopeTreeResolver {
  pub fn new() -> ScopeTreeResolver {
    ScopeTreeResolver { }
  }
}

impl TreeMutVisitor<Scope, (), String> for ScopeTreeResolver {
  fn map_mut( &self, tree: &mut Tree<Scope>, node_key: u64, _depth: u32 ) -> Result<(), String> {

    // go back through this node's ancestry, looking for symbols defined therein
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

  fn fold_mut( &self, _parent_result: &(), _children_results: &Vec<()> ) -> () { }
}