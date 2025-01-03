// private module rlox::util

use std::{collections::HashMap, hash::{DefaultHasher, Hash, Hasher}};

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

pub struct StringManager {
  db: HashMap<StringKey, String>
}

impl StringManager {

  pub fn new() -> StringManager {
    StringManager{
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