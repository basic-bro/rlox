// private module rlox::util

pub fn substring<'a>( s: &'a str, start: usize, len: usize ) -> Option<&'a str> {
  if start < s.len() && ( start + len - 1 ) < s.len() {
      Some( &s[ start .. ( start + len ) ] )
  }
  else {
      None
  }
}

pub fn char_at( s: &str, idx: usize ) -> Option<char> {
  for x in s.char_indices().filter( | ( i, _c ) | *i == idx ).take( 1 ) {
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