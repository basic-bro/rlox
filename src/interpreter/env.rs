///////////////////////////////////////////
// private module rlox::interpreter::env //
///////////////////////////////////////////


/////////
// use //
/////////

use std::collections::HashMap;

use crate::util::*;

use crate::interpreter::eval::*;
use crate::interpreter::scope_tree::*;
// use crate::interpreter::format::*;


//////////////////
// declarations //
//////////////////

#[derive(Debug, Clone)]
pub struct Env {
  db: HashMap<StringKey, Eval>,
  scope_key: u64,
  child_idx: u32,
  line: i32
}

pub type EnvStack = Stack<RcMut<Env>>;

// pub type EnvTree = Tree<Env>;

// creates an EnvTree instance that matches a given ScopeTree instance
// node keys are the same
// pub struct EnvTreeBuilder {
//   env_tree: RcMut<EnvTree>
// }


/////////////////////
// implementations //
/////////////////////

// impl EnvTreeBuilder {
//   pub fn new() -> EnvTreeBuilder {
//     EnvTreeBuilder {
//       env_tree: RcMut::new( EnvTree::new( Env::new( 0, 0 ) ) )
//     }
//   }
//   pub fn get_tree( &mut self ) -> RcMut<EnvTree> {
//     self.env_tree.clone()
//   }
// }

// impl TreeVisitor<Scope, ()> for EnvTreeBuilder {
//   fn visit( &mut self, scope_tree: &Tree<Scope>, node_key: u64, _depth: u32 ) -> Result<(), ()> {

//     // add each local in the Scope to the Env
//     for ( &symbol_key, &jump ) in scope_tree.read_node( node_key ).read_jump_table() {
//       if jump == 0 {
//         self.env_tree.view_mut().write_node( node_key ).create_symbol( symbol_key, Eval::Nil );
//       }
//     }
//     Ok( () )
//   }
//   fn before_children( &mut self, scope_tree: &Tree<Scope>, node_key: u64, _depth: u32 ) {

//     // early return: no children to copy
//     if !scope_tree.has_children( node_key ) {
//       return;
//     } 

//     // create children in the EnvTree so visit() method above works
//     for &child_key in scope_tree.get_children( node_key ) {
//       let new_key = self.env_tree.view_mut().add_node_with_key( node_key, Env::new( child_key, scope_tree.read_node( child_key ).get_line() ), child_key );
//       assert!( new_key == child_key );
//     }
//   }
//   fn after_children( &mut self, _scope_tree: &Tree<Scope>, _node_key: u64, _depth: u32 ) { }
// }


impl Env {
  pub fn new( scope_key: u64, line: i32 ) -> Env {
    Env {
      db: HashMap::new(),
      scope_key,
      child_idx: 0,
      line
    }
  }
  pub fn from_scope_tree( scope_tree: &RcMut<ScopeTree>, scope_key: u64 ) -> Env {
    let mut env = Env::new( scope_key, scope_tree.view().read_node( scope_key ).get_line() );
    for ( &symbol_key, &jump ) in scope_tree.view().read_node( scope_key ).read_jump_table() {
      if jump == 0 {
        env.create_symbol( symbol_key, Eval::Nil );
      }
    }
    env
  }
  pub fn read_ip( &self ) -> ( u64, u32 ) {
    ( self.scope_key, self.child_idx )
  }
  pub fn adv_ip( &mut self ) {
    self.child_idx += 1;
  }
  pub fn rev_ip( &mut self ) {
    self.child_idx -= 1;
  }
  pub fn has_symbol( &self, key: StringKey ) -> bool {
    self.db.contains_key( &key )
  }
  pub fn read_symbol( &self, key: StringKey ) -> Eval {
    if self.has_symbol( key ) {
      self.db.get( &key ).unwrap().clone()
    }
    else {
      panic!( "Internal error: Unknown key. [ The caller of read_var() assumes responsibility for verifying that the key exists. ]" )
    }
  }
  pub fn create_symbol( &mut self, key: StringKey, value: Eval ) {
    assert!( !self.has_symbol( key ),
      "Internal error: Known/duplicate key. [ The caller of create_var() assumes responsibility for verifying that a key is unique. ]"
    );
    self.db.insert( key, value );
  }
  pub fn write_symbol( &mut self, key: StringKey, value: Eval ) {
    if self.has_symbol( key ) {
      self.db.insert( key, value );
    }
    else {
      panic!( "Internal error: Unknown key. [ The caller of write_var() assumes responsibility for verifying that the key exists. ]" )
    }
  }
  pub fn debug_format( &self, sc: &StringCache ) -> String {
    let mut fmt = format!( "Env beginning on line {} has {} entries:", self.line, self.db.len() );
    for ( key, value ) in &self.db {
      fmt.push_str( format!( " {} = {}", sc.gets( *key ), value ).as_str() );
    }
    fmt
  }
  // pub fn to_string( env_tree: EnvTree, sc: &RcMut<StringCache> ) -> String {
  //   env_tree.map_fold( &EnvTreeFormatter::new( sc ), 0, 0 ).unwrap()
  // }
  // pub fn make_env_tree( scope_tree: &RcMut<ScopeTree> ) -> RcMut<EnvTree> {
  //   let mut builder = EnvTreeBuilder::new();
  //   match scope_tree.view().accept( &mut builder, 0, 0 ) {
  //     Ok( _ ) => builder.get_tree(),
  //     Err( _ ) => panic!( "EnvTreeBuilder doesn't fail?" ),
  //   }
  // }
}