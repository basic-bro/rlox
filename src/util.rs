// private module rlox::util

use std::{cell::{Ref, RefCell, RefMut}, collections::HashMap, hash::{DefaultHasher, Hash, Hasher}, rc::Rc};

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

pub struct Stack<T> {
  vec: Vec<T>
}

impl<T> Stack<T> {

  pub fn new() -> Stack<T> {
    Stack { vec: vec![] }
  }

  pub fn is_empty( &self ) -> bool {
    self.vec.is_empty()
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

#[derive(Clone)]
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