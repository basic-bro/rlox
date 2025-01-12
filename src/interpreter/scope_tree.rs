/////////////////////////////////////////////
// private module rlox::interpreter::scope //
/////////////////////////////////////////////


/////////
// use //
/////////

use std::collections::HashMap;

use crate::util::*;

use crate::interpreter::token::*;
use crate::interpreter::expr::*;
use crate::interpreter::stmt::*;
use crate::interpreter::decl::*;


//////////////////
// declarations //
//////////////////

#[derive(Clone)]
pub struct Scope {
  rsolns: HashMap<StringKey, i32>,
  line: i32
}

pub type ScopeTree = Tree<Scope>;

struct ScopeTreeBuilder_Expr {
  tree: RcMut<ScopeTree>,
  scope_key: u64
}

struct ScopeTreeBuilder_Stmt {
  tree: RcMut<ScopeTree>,
  stack: Stack<u64>
}

pub struct ScopeTreeBuilder_Decl {
  tree: RcMut<ScopeTree>,
  scope_stack: Stack<u64>
}

// finalises the scope tree
pub struct ScopeTreeResolver {
  parent_key_stack: Stack<u64>
}


/////////////////////
// implementations //
/////////////////////

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

impl ScopeTreeResolver {
  pub fn new() -> ScopeTreeResolver {
    ScopeTreeResolver {
      parent_key_stack: Stack::new()
    }
  }
  fn get_parent_key_stack( &self ) -> Stack<u64> {
    self.parent_key_stack.clone()
  }
}

