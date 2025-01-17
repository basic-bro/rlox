/////////////////////////////////////////////
// private module rlox::interpreter::scope //
/////////////////////////////////////////////


/////////
// use //
/////////

use std::collections::HashMap;

use crate::util::*;

use crate::interpreter::error::*;
use crate::interpreter::token::*;
use crate::interpreter::expr::*;
use crate::interpreter::stmt::*;
use crate::interpreter::decl::*;
use crate::interpreter::visitor::*;


//////////////////
// declarations //
//////////////////

#[derive(Clone)]
pub struct Scope {
  jump_table: HashMap<StringKey, i32>,
  fun_decls: Vec<StringKey>,
  fun_scope_for: Option<StringKey>,
  line: i32
}

pub type ScopeTree = Tree<Scope>;

pub struct ScopeTreeBuilder {
  sc: RcMut<StringCache>,
  tree: RcMut<ScopeTree>,
  scope_stack: Stack<u64>
}

// finalises the scope tree
pub struct ScopeTreeResolver {
  sc: RcMut<StringCache>,
  ancestors: Stack<u64>
}


/////////////////////
// implementations //
/////////////////////

impl Scope {
  pub fn new( line: i32 ) -> Scope {
    Scope {
      jump_table: HashMap::new(),
      fun_decls: Vec::new(),
      fun_scope_for: None,
      line
    }
  }
  pub fn read_jump_table( &self ) -> &HashMap<StringKey, i32> {
    &self.jump_table
  }
  pub fn get_line( &self ) -> i32 {
    self.line
  }
  pub fn add_local_symbol( &mut self, symbol_key: StringKey, is_fun_decl: bool ) {
    self.jump_table.insert( symbol_key, 0 );
    if is_fun_decl {
      self.fun_decls.push( symbol_key );
    }
  }
  pub fn add_symbol( &mut self, symbol_key: StringKey, line: i32 ) {
    if !self.jump_table.contains_key( &symbol_key ) {
      self.jump_table.insert( symbol_key, -line );
    }
  }
  pub fn set_fun_scope( &mut self, fun_name_key: StringKey ) {
    assert( self.fun_scope_for.is_none(), format!( "Internal error: set_fun_scope() already called on this scope." ) );
    self.fun_scope_for = Some( fun_name_key );
  }
  pub fn is_fun_scope_for( &self, fun_name_key: StringKey ) -> bool {
    if let Some( key ) = self.fun_scope_for {
      key == fun_name_key
    } else {
      false
    }
  }
  pub fn has_local( &self, symbol_key: StringKey ) -> bool {
    if let Some( &jump ) = self.jump_table.get( &symbol_key ) {
      jump == 0
    }
    else {
      false
    }
  }
  pub fn new_scope_tree() -> RcMut<ScopeTree> {
    RcMut::new( ScopeTree::new( Scope::new( 0 ) ) )
  }
  pub fn resolve_scope_tree( scope_tree: &mut RcMut<ScopeTree>, sc: &RcMut<StringCache> ) -> Result<(), Error> {
    scope_tree.view_mut().accept_mut( &mut ScopeTreeResolver::new( sc ), 0, 0 )
  }
}

impl ScopeTreeResolver {
  pub fn new( sc: &RcMut<StringCache> ) -> ScopeTreeResolver {
    ScopeTreeResolver {
      sc: sc.clone(),
      ancestors: Stack::new()
    }
  }
  fn get_ancestors( &self ) -> Stack<u64> {
    self.ancestors.clone()
  }
}

