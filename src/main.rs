use rlox::{run_file, run_prompt};

fn main() {
  let args: Vec<String> = std::env::args().collect();

  if args.len() > 2 {
    eprintln!( "Usage: {} [optional:script]", args.get( 0 ).unwrap() );
  }
  if args.len() == 2 {
    run_file( args.get( 1 ).unwrap() );
  }
  else {
    run_prompt();
  }
}