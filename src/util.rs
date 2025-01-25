///////////////////////////////
// private module rlox::util //
///////////////////////////////


/////////
// use //
/////////

use std::{cell::{Ref, RefCell, RefMut}, collections::HashMap, hash::{DefaultHasher, Hash, Hasher}, rc::Rc};


///////////////////
// miscellaneous //
///////////////////

pub fn assert( condition: bool, msg: String ) {
  if !condition {
    eprintln!( "{}", msg );
    panic!();
  }
}

pub fn substring<'a>( s: &'a str, start: usize, len: usize ) -> Option<&'a str> {
  if start < s.len() && ( start + len - 1 ) < s.len() {
      Some( &s[ start .. ( start + len ) ] )
  }
  else {
      None
  }
}

pub fn char_at( s: &str, idx: usize ) -> Option<char> {
  for x in s.char_indices().filter( | ( i, _ ) | *i == idx ).take( 1 ) {
      return Some( x.1 );
  }
  None
}

pub fn ifte<T>( condition: bool, true_val: T, false_val: T ) -> T {
  if condition {
      true_val
  }
  else {
      false_val
  }
}

pub fn is_digit( c: char ) -> bool {
  c >= '0' && c <= '9'
}

pub fn is_alpha( c: char ) -> bool {
  ( c >= 'a' && c <= 'z' ) ||
  ( c >= 'A' && c <= 'Z' ) ||
  c == '_'
}

pub fn is_alphanumeric( c: char ) -> bool {
  is_alpha( c ) || is_digit( c )
}

fn default_hash( s: &str ) -> u64 {
  let mut hasher = DefaultHasher::new();
  s.hash( &mut hasher );
  hasher.finish()
}


/////////////////
// StringCache //
/////////////////

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub struct StringKey {
  key: u64
}

impl StringKey {
  pub fn new( s: &str ) -> StringKey {
    StringKey {
      key: default_hash( s )
    }
  }
}

#[derive(Clone)]
pub struct StringCache {
  db: HashMap<StringKey, String>
}

impl StringCache {
  pub fn new() -> StringCache {
    StringCache{
      db: HashMap::new()
    }
  }
  pub fn puts( &mut self, s: &str ) -> StringKey {
    let key = StringKey::new( s );
    if !self.db.contains_key( &key ) {
      self.db.insert( key, String::from( s ) );
    }
    key
  }
  pub fn gets( &self, key: StringKey ) -> &String {
    self.db.get( &key ).expect( "Unknown key. The caller of gets() assumes responsibility for checking that the key exists." )
  }
}


///////////////
// Stream<T> //
///////////////

pub struct Stream<T> {
  vec: Vec<T>,
  pos: usize
}

impl<T> Stream<T> {
  pub fn new( data: Vec<T> ) -> Stream<T> {
    Stream {
      vec: data,
      pos: 0
    }
  }
  fn pos_is_valid( &self, pos: usize ) -> bool {
    pos < self.vec.len()
  }
  fn is_eos( &self ) -> bool {
    self.pos >= self.vec.len()
  }
  fn get_pos( &self, offset: i32 ) -> usize {
    if offset < 0 {
      self.pos - ( -offset as usize )
    }
    else {
      self.pos + ( offset as usize )
    }
  }
  fn offset_is_valid( &self, offset: i32 ) -> bool {
    self.pos_is_valid( self.get_pos( offset ) )
  }
  fn assert_valid( &self, offset: i32 ) {
    if !self.offset_is_valid( offset ) {
      panic!( "Requested stream position is invalid. Did you check the stream existed at this location?" );
    } 
  }
  fn get( &self, offset: i32 ) -> &T {
    self.assert_valid( offset );
    self.vec.get( self.get_pos( offset ) ).unwrap()
  }
  pub fn has_prev( &self ) -> bool {
    self.offset_is_valid( -1 )
  }
  pub fn has_curr( &self ) -> bool {
    self.offset_is_valid( 0 )
  }
  pub fn has_next( &self ) -> bool {
    self.offset_is_valid( 1 )
  }
  pub fn prev( &self ) -> &T {
    self.get( -1 )
  }
  pub fn curr( &self ) -> &T {
    self.get( 0 )
  }
  pub fn next( &self ) -> &T {
    self.get( 1 )
  }
  pub fn adv( &mut self ) {
    if self.is_eos() {
      return;
    }
    self.pos +=1;
  }
  pub fn rev( &mut self ) {
    if self.pos == 0 {
      return;
    }
    self.pos -= 1;
  }
}


