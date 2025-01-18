///////////////////////////////////////////////
// private module rlox::interpreter::runtime //
///////////////////////////////////////////////


/////////
// use //
/////////

use crate::util::*;

use crate::interpreter::scope_tree::*;
use crate::interpreter::env::*;
use crate::interpreter::eval::*;
use crate::interpreter::format::*;
use crate::interpreter::decl::*;


////////////////////
// struct Runtime //
////////////////////

#[derive(Clone)]
pub struct Runtime {
  sc: RcMut<StringCache>,
  scope_tree: RcMut<ScopeTree>,
  call_stack: Stack<EnvStack>
}

impl Runtime {
  pub fn new( sc: &RcMut<StringCache>, scope_tree: &RcMut<ScopeTree>, scope_key: u64, decls: &Vec<Decl> ) -> Runtime {
    let mut env_stack: EnvStack = Stack::new();
    env_stack.push( Self::init_env( scope_tree, scope_key, decls, sc ) );
    let mut call_stack: Stack<EnvStack> = Stack::new();
    call_stack.push( env_stack );
    Runtime {
      sc: sc.clone(),
      scope_tree: scope_tree.clone(),
      call_stack
    }
  }
  fn init_env( scope_tree: &RcMut<ScopeTree>, scope_key: u64, decls: &Vec<Decl>, sc: &RcMut<StringCache> ) -> RcMut<Env> {
    let mut env = RcMut::new( Env::from_scope_tree( scope_tree, scope_key ) );
    for decl in decls {
      if let Decl::Fun( fun_name, param_names, body ) = decl {
        assert( env.view().has_symbol( fun_name.get_key() ),
          format!( "Internal error: The function '{}' is declared in this scope (according to 'decls'), but does not exist in the Env.",
            sc.view().gets( fun_name.get_key() ) ) );
        let mut param_keys: Vec<StringKey> = Vec::new();
        for t in param_names {
          param_keys.push( t.get_key() );
        }
        env.view_mut().write_symbol( fun_name.get_key(), Eval::Fun( fun_name.get_key(), param_keys, body.clone() ) );           
      }
    }
    env
  }
  pub fn curr_scope( &self ) -> u64 {
    self.read_env_stack().peek( 0 ).view().read_ip().0
  }
  pub fn curr_child_scope( &self ) -> u64 {
    let child_idx = self.read_env_stack().peek( 0 ).view().read_ip().1;
    *self.scope_tree.view().get_children( self.curr_scope() )
      .get( child_idx as usize )
      .expect( "Internal error: Expected a child scope." )
  }
  pub fn read_env_stack( &self ) -> &EnvStack {
    self.call_stack.peek( 0 )
  }
  fn write_env_stack( &mut self ) -> &mut EnvStack {
    self.call_stack.peek_mut( 0 )
  }
  pub fn init_fun_call( &mut self, fun_name_key: StringKey, args: &Vec<Eval>, mut closure: EnvStack ) {

    println!( "\nInitialising function call '{}'", self.sc.view().gets( fun_name_key ) );

    // (1) get the function's definition
    let fun = closure.peek( 0 ).view().read_symbol( fun_name_key );
    if let Eval::Fun( _, param_keys, _, _ ) = fun {

      // (2) find the key to the function's parameter scope
      let fun_scope_key = closure.peek( 0 ).view().read_ip().0;
      for child_scope_key in self.scope_tree.view().get_children( fun_scope_key ).clone() {
        if self.scope_tree.view().read_node( child_scope_key ).is_fun_scope_for( fun_name_key ) {

          // (3) prepare the parameter env from the parameter scope
          let mut param_env = Env::from_scope_tree( &self.scope_tree, child_scope_key );
          for ( key, value ) in std::iter::zip( param_keys, args ) {
            param_env.write_symbol( key, value.clone() );
          }
          closure.push( RcMut::new( param_env ) );
          self.call_stack.push( closure );

          // the runtime now sees the function's parameter scope
          // as its current env
          return;
        }
      }
      panic!( "Internal error: Function has no parameter scope." );
    } else {
      panic!( "Internal error: Eval is not a function." );
    }
  }
  pub fn finish_fun_call( &mut self ) {
    self.call_stack.pop();
  }
  pub fn push_env( &mut self, decls: &Vec<Decl> ) {
    let env = Self::init_env( &self.scope_tree, self.curr_child_scope(), decls, &self.sc );
    self.write_env_stack().push( env );
    // println!( "[push_env] ip = {:?}", self.read_env_stack().peek( 0 ).view().read_ip() )
  }
  pub fn pop_env( &mut self ) {
    self.write_env_stack().pop();
    // println!( "[pop_env] ip = {:?}", self.read_env_stack().peek( 0 ).view().read_ip() );
  }
  pub fn adv_ip( &mut self ) {
    self.write_env_stack().peek_mut( 0 ).view_mut().adv_ip();
    // println!( "[adv_ip] ip = {:?}", self.read_env_stack().peek( 0 ).view().read_ip() )
  }
  pub fn rev_ip( &mut self ) {
    self.write_env_stack().peek_mut( 0 ).view_mut().rev_ip();
  }
  pub fn symbol_is_visible( &self, symbol_key: StringKey ) -> bool {
    for depth in 0..self.read_env_stack().depth() {
      if self.read_env_stack().peek( depth ).view().has_symbol( symbol_key ) {
        return true;
      }
    }
    false
  }
  fn get_jump( &self, symbol_key: StringKey ) -> usize {
    assert!( self.symbol_is_visible( symbol_key ), "Internal error: Missing symbol. This should have been caught in the scope tree resolver." );
    *self.scope_tree.view().read_node( self.curr_scope() ).read_jump_table().get( &symbol_key )
      .expect( "Internal error: Symbol is visible in the EnvStack, but it's not in the ScopeTree." ) as usize
  }
  pub fn read_symbol( &self, symbol_key: StringKey ) -> Eval {
    self.read_env_stack().peek( self.get_jump( symbol_key ) ).view().read_symbol( symbol_key )
  }
  pub fn write_symbol( &mut self, symbol_key: StringKey, value: Eval ) {
    let jump = self.get_jump( symbol_key );
    self.write_env_stack().peek_mut( jump ).view_mut().write_symbol( symbol_key, value );
  }
  pub fn debug_print( &self ) {
    println!( "\n{}", self.scope_tree.view().map_fold( &ScopeTreeFormatter::new( &self.sc.view() ), 0, 0 ).unwrap() );
    println!( "\nCall stack has {} entries:", self.call_stack.depth() );
    for entry in self.call_stack.iter() {
      print!( "\n{{" );
      for env in entry.iter() {
        println!( "\n\t{}", env.view().debug_format( &self.sc.view() ) );
      }
      println!( "\n}}" );
    }
  }
}