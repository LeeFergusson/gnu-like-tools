use std::{fs::File, io::Read};
use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
/// Concatenate FILE(s) to standard output.
/// With no FILE, or when FILE is -, read standard input.
struct Args {
  #[arg(name = "FILE", required = true)]
  files: Vec<String>,
}

fn main() {
  let args = Args::parse();
  
  for passed_file in args.files {
    match File::open(&passed_file) {
      Ok(mut file) => {
        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Ok(_) => print!("{}", contents),
            Err(_) => eprintln!("Could not read file: {}", passed_file),
        }
        ;
      }
      Err(e) => eprintln!("{}", e),
    }
  }
}