//////////////
// Stack<T> //
//////////////

#[derive(Clone, Debug)]
pub struct Stack<T> {
  vec: Vec<T>
}

impl<T> Stack<T> {
  pub fn new() -> Stack<T> {
    Stack { vec: Vec::new() }
  }
  pub fn clear( &mut self ) {
    self.vec.clear();
  }
  pub fn is_empty( &self ) -> bool {
    self.vec.is_empty()
  }
  pub fn iter( &self ) -> std::slice::Iter<T> {
    self.vec.iter()
  }
  fn assert_non_empty( &self ) {
    if self.is_empty() {
      panic!( "Stack is empty, so cannot pop() or peek()." );
    }
  }
  pub fn depth( &self ) -> usize {
    self.vec.len()
  }
  pub fn peek( &self, depth: usize ) -> &T {
    self.assert_non_empty();
    assert!( depth < self.depth() );
    self.vec.get( self.vec.len() - 1 - depth ).unwrap()
  }
  pub fn peek_mut( &mut self, depth: usize ) -> &mut T {
    self.assert_non_empty();
    assert!( depth < self.depth() );
    let idx = self.vec.len() - 1 - depth;
    self.vec.get_mut( idx ).unwrap()
  }
  pub fn pop( &mut self ) -> T {
    self.assert_non_empty();
    self.vec.pop().unwrap()
  }
  pub fn push( &mut self, value: T ) {
    self.vec.push( value );
  }
}


//////////////
// RcMut<T> //
//////////////

#[derive(Debug, Clone)]
pub struct RcMut<T> {
  shared_ptr: Rc<RefCell<T>>
}

impl<T> RcMut<T> {
  pub fn new( t: T ) -> RcMut<T> {
    RcMut {
      shared_ptr: Rc::new( RefCell::new( t ) )
    }
  }
  pub fn view( &self ) -> Ref<T> {
    self.shared_ptr.as_ref().borrow()
  }
  pub fn view_mut( &mut self ) -> RefMut<T> {
    self.shared_ptr.as_ref().borrow_mut()
  }
}


/////////////
// Tree<N> //
/////////////

#[derive(Clone)]
pub struct Tree<N> {
  nodes: HashMap<u64, N>,
  children: HashMap<u64, Vec<u64>>,
  key: u64
}

pub trait TreeFolder<N, T, E> {
  fn map( &self, db: &Tree<N>, node_key: u64, depth: u32 ) -> Result<T, E>;
  fn fold( &self, parent_result: &T, children_results: &Vec<T> ) -> T;
}

pub trait TreeFolderTgt<N, T, E> {
  fn map_fold<V: TreeFolder<N, T, E>>( &self, visitor: &V, node_key: u64, depth: u32 ) -> Result<T, E>;
}

pub trait TreeVisitor<N, E> {
  fn visit( &mut self, db: &Tree<N>, node_key: u64, depth: u32 ) -> Result<(), E>;
  fn before_children( &mut self, db: &Tree<N>, node_key: u64, depth: u32 );
  fn after_children( &mut self, db: &Tree<N>, node_key: u64, depth: u32 );
}

pub trait TreeVisitorTgt<N, E> {
  fn accept<V: TreeVisitor<N, E>>( &self, visitor: &mut V, node_key: u64, depth: u32 ) -> Result<(), E>;
}

pub trait TreeMutVisitor<N, E> {
  fn visit( &mut self, db: &mut Tree<N>, node_key: u64, depth: u32 ) -> Result<(), E>;
  fn before_children( &mut self, db: &mut Tree<N>, node_key: u64, depth: u32 );
  fn after_children( &mut self, db: &mut Tree<N>, node_key: u64, depth: u32 );
}

pub trait TreeMutVisitorTgt<N, E> {
  fn accept_mut<V: TreeMutVisitor<N, E>>( &mut self, visitor: &mut V, node_key: u64, depth: u32 ) -> Result<(), E>;
}

