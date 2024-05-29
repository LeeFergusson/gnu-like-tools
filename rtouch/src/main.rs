use clap::Arg;
use std::{
  env::args,
  fs::{File, FileTimes, OpenOptions},
};

fn main() {
  args()
    .skip(1)
    .for_each(|path| match OpenOptions::new().write(true).open(&path) {
      Ok(file) => {
        let time = std::time::SystemTime::now();
        match file.set_times(FileTimes::new().set_accessed(time).set_modified(time)) {
          Ok(_) => println!("File updated: {}", path),
          Err(error) => match error.kind() {
            std::io::ErrorKind::PermissionDenied => {
              eprintln!("Error updating file: Permission denied");
            }
            _ => {
              eprintln!("Error updating file: {}", error);
            }
          },
        };
      }
      Err(error) => match error.kind() {
        std::io::ErrorKind::NotFound => {
          match File::create(&path) {
            Ok(_) => {
              println!("New file created: {}", path);
            }
            Err(error) => {
              eprintln!("Error creating file: {}", error);
            }
          };
        }
        _ => {
          eprintln!("Error opening file: {}", error);
        }
      },
    });
}
