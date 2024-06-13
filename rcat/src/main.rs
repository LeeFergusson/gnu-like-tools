use std::{fs::File, io::{BufRead, BufReader, ErrorKind}, process::exit};
use clap::Parser;

// CLI ------------------------------------------------------------------------
#[derive(Parser)]
#[command(version, about, long_about = None)]
/// Concatenate FILE(s) to standard output.
/// With no FILE, or when FILE is -, read standard input.
struct Args {
  // Options --------------------------------------------------------

  /// Number all output lines.
  #[arg(short('n'), long("number"))]
  number: bool,

  // Positional arguments -------------------------------------------
  #[arg(name = "FILE", required = true)]
  files: Vec<String>,
}
// ----------------------------------------------------------------------------
fn main() {
  let args = Args::parse();
  
  for passed_file in args.files {
    match File::open(&passed_file) {
      Ok(file) => {
        let lines = split_into_lines(file);

        for (i, line) in lines.iter().enumerate() {
          if args.number {
            println!("{:6} {}", i + 1, line);
          } else {
            println!("{}", line);
          }
        }
      }
      Err(error) => match error.kind() {
        ErrorKind::NotFound => {
          eprintln!("File not found: {}", passed_file);
          exit(1)
        }
        ErrorKind::PermissionDenied => {
          eprintln!("Permission denied: {}", passed_file);
          exit(1)
        }
        _ => {
          eprintln!("Error opening file: {}", passed_file);
          exit(1)
        }
      }
    }
  }
}

fn split_into_lines(file: File) -> Vec<String> {
  let reader = BufReader::new(file);
  reader.lines().map(|line| {
    match line {
      Ok(value) => value,
      Err(error) => match error.kind() {
        ErrorKind::InvalidData => {
          eprintln!("Invalid UTF-8 data in file");
          exit(1)
        }
        _ => {
          eprintln!("Error reading file");
          exit(1)
        }
      }
    }
  }).collect()
}