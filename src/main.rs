use std::process;

use mirror;

fn main() {
  if let Err(error) = mirror::run() {
    eprintln!("error: {}", error);
    process::exit(1);
  }
}
