// Imports. -------------------------------------------------------------------
use chrono::DateTime;
use clap::Parser;
use std::{
  fs::{File, FileTimes, OpenOptions},
  io::{Error, ErrorKind},
  time::{Duration, SystemTime},
};

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
  #[arg(short('d'), long("date"), default_value = None)]
  date: Option<String>,

  /// Change the modification time only.
  #[arg(short('m'), long = None, conflicts_with = "update_access_only", default_value = "false")]
  update_modification_only: bool,

  /// Use this file's times instead of the current time.
  #[arg(short('r'), long("reference"))]
  file_reference: Option<String>,

  /// Files to update.
  #[arg(name = "FILES", required = true)]
  files: Vec<String>,
}

// Main entry point. ----------------------------------------------------------
fn main() {
  let args = Args::parse();
  let mut file_time = SystemTime::now();

  if let Some(time) = &args.date {
    if let Some(system_time) = parse_time(&time) {
      file_time = system_time;
    }
  }

  for file in &args.files {
    match update_file(file, file_time, &args) {
      Ok(_) => {}
      Err(error) => {
        eprintln!("Error updating file: {}", error);
      }
    }
  }
}

// Functions. -----------------------------------------------------------------

/// ## Parse the time string.
///
/// ### Arguments:
/// * `time` - The time string to parse.
///
/// ### Returns:
/// * `Option<SystemTime>` - The parsed time.
fn parse_time(time: &str) -> Option<SystemTime> {
  let parsed_time = DateTime::parse_from_str(time, "%Y-%m-%d %H:%M:%S.%3f %z");
  match parsed_time {
    Ok(offset) => {
      if let Some(date_time) = DateTime::from_timestamp(0, 0) {
        let duration = offset.signed_duration_since(date_time);
        return Some(
          SystemTime::UNIX_EPOCH
            + Duration::from_secs(duration.num_seconds() as u64),
        );
      }
    }
    Err(_) => {
      eprintln!(
        "Invalid date format. Use: -d \"YYYY-MM-DD HH:MM:SS.sss +HHMM\""
      );
    }
  }
  None
}

/// ## Update the access and modification times of a file.
///
/// ### Arguments:
/// * `file` - The file to update.
/// * `time` - The time to update the file to.
/// * `args` - The command line arguments.
///
/// ### Returns:
/// * `Result<(), Error>` - The result of the operation.
fn update_file(file: &str, time: SystemTime, args: &Args) -> Result<(), Error> {
  match OpenOptions::new().write(true).open(file) {
    Ok(file) => {
      file.set_times(get_file_times(time, args))?;
      return Ok(());
    }
    Err(error) => match error.kind() {
      ErrorKind::PermissionDenied => {
        eprintln!("Error updating file: Permission denied");
      }
      ErrorKind::NotFound => {
        if !args.no_create {
          match File::create(file) {
            Ok(_) => update_file(file, time, args)?,
            Err(error) => eprintln!("Error creating file: {}", error),
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

/// ## Get the file times to update.
///
/// ### Arguments:
/// * `time` - The time to update the file to.
/// * `args` - The command line arguments.
///
/// ### Returns:
/// * `FileTimes` - The file times to apply.
fn get_file_times(time: SystemTime, args: &Args) -> FileTimes {
  let file_times = FileTimes::new();

  if args.update_access_only {
    file_times.set_accessed(time)
  } else if args.update_modification_only {
    file_times.set_modified(time)
  } else {
    file_times.set_accessed(time).set_modified(time)
  }
}
