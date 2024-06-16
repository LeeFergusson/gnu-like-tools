use std::{fs::File, io::{BufRead, BufReader, ErrorKind}, process::exit};
use clap::Parser;

// CLI ------------------------------------------------------------------------
#[derive(Parser)]
#[command(version, about, long_about = None)]
/// Concatenate FILE(s) to standard output.
/// With no FILE, or when FILE is -, read standard input.
struct Args {
  // Options --------------------------------------------------------

  // Show line endings.
  #[arg(short('E'), long("show-ends"))]
  show_ends: bool,

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

  for path in args.files {
    let _ = File::open(&path)
      .map(|file| {
        let lines = file_to_lines(file);
        for (i, line) in lines.iter().enumerate() {
          let mut string_buffer;

          if args.number {
            string_buffer = format!("{:6} {}", i + 1, line);
          } else {
            string_buffer = line.to_string();
          }
          if args.show_ends {
            string_buffer += "$";
          }
          println!("{}", string_buffer);
        }
      })
      .map_err(|error| match error.kind() {
        ErrorKind::NotFound => {
          eprintln!("File not found: {}", path);
          exit(1)
        }
        ErrorKind::PermissionDenied => {
          eprintln!("Permission denied: {}", path);
          exit(1)
        }
        _ => {
          eprintln!("Error opening file: {}", path);
          exit(1)
        }
      });
  }
}

/// ## Read file into a vector of lines.
///
/// ### Arguments
/// * `file` - A file to read.
///
/// ### Returns
/// A vector of lines.
fn file_to_lines(file: File) -> Vec<String> {
  BufReader::new(file)
    .lines()
    .map(|line| line.unwrap_or_default())
    .collect()
}
