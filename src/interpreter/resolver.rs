////////////////////////////////////////////////
// private module rlox::interpreter::resolver //
////////////////////////////////////////////////


/////////
// use //
/////////

use crate::util::*;

use crate::interpreter::decl::*;
use crate::interpreter::stmt::*;
use crate::interpreter::expr::*;
use crate::interpreter::token::*;
use crate::interpreter::scope::*;


//////////////////////
// public interface //
//////////////////////

// traverses the AST and produces a scope tree
pub struct Resolver {
  sc: RcMut<StringCache>,
  scope_tree: ScopeTree,
  had_error: bool
}

impl Resolver {

  pub fn new( sc: RcMut<StringCache> ) -> Resolver {
    Resolver {
      sc,
      scope_tree: Tree::new( Scope::new( 0 ) ),
      had_error: false
    }
  }

  pub fn resolve( &mut self, decls: &Vec<Decl> ) -> ( Tree<Scope>, bool ) {
    self.restart();

    self.resolve_decls( decls, 0 );

    
    let _ = self.scope_tree.accept_mut( &ScopeTreeResolver::new() );

    if let Result::Ok( s ) = self.scope_tree.accept( &ScopeTreeFormatter::new( &self.sc.view() ) ) {
      println!( "{}", s );
    }


    let scopes = self.scope_tree.clone();
    ( scopes, self.had_error )
  }

  ////////////////////////////
  // private implementation //
  ////////////////////////////

  fn restart( &mut self ) {
    self.scope_tree = Tree::new( Scope::new( 0 ) );
    self.had_error = false;
  }

  fn resolve_decls( &mut self, decls: &Vec<Decl>, scope_key: u64 ) {
    for decl in decls {
      self.resolve_decl( decl, scope_key );
    }
  }

  fn resolve_decl( &mut self, decl: &Decl, scope_key: u64 ) {
    match decl {
      Decl::Stmt( stmt )
        => self.resolve_stmt( stmt, scope_key ),
      Decl::Fun( fun_name, arg_names, body)
        => self.resolve_fun_decl( fun_name, arg_names, body, scope_key ),
      Decl::Var( var_name, init)
        => self.resolve_var_decl( var_name, init.as_ref(), scope_key ),
    }
  }

  fn resolve_stmt( &mut self, stmt: &Stmt, scope_key: u64 ) {
    match stmt {
      Stmt::Expr( expr ) => {
        self.resolve_expr( expr, scope_key );
      },
      Stmt::Print( expr ) => {
        self.resolve_expr( expr, scope_key );
      },
      Stmt::Block( decls, line ) => {
        let block_scope_key = self.scope_tree.add_node( scope_key, Scope::new( *line ) );
        self.resolve_decls( decls, block_scope_key );
      },
      Stmt::If( init, condition, then, else_ ) => {
        let if_scope_key = self.scope_tree.add_node( scope_key, Scope::new( -100 ) );
        if let Some( cfi ) = init {
          self.resolve_ctrl_flow_init( cfi, if_scope_key );
        }
        self.resolve_expr( condition, if_scope_key );
        self.resolve_stmt( then, if_scope_key );
        if let Some( stmt ) = else_ {
          self.resolve_stmt( stmt, if_scope_key );
        }
      },
      Stmt::While( init, condition, body ) => {
        let while_scope_key = self.scope_tree.add_node( scope_key, Scope::new( -200 ) );
        if let Some( cfi ) = init {
          self.resolve_ctrl_flow_init( cfi, while_scope_key );
        }
        self.resolve_expr( condition, while_scope_key );
        self.resolve_stmt( body, while_scope_key );
      },
      Stmt::For( init, condition, incr, body ) => {
        let for_scope_key = self.scope_tree.add_node( scope_key, Scope::new( -300 ) );
        if let Some( cfi ) = init {
          self.resolve_ctrl_flow_init( cfi, for_scope_key );
        }
        if let Some( cond ) = condition {
          self.resolve_expr( cond, for_scope_key );
        }
        if let Some( inc ) = incr {
          self.resolve_expr( inc, for_scope_key );
        }
        self.resolve_stmt( body, for_scope_key );
      },
      Stmt::Return( retval ) => {
        if let Some( expr ) = retval {
          self.resolve_expr( expr, scope_key );
        }
      }
    }
  }

  fn resolve_ctrl_flow_init( &mut self, init: &CtrlFlowInit, scope_key: u64 ) {
    match init {
        CtrlFlowInit::VarDecl( decl ) => {
          self.resolve_decl( decl, scope_key );
        },
        CtrlFlowInit::ExprStmt( stmt ) => {
          self.resolve_stmt( stmt, scope_key );
        },
    }
  }

  fn resolve_fun_decl( &mut self, fun_name: &Token, arg_names: &Vec<Token>, body: &Stmt, scope_key: u64 ) {

    // fun_name is added to current scope as a local
    self.write_scope( scope_key ).add_local_symbol( fun_name.get_key() );

    // everything else belongs to a *new* scope, fun_scope
    let fun_scope_key = self.scope_tree.add_node( scope_key, Scope::new( fun_name.get_line() ) );

    // arg_names
    for arg_name in arg_names {
      self.write_scope( fun_scope_key ).add_local_symbol( arg_name.get_key() );
    }

    // body
    self.resolve_stmt( body, fun_scope_key );
  }

  fn resolve_var_decl( &mut self, var_name: &Token, init: Option<&Expr>, scope_key: u64 ) {

    // var_name is added to the current scope as a local
    self.write_scope( scope_key ).add_local_symbol( var_name.get_key() );

    if let Some( rhs ) = init {
      self.resolve_expr( rhs, scope_key );
    }
  }

  fn resolve_expr( &mut self, expr: &Expr, scope_key: u64 ) {
    match expr {
      Expr::Assignment( symbol_name, rhs ) => {
        self.write_scope( scope_key ).add_symbol( symbol_name.get_key() );
        self.resolve_expr( rhs, scope_key );
      },
      Expr::Binary( lhs, _, rhs ) => {
        self.resolve_expr( lhs, scope_key );
        self.resolve_expr( rhs, scope_key );
      },
      Expr::Call( callee, _, args ) => {
        self.resolve_expr( callee, scope_key );
        for arg in args {
          self.resolve_expr( arg, scope_key );
        }
      },
      Expr::Grouping( expr ) => {
        self.resolve_expr( expr, scope_key );
      },
      Expr::Literal( t ) => {
        if let TokenType::Identifier( symbol_key ) = t.get_type() {
          println!( "Warning: Identifier found as Expr::Literal. Is this ok?" );
          self.write_scope( scope_key ).add_symbol( *symbol_key );
        }
      },
      Expr::Unary( _, expr ) => {
        self.resolve_expr( expr, scope_key );
      },
      Expr::Symbol( symbol_name ) => {
        self.write_scope( scope_key ).add_symbol( symbol_name.get_key() );
      },
    }
  }

  fn read_scope( &self, scope_key: u64 ) -> &Scope {
    self.scope_tree.read_node( scope_key )
  }

  fn write_scope( &mut self, scope_key: u64 ) -> &mut Scope {
    self.scope_tree.write_node( scope_key )
  }
}