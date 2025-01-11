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
use crate::interpreter::scope_tree::*;
use crate::interpreter::format::*;


//////////////////////
// public interface //
//////////////////////

// traverses the AST and produces a scope tree
pub struct Resolver {
  sc: RcMut<StringCache>,
  scope_tree: RcMut<ScopeTree>,
  had_error: bool
}

impl Resolver {

  pub fn new( sc: RcMut<StringCache> ) -> Resolver {
    Resolver {
      sc,
      scope_tree: RcMut::new( Tree::new( Scope::new( 0 ) ) ),
      had_error: false
    }
  }

  pub fn resolve( &mut self, decls: &Vec<Decl> ) -> ( RcMut<ScopeTree>, bool ) {
    self.restart();

    self.resolve_decls( decls, 0 );


    //
    // variable declarations inside a block statement don't get noticed.
    //

    
    let _ = self.scope_tree.view_mut().map_fold_mut( &ScopeTreeResolver::new(), 0, 0 );

    if let Result::Ok( s ) = self.scope_tree.view().map_fold( &ScopeTreeFormatter::new( &self.sc.view() ), 0, 0 ) {
      println!( "{}", s );
    }


    let scopes = self.scope_tree.clone();
    ( scopes, self.had_error )
  }

  ////////////////////////////
  // private implementation //
  ////////////////////////////

  fn restart( &mut self ) {
    self.scope_tree = RcMut::new( Tree::new( Scope::new( 0 ) ) );
    self.had_error = false;
  }

  fn resolve_decls( &mut self, decls: &Vec<Decl>, scope_key: u64 ) {
    for decl in decls {
      self.resolve_decl( decl, scope_key );
    }
  }

  fn resolve_decl( &mut self, decl: &Decl, scope_key: u64 ) {
    let _ = decl.map_fold_mut( &mut ScopeTreeBuilder_Decl::new( self.scope_tree.clone(), scope_key ) );
    // match decl {
    //   Decl::Stmt( stmt )
    //     => self.resolve_stmt( stmt, scope_key ),
    //   Decl::Fun( fun_name, arg_names, body)
    //     => self.resolve_fun_decl( fun_name, arg_names, body, scope_key ),
    //   Decl::Var( var_name, init)
    //     => self.resolve_var_decl( var_name, init.as_ref(), scope_key ),
    // }
  }

  fn resolve_stmt( &mut self, stmt: &Stmt, scope_key: u64 ) {
    // let _ = stmt.map_fold_mut( &mut ScopeTreeBuilder_Stmt::new( self.scope_tree.clone(), scope_key ) );
    // match stmt {
    //   Stmt::Expr( expr ) => {
    //     self.resolve_expr( expr, scope_key );
    //   },
    //   Stmt::Print( expr ) => {
    //     self.resolve_expr( expr, scope_key );
    //   },
    //   Stmt::Block( decls, line ) => {
    //     let block_scope_key = self.scope_tree.view_mut().add_node( scope_key, Scope::new( *line ) );
    //     self.resolve_decls( decls, block_scope_key );
    //   },
    //   Stmt::If( init, condition, then, else_ ) => {
    //     let if_scope_key = self.scope_tree.view_mut().add_node( scope_key, Scope::new( -100 ) );
    //     if let Some( cfi ) = init {
    //       self.resolve_ctrl_flow_init( cfi, if_scope_key );
    //     }
    //     self.resolve_expr( condition, if_scope_key );
    //     self.resolve_stmt( then, if_scope_key );
    //     if let Some( stmt ) = else_ {
    //       self.resolve_stmt( stmt, if_scope_key );
    //     }
    //   },
    //   Stmt::While( init, condition, body ) => {
    //     let while_scope_key = self.scope_tree.view_mut().add_node( scope_key, Scope::new( -200 ) );
    //     if let Some( cfi ) = init {
    //       self.resolve_ctrl_flow_init( cfi, while_scope_key );
    //     }
    //     self.resolve_expr( condition, while_scope_key );
    //     self.resolve_stmt( body, while_scope_key );
    //   },
    //   Stmt::For( init, condition, incr, body ) => {
    //     let for_scope_key = self.scope_tree.view_mut().add_node( scope_key, Scope::new( -300 ) );
    //     if let Some( cfi ) = init {
    //       self.resolve_ctrl_flow_init( cfi, for_scope_key );
    //     }
    //     if let Some( cond ) = condition {
    //       self.resolve_expr( cond, for_scope_key );
    //     }
    //     if let Some( inc ) = incr {
    //       self.resolve_expr( inc, for_scope_key );
    //     }
    //     self.resolve_stmt( body, for_scope_key );
    //   },
    //   Stmt::Return( retval ) => {
    //     if let Some( expr ) = retval {
    //       self.resolve_expr( expr, scope_key );
    //     }
    //   }
    // }
  }

  // fn resolve_ctrl_flow_init( &mut self, init: &CtrlFlowInit, scope_key: u64 ) {
  //   match init {
  //       CtrlFlowInit::VarDecl( decl ) => {
  //         self.resolve_decl( decl, scope_key );
  //       },
  //       CtrlFlowInit::ExprStmt( stmt ) => {
  //         self.resolve_stmt( stmt, scope_key );
  //       },
  //   }
  // }

  // fn resolve_fun_decl( &mut self, fun_name: &Token, arg_names: &Vec<Token>, body: &Stmt, scope_key: u64 ) {

