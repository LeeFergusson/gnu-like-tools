use clap::Parser;
use std::{
  fs::{File, FileTimes, OpenOptions},
  io::{Error, ErrorKind},
};

fn main() {
  let args = Args::parse();

  for file in &args.files {
    match update_file(file, &args) {
      Ok(_) => {}
      Err(error) => {
        eprintln!("Error updating file: {}", error);
      }
    }
  }
}

// Argument parsing. ----------------------------------------------------------

#[derive(Parser)]
#[command(version, about, long_about = None)]
/// Update the access and modification times of each FILE to the current time.
struct Args {
  /// Change the access time only.
  #[arg(short('a'), long = None, conflicts_with = "update_modification_only", default_value = "false")]
  update_access_only: bool,

  /// Do not create any files.
  #[arg(short('c'), long("no-create"), default_value = "false")]
  no_create: bool,

  /// Parse the date string.
  #[arg(short('d'), long("date"), default_value = None, conflicts_with = "update_access_only")]
  date: Option<String>,

  /// Change the modification time only.
  #[arg(short('m'), long = None)]
  update_modification_only: bool,

  /// Use this file's times instead of the current time.
  #[arg(short('r'), long("reference"))]
  file_reference: Option<String>,

  /// Files to update.
  #[arg(name = "FILES", required = true)]
  files: Vec<String>,
}

// Functions. -----------------------------------------------------------------

/// Update the access and modification times of a file.
fn update_file(file: &str, args: &Args) -> Result<(), Error> {
  // Attempt to open the file for writing.
  match OpenOptions::new().write(true).open(file) {
    Ok(file) => {
      let time = std::time::SystemTime::now();
      let file_times = FileTimes::new();

      // Update the file's times depending on the arguments.
      if args.update_access_only {
        file.set_times(file_times.set_accessed(time))?;
      } else if args.update_modification_only {
        file.set_times(file_times.set_modified(time))?;
      } else {
        file.set_times(file_times.set_accessed(time).set_modified(time))?;
      }
    }
    Err(error) => match error.kind() {
      ErrorKind::PermissionDenied => {
        eprintln!("Error updating file: Permission denied");
      }
      ErrorKind::NotFound => {
        if !args.no_create {
          match File::create(file) {
            Ok(_) => {}
            Err(error) => {
              eprintln!("Error creating file: {}", error);
            }
          };
        }
      }
      _ => {
        eprintln!("Error opening file: {}", error);
      }
    },
  };
  Ok(())
}