impl TreeMutVisitor<Scope, String> for ScopeTreeResolver {
  fn visit( &mut self, tree: &mut Tree<Scope>, node_key: u64, _depth: u32 ) -> Result<(), String> {

    // go back through this node's ancestry, looking for symbols defined therein
    let mut parent_key_stack = self.get_parent_key_stack();
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
  
  fn before_children( &mut self, _db: &mut Tree<Scope>, node_key: u64, _depth: u32 ) {
    self.parent_key_stack.push( node_key );
  }
  
  fn after_children( &mut self, _db: &mut Tree<Scope>, _node_key: u64, _depth: u32 ) {
    self.parent_key_stack.pop();
  }
}

impl ScopeTreeBuilder_Expr {
  pub fn new( tree: RcMut<ScopeTree>, scope_key: u64 ) -> ScopeTreeBuilder_Expr {
    ScopeTreeBuilder_Expr {
      tree,
      scope_key
    }
  }
  fn add_symbol( &mut self, symbol_name: Token ) {
    self.tree.view_mut().write_node( self.scope_key ).add_symbol( symbol_name.get_key() );
  }
}

impl ExprVisitor<()> for ScopeTreeBuilder_Expr {
  fn visit( &mut self, node: &Expr ) -> Result<(), ()> {
    match node {
      Expr::Assignment( symbol_name, _ ) => self.add_symbol( *symbol_name ),
      Expr::Binary( _lhs, _op, _rhs) => {},
      Expr::Call( _callee, _paren, _args ) => {},
      Expr::Grouping( _inner) => {},
      Expr::Literal( literal) => {
        if let TokenType::Identifier( _ ) = literal.get_type() {
          println!( "Warning: Identifier found as a literal. Is that ok?" );
        }
      },
      Expr::Unary( _op, _rhs) => {},
      Expr::Symbol( symbol_name ) => self.add_symbol( *symbol_name ),
    }
    Ok( () )
  }  
  fn before_children( &mut self, _node: &Expr ) { }
  fn after_children( &mut self, _node: &Expr ) { }
}

impl ScopeTreeBuilder_Stmt {
  pub fn new( tree: RcMut<ScopeTree>, scope_key: u64 ) -> ScopeTreeBuilder_Stmt {
    let mut builder = ScopeTreeBuilder_Stmt {
      tree: tree.clone(),
      stack: Stack::new()
    };
    builder.stack.push( scope_key );
    builder
  }
  fn curr_scope( &self ) -> u64 {
    *self.stack.peek( 0 )
  }
  fn push_scope( &mut self, line: i32 ) {
    let cs = self.curr_scope();
    self.stack.push( self.tree.view_mut().add_node( cs, Scope::new( line ) ) );
  }
  fn pop_scope( &mut self ) {
    self.stack.pop();
  }
}

impl StmtVisitor<()> for ScopeTreeBuilder_Stmt {
    fn get_expr_visitor( &mut self ) -> impl ExprVisitor<()> {
      ScopeTreeBuilder_Expr::new( self.tree.clone(), self.curr_scope() )
    }
    fn get_decl_visitor( &mut self ) -> impl DeclVisitor<()> {
      ScopeTreeBuilder_Decl::new( self.tree.clone(), self.curr_scope() )
    }
    fn visit( &self, _node: &Stmt ) -> Result<(), ()> { Ok( () ) }
    fn before_children( &mut self, stmt: &Stmt ) {
      match stmt {
        Stmt::Block( _, line ) => self.push_scope( *line ),
        Stmt::If( _, _, _, _ ) => self.push_scope( -100 ),
        Stmt::While( _, _, _ ) => self.push_scope( -200 ),
        Stmt::For( _, _, _, _ ) => self.push_scope( -300 ),
        _ => {}
      }
    }
    fn after_children( &mut self, stmt: &Stmt ) {
      match stmt {
        Stmt::Block( _, _ ) => self.pop_scope(),
        Stmt::If( _, _, _, _ ) => self.pop_scope(),
        Stmt::While( _, _, _ ) => self.pop_scope(),
        Stmt::For( _, _, _, _ ) => self.pop_scope(),
        _ => {}
      }
    }
}

impl ScopeTreeBuilder_Decl {
  pub fn new( tree: RcMut<ScopeTree>, scope_key: u64 ) -> ScopeTreeBuilder_Decl {
    let mut builder = ScopeTreeBuilder_Decl {
      tree: tree.clone(),
      scope_stack: Stack::new()
    };
    builder.scope_stack.push( scope_key );
    builder
  }
  fn curr_scope( &self ) -> u64 {
    *self.scope_stack.peek( 0 )
  }
  fn push_scope( &mut self, line: i32 ) {
    let cs = self.curr_scope();
    self.scope_stack.push( self.tree.view_mut().add_node( cs, Scope::new( line ) ) );
  }
  fn pop_scope( &mut self ) {
    self.scope_stack.pop();
  }
  fn add_local_symbol( &mut self, symbol_name: Token ) {
    let cs = self.curr_scope();
    self.tree.view_mut().write_node( cs ).add_local_symbol( symbol_name.get_key() );
  }
}

impl DeclVisitor<()> for ScopeTreeBuilder_Decl {
  fn get_stmt_visitor( &mut self ) -> impl StmtVisitor<()> {
    ScopeTreeBuilder_Stmt::new( self.tree.clone(), self.curr_scope() )
  }
  fn visit( &mut self, _node: &Decl ) -> Result<(), ()> { Ok( () ) }
  fn before_children( &mut self, decl: &Decl ) {
    match decl {
      Decl::Stmt( _ ) => {},
      Decl::Fun( fun_name, arg_names, _ ) => {
        self.add_local_symbol( *fun_name );
        self.push_scope( fun_name.get_line() );
        for arg_name in arg_names {
          self.add_local_symbol( *arg_name );
        }
      },
      Decl::Var( var_name, _ ) => {
        self.add_local_symbol( *var_name );
      },
    }
  }
  fn after_children( &mut self, decl: &Decl ) {
    match decl {
      Decl::Stmt( _ ) => {},
      Decl::Fun( _, _, _ ) => {
        self.pop_scope();
      },
      Decl::Var( _, _ ) => {}
    }
  }
}