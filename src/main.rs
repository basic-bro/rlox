
use rlox::interpreter::*;

fn main() {
  let args: Vec<String> = std::env::args().collect();

  if args.len() > 2 {
    eprintln!( "Usage: {} [optional:script]", args.get( 0 ).unwrap() );
  }

  let mut interpreter = Interpreter::new();
  if args.len() == 2 {
    interpreter.run_file( args.get( 1 ).unwrap() );
  }
  else {
    interpreter.run_prompt();
  }
}