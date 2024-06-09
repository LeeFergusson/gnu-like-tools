/// # rtouch
///
/// Update the access and modification times of each FILE to the current time.
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

  /// Change the modification time only.
  #[arg(short('m'), long = None, conflicts_with = "update_access_only", default_value = "false")]
  update_modification_only: bool,

  /// Use this file's times instead of the current time.
  #[arg(short('r'), long("reference"), conflicts_with = "time", default_value = None)]
  reference_file: Option<String>,

  /// Attempt to parse the time string.
  #[arg(short('t'), long("time"), default_value = None, conflicts_with = "reference_file")]
  time: Option<String>,

  /// Files to update.
  #[arg(name = "FILE", required = true)]
  files: Vec<String>,
}

// Main entry point. ----------------------------------------------------------
fn main() -> Result<(), Error> {
  let time = SystemTime::now();
  let args = Args::parse();
  let mut file_times = FileTimes::new();
  // The default is to update both the access and modification times.
  file_times = file_times.set_accessed(time).set_modified(time);

  // If a time is provided, use it instead of the current time.
  if let Some(time) = &args.time {
    match parse_time(time) {
      Ok(system_time) => {
        if args.update_access_only {
          file_times = file_times.set_accessed(system_time);
        } else if args.update_modification_only {
          file_times = file_times.set_modified(system_time);
        } else {
          file_times = file_times
            .set_accessed(system_time)
            .set_modified(system_time);
        }
      }
      Err(error) => {
        eprintln!("Error parsing time: {}", error);
      }
    }
  }
  // If a file reference is provided, use its times instead of the current time.
  else if let Some(reference) = &args.reference_file {
    match parse_reference(reference, &args) {
      Ok(times) => {
        file_times = times;
      }
      Err(error) => {
        let error_type = "Error parsing reference file:";
        match error.kind() {
          ErrorKind::NotFound => {
            eprintln!("{} File not found", error_type);
          }
          ErrorKind::PermissionDenied => {
            eprintln!("{} Permission denied", error_type);
          }
          ErrorKind::Unsupported => {
            eprintln!("{} Unsupported operation", error_type);
          }
          _ => {
            eprintln!("{} {}", error_type, error);
          }
        }
      }
    }
  }

  // Update the access and modification times of each file.
  for file in &args.files {
    match update_file(file, file_times, &args) {
      Ok(_) => {}
      Err(error) => {
        let error_type = "Error updating file:";
        match error.kind() {
          ErrorKind::PermissionDenied => {
            eprintln!("{} Permission denied", error_type);
          }
          ErrorKind::Unsupported => {
            eprintln!("{} Unsupported operation", error_type);
          }
          _ => {
            eprintln!("{} {}", error_type, error);
          }
        }
      }
    }
  }
  Ok(())
}

// Functions. -----------------------------------------------------------------

/// ## Parse the time string.
///
/// ### Arguments:
/// * `time` - The time string to parse.
///
/// ### Returns:
/// * `Result<SystemTime, Error>` - The parsed time.
fn parse_time(time: &str) -> Result<SystemTime, Error> {
  let formats = [
    // ISO 8601
    "%Y-%m-%dT%H:%M:%S.%3f%z",
    "%Y-%m-%dT%H:%M:%S%z",
    // With SPACE as time separator
    "%Y-%m-%d %H:%M:%S.%3f%z",
    "%Y-%m-%d %H:%M:%S%z",
    // Without time separator
    "%Y-%m-%d%H:%M:%S.%3f%z",
    "%Y-%m-%d%H:%M:%S%z",
  ];
  for format in formats {
    match DateTime::parse_from_str(time, format) {
      Ok(offset) => {
        if let Some(date_time) = DateTime::from_timestamp(0, 0) {
          return Ok(
            SystemTime::UNIX_EPOCH
              + Duration::from_secs(
                offset.signed_duration_since(date_time).num_seconds() as u64,
              ),
          );
        }
      }
      Err(_) => continue,
    }
  }
  Err(Error::new(ErrorKind::InvalidInput, "Unsupported date format"))
}

/// ## Parse the reference file.
///
/// ### Arguments:
/// * `path` - The path to the reference file.
/// * `args` - The command line arguments.
///
/// ### Returns:
/// * `Result<FileTimes, Error>` - The file times of the reference file.
fn parse_reference(path: &str, args: &Args) -> Result<FileTimes, Error> {
  let file_times = FileTimes::new();
  let metadata = File::open(path)?.metadata()?;

  if args.update_access_only {
    return Ok(file_times.set_accessed(metadata.accessed()?));
  } else if args.update_modification_only {
    return Ok(file_times.set_modified(metadata.modified()?));
  } else {
    return Ok(
      file_times
        .set_accessed(metadata.accessed()?)
        .set_modified(metadata.modified()?),
    );
  }
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
fn update_file(file: &str, time: FileTimes, args: &Args) -> Result<(), Error> {
  match OpenOptions::new().write(true).open(file) {
    Ok(file) => {
      file.set_times(time)?;
    }
    Err(error) => match error.kind() {
      ErrorKind::NotFound => {
        if !args.no_create {
          match File::create(file) {
            Ok(_) => update_file(file, time, args)?,
            Err(error) => {
              eprintln!("Error creating file: {}", error)
            }
          };
        }
      }
      _ => return Err(error),
    },
  };
  Ok(())
}