impl TreeMutVisitor<Scope, Error> for ScopeTreeResolver {
  fn visit( &mut self, tree: &mut Tree<Scope>, node_key: u64, _depth: u32 ) -> Result<(), Error> {

    // go back through this node's ancestry, looking for symbols defined therein
    let mut ancestors = self.get_ancestors();
    let mut curr_jumps = 1;
    loop {

      // exit if no more ancestors to check
      if ancestors.is_empty() {
        break;
      }

      // init current parent
      let parent_key = *ancestors.peek( 0 );
      let parent_scope = tree.read_node( parent_key ).clone();

      // go through symbols in my scope
      for ( &symbol_key, &mut ref mut jumps ) in tree.write_node( node_key ).jump_table.iter_mut() {

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
      ancestors.pop();
    }

    // check for undefined variables here!
    for ( &symbol_key, &jump ) in tree.read_node( node_key ).read_jump_table() {
      if jump < 0 {
        return Err( Error::new( -jump, format!( " at '{}'", self.sc.view().gets( symbol_key ) ),
          "Undeclared variable.".to_string() ) );
      }
    }

    Ok( () )
  }
  
  fn before_children( &mut self, _db: &mut Tree<Scope>, node_key: u64, _depth: u32 ) {
    self.ancestors.push( node_key );
  }
  
  fn after_children( &mut self, _db: &mut Tree<Scope>, _node_key: u64, _depth: u32 ) {
    self.ancestors.pop();
  }
}

impl ScopeTreeBuilder {
  pub fn new( sc: &RcMut<StringCache>, tree: &RcMut<ScopeTree>, scope_key: u64 ) -> ScopeTreeBuilder {
    let mut builder = ScopeTreeBuilder {
      sc: sc.clone(),
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
  fn add_symbol( &mut self, symbol_name: Token ) {
    let cs = self.curr_scope();
    self.tree.view_mut().write_node( cs ).add_symbol( symbol_name.get_key(), symbol_name.get_line() );
  }
  fn add_local_symbol( &mut self, symbol_name: Token, is_fun_decl: bool ) {
    let cs = self.curr_scope();
    self.tree.view_mut().write_node( cs ).add_local_symbol( symbol_name.get_key(), is_fun_decl );
  }
  fn set_fun_scope( &mut self, fun_name: Token ) {
    let cs = self.curr_scope();
    self.tree.view_mut().write_node( cs ).set_fun_scope( fun_name.get_key() );
  }
  fn has_local( &self, symbol_name: Token ) -> bool {
    self.tree.view().read_node( self.curr_scope() ).has_local( symbol_name.get_key() )
  }
}

impl DeclVisitor<Error> for ScopeTreeBuilder {
  fn get_stmt_visitor( &mut self ) -> &mut impl StmtVisitor<Error> {
    self
  }
  fn visit_decl( &mut self, _node: &Decl ) -> Result<(), Error> { Ok( () ) }
  fn before_decl_children( &mut self, decl: &Decl ) -> Result<VisitorControl, Error> {
    match decl {
      Decl::Stmt( _ ) => {
        // NB: naked block (not part of a fun_decl or control flow block)
        // if let Stmt::Block( _, line ) = stmt {
        //   self.push_scope( *line );
        // }
      },
      Decl::Fun( fun_name, arg_names, _ ) => {
        if self.has_local( *fun_name ) {
          return Err( Error::from_token( fun_name, "This symbol is already in use.".to_string(), &self.sc.view() ) );
        }
        self.add_local_symbol( *fun_name, true );
        self.push_scope( fun_name.get_line() );
        self.set_fun_scope( *fun_name );
        for arg_name in arg_names {
          self.add_local_symbol( *arg_name, false );
        }
      },
      Decl::Var( var_name, _ ) => {
        if self.has_local( *var_name ) {
          return Err( Error::from_token( var_name, "This symbol is already in use.".to_string(), &self.sc.view() ) );
        }
        self.add_local_symbol( *var_name, false );
        // if let Some( expr ) = init {
        //   expr.accept( self )?;
        // }
      },
    }
    Ok( VisitorControl::VisitChildren )
  }
  fn after_decl_children( &mut self, decl: &Decl ) {
    match decl {
      Decl::Stmt( _ ) => {
        // NB: naked block (not part of a fun_decl or control flow block)
        // if let Stmt::Block( _, _ ) = stmt {
        //   self.pop_scope();
        // }
      },
      Decl::Fun( _, _, _ ) => {
        self.pop_scope();
      },
      Decl::Var( _, _ ) => {}
    }
  }
}

impl StmtVisitor<Error> for ScopeTreeBuilder {
  fn get_expr_visitor( &mut self ) -> &mut impl ExprVisitor<Error> {
    self
  }
  fn get_decl_visitor( &mut self ) -> &mut impl DeclVisitor<Error> {
    self
  }
  fn visit_stmt( &mut self, _node: &Stmt ) -> Result<(), Error> { Ok( () ) }
  fn before_stmt_children( &mut self, stmt: &Stmt ) -> Result<VisitorControl, Error> {
    match stmt {

      // Deal with naked block separately?
      // NB: naked block dealt with in DeclVisitor<Error> impl
      // Stmt::Block( _, line ) => self.push_scope( *line ),

      // Treat naked blocks just like others?
      Stmt::Block( _, line ) => self.push_scope( *line ),

      Stmt::If( _, _, _, _ ) => self.push_scope( -100 ),
      Stmt::While( _, _, _ ) => self.push_scope( -200 ),
      Stmt::For( _, _, _, _ ) => self.push_scope( -300 ),
      _ => {}
    }
    Ok( VisitorControl::VisitChildren )
  }
  fn after_stmt_children( &mut self, stmt: &Stmt ) {
    match stmt {

      // Deal with naked block separately?
      // NB: naked block dealt with in DeclVisitor<Error> impl
      // Stmt::Block( _, _ ) => self.pop_scope(),

      // Treat naked block just like others?
      Stmt::Block( _, _ ) => self.pop_scope(),

      Stmt::If( _, _, _, _ ) => self.pop_scope(),
      Stmt::While( _, _, _ ) => self.pop_scope(),
      Stmt::For( _, _, _, _ ) => self.pop_scope(),
      _ => {}
    }
  }
}

impl ExprVisitor<Error> for ScopeTreeBuilder {
  fn visit_expr( &mut self, node: &Expr ) -> Result<(), Error> {
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
  fn before_expr_children( &mut self, _node: &Expr )
    -> Result<VisitorControl, Error> { Ok( VisitorControl::VisitChildren ) }
  fn after_expr_children( &mut self, _node: &Expr ) { }
}