  //   // fun_name is added to current scope as a local
  //   self.scope_tree.view_mut().write_node( scope_key ).add_local_symbol( fun_name.get_key() );

  //   // everything else belongs to a *new* scope, fun_scope
  //   let fun_scope_key = self.scope_tree.view_mut().add_node( scope_key, Scope::new( fun_name.get_line() ) );

  //   // arg_names
  //   for arg_name in arg_names {
  //     self.scope_tree.view_mut().write_node( fun_scope_key ).add_local_symbol( arg_name.get_key() );
  //   }

  //   // body
  //   self.resolve_stmt( body, fun_scope_key );
  // }

  // fn resolve_var_decl( &mut self, var_name: &Token, init: Option<&Expr>, scope_key: u64 ) {

  //   // var_name is added to the current scope as a local
  //   self.scope_tree.view_mut().write_node( scope_key ).add_local_symbol( var_name.get_key() );

  //   if let Some( rhs ) = init {
  //     self.resolve_expr( rhs, scope_key );
  //   }
  // }

  // fn resolve_expr( &mut self, expr: &Expr, scope_key: u64 ) {
  //   let _ = expr.map_fold_mut(
  //     &mut ScopeTreeBuilder_Expr::new(
  //       self.scope_tree.clone(), scope_key
  //     )
  //   );
  // }
}


struct ScopeTreeBuilder_Expr {
  tree: RcMut<ScopeTree>,
  scope_key: u64
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

impl ExprVisitorMut<(), ()> for ScopeTreeBuilder_Expr {
    fn fold_mut_assignment( &mut self, symbol_name: &Token, _right: () ) -> Result<(), ()> {
        self.add_symbol( *symbol_name );
        Ok( () )
    }
    fn fold_mut_binary( &mut self, _left: (), _op: &Token, _right: () ) -> Result<(), ()> {
        Ok( () )
    }
    fn fold_mut_call( &mut self, _callee: (), _paren: &Token, _args: &Vec<()> ) -> Result<(), ()> {
      Ok( () )
    }
    fn fold_mut_grouping( &mut self, _expr: () ) -> Result<(), ()> {
      Ok( () )
    }
    fn fold_mut_literal( &mut self, _literal: &Token ) -> Result<(), ()> {
      if let TokenType::Identifier( _ ) = _literal.get_type() {
        println!( "Warning: Identifier found as a literal. Is that ok?" );
      }
      Ok( () )
    }
    fn fold_mut_unary( &mut self, _op: &Token, _expr: () ) -> Result<(), ()> {
      Ok( () )
    }
    fn fold_mut_symbol( &mut self, symbol_name: &Token ) -> Result<(), ()> {
      self.add_symbol( *symbol_name );
      Ok( () )
    }
}



struct ScopeTreeBuilder_Stmt {
  tree: RcMut<ScopeTree>,
  stack: Stack<u64>
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
  fn add_symbol( &mut self, symbol_name: Token ) {
    let cs = self.curr_scope();
    self.tree.view_mut().write_node( cs ).add_symbol( symbol_name.get_key() );
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


impl StmtVisitorMut<(), ()> for ScopeTreeBuilder_Stmt {
    fn get_expr_visitor_mut( &mut self ) -> impl ExprVisitorMut<(), ()> {
      ScopeTreeBuilder_Expr::new( self.tree.clone(), self.curr_scope() )
    }
    fn get_decl_visitor_mut( &mut self ) -> impl DeclVisitorMut<(), ()> {
      ScopeTreeBuilder_Decl::new( self.tree.clone(), self.curr_scope() )
    }
    fn fold_mut_expr( &mut self, _expr: () ) -> Result<(), ()> { Ok( () ) }
    fn fold_mut_print( &mut self, _expr: () ) -> Result<(), ()> { Ok( () ) }
    fn fold_mut_block( &mut self, _decls: Vec<()>, _line: i32 ) -> Result<(), ()> { Ok( () ) }
    fn fold_mut_if( &mut self, _init: Option<()>, _condition: (), _then: (), _else_: Option<()> ) -> Result<(), ()> { Ok( () ) }
    fn fold_mut_while( &mut self, _init: Option<()>, _condition: (), _body: () ) -> Result<(), ()> { Ok( () ) }
    fn fold_mut_for( &mut self, _init: Option<()>, _condition: Option<()>, _incr: Option<()>, _body: () ) -> Result<(), ()> { Ok( () ) }
    fn fold_mut_return( &mut self, _expr: Option<()> ) -> Result<(), ()> { Ok( () ) }
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






struct ScopeTreeBuilder_Decl {
  tree: RcMut<ScopeTree>,
  scope_stack: Stack<u64>
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

impl DeclVisitorMut<(), ()> for ScopeTreeBuilder_Decl {
  fn get_stmt_visitor_mut( &mut self ) -> impl StmtVisitorMut<(), ()> {
    ScopeTreeBuilder_Stmt::new( self.tree.clone(), self.curr_scope() )
  }
  fn fold_mut_stmt( &mut self, _stmt: () ) -> Result<(), ()> { Ok( () ) }
  fn fold_mut_fun( &mut self, _fun_name: Token, _arg_names: Vec<Token>, _body: () ) -> Result<(), ()> { Ok( () ) }
  fn fold_mut_var( &mut self, _var_name: Token, _init: Option<()> ) -> Result<(), ()> { Ok( () ) }
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