impl<N> Tree<N> {
  pub fn new( root: N ) -> Tree<N> {
    let mut new_tree = Tree {
      nodes: HashMap::new(),
      children: HashMap::new(),
      key: 0
    };
    new_tree.add_root_node( root );
    new_tree
  }
  fn use_key( &mut self ) -> u64 {
    let key = self.key;
    self.key += 1;
    key
  }
  pub fn has_node( &self, node_key: u64 ) -> bool {
    node_key < self.key
  }
  pub fn has_children( &self, parent_node: u64 ) -> bool {
    self.children.contains_key( &parent_node )
  }
  fn add_root_node( &mut self, root: N ) {
    let root_key = self.use_key();
    assert!( root_key == 0 );
    self.nodes.insert( root_key, root );
  }
  pub fn add_node( &mut self, parent_key: u64, node: N ) -> u64 {

    // add child to self.nodes
    let child_key = self.use_key();
    self.nodes.insert( child_key, node );

    // update parent's list of children
    if self.children.contains_key( &parent_key ) {
      self.children.get_mut( &parent_key ).unwrap().push( child_key );
    } else {
      self.children.insert( parent_key, vec![ child_key ] );
    }
    
    // return child key
    child_key
  }
  pub fn add_node_with_key( &mut self, parent_key: u64, node: N, child_key: u64 ) -> u64 {

    // add child to self.nodes
    self.nodes.insert( child_key, node );

    // update parent's list of children
    if self.children.contains_key( &parent_key ) {
      self.children.get_mut( &parent_key ).unwrap().push( child_key );
    } else {
      self.children.insert( parent_key, vec![ child_key ] );
    }
    
    // return child key
    child_key
  }
  pub fn is_parent_of( &self, parent_key: u64, child_key: u64 ) -> bool {
    match self.children.get( &parent_key ) {
        Some( child_keys ) => child_keys.contains( &child_key ),
        None => false
    }
  }
  pub fn get_parent_key( &self, child_key: u64 ) -> u64 {
    for &node_key in self.nodes.keys() {
      if self.is_parent_of( node_key, child_key ) {
        return node_key;
      }
    }
    panic!( "Node has no parent. The caller of get_parent_key() assumes responsibility for checking that a parent exists." );
  }
  pub fn get_children( &self, parent_key: u64 ) -> &Vec<u64> {
    self.children.get( &parent_key )
      .expect( "Node has no children. The caller of get_children() assumes responsibility for checking that the node has children." )
  }
  pub fn read_node( &self, node_key: u64 ) -> &N {
    self.nodes.get( &node_key )
      .expect( "Node not found. The caller of read_node() assumes responsibility for checking that the node exists." )
  }
  pub fn write_node( &mut self, node_key: u64 ) -> &mut N {
    self.nodes.get_mut( &node_key )
      .expect( "Node not found. The caller of write_node() assumes responsibility for checking that the node exists." )
  }
}

impl<N, T, E> TreeFolderTgt<N, T, E> for Tree<N> {
  fn map_fold<V: TreeFolder<N, T, E>>( &self, visitor: &V, node_key: u64, depth: u32 ) -> Result<T, E> {
    let node_result = visitor.map( self, node_key, depth )?;
    let mut child_results: Vec<T> = Vec::new();
    if self.has_children( node_key ) {
      for child_key in self.get_children( node_key ) {
        child_results.push( self.map_fold( visitor, *child_key, depth + 1 )? );
      }
    }
    Ok( visitor.fold( &node_result, &child_results ) )
  }
}

impl<N, E> TreeVisitorTgt<N, E> for Tree<N> {
  fn accept<V: TreeVisitor<N, E>>( &self, visitor: &mut V, node_key: u64, depth: u32 ) -> Result<(), E> {
    visitor.visit( self, node_key, depth )?;
    visitor.before_children( self, node_key, depth );
    if self.has_children( node_key ) {
      for child_key in self.get_children( node_key ).clone() {
        self.accept( visitor, child_key, depth + 1 )?;
      }
    }
    visitor.after_children( self, node_key, depth );
    Ok( () )
  }
}

impl<N, E> TreeMutVisitorTgt<N, E> for Tree<N> {
  fn accept_mut<V: TreeMutVisitor<N, E>>( &mut self, visitor: &mut V, node_key: u64, depth: u32 ) -> Result<(), E> {
    visitor.visit( self, node_key, depth )?;
    visitor.before_children( self, node_key, depth );
    if self.has_children( node_key ) {
      for child_key in self.get_children( node_key ).clone() {
        self.accept_mut( visitor, child_key, depth + 1 )?;
      }
    }
    visitor.after_children( self, node_key, depth );
    Ok( () )
  }
}