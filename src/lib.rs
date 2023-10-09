use std::{error::Error, path::PathBuf};

pub mod database;
pub mod models;
pub mod settings;

pub type SyncError = dyn Error + Send + Sync;

pub fn get_full_path_with_file(file_name: &str) -> PathBuf {
  std::env::current_exe().unwrap().with_file_name(file_name)
}

pub fn get_full_path() -> PathBuf {
  std::env::current_exe().unwrap().parent().unwrap().to_path_buf()
}
