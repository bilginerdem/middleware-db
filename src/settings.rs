use crate::get_full_path_with_file;
use config::{Config, ConfigError, File, FileFormat};
use serde::Deserialize;
use std::{mem::MaybeUninit, sync::Once};

fn get_instance() -> &'static Config {
  static mut SINGLETON: MaybeUninit<Config> = MaybeUninit::uninit();
  static ONCE: Once = Once::new();

  unsafe {
    ONCE.call_once(|| {
      let service_binary_path = get_full_path_with_file("config.ini");
      let singleton = Config::builder()
        .add_source(File::new(service_binary_path.to_str().unwrap(), FileFormat::Ini))
        .build()
        .unwrap();
      SINGLETON.write(singleton);
    });

    SINGLETON.assume_init_ref()
  }
}

fn get<'de, T: Deserialize<'de>>(key: &str) -> Result<T, ConfigError> {
  get_instance().get::<T>(key)
}

pub fn get_string(key: &str, def: Option<&str>) -> String {
  if def.is_none() {
    return get(key).unwrap_or_else(|_| panic!("settings.{} not found", key));
  }
  get(key).unwrap_or_else(|_| String::from(def.unwrap_or_else(|| "")))
}

pub fn get_int(key: &str, def: Option<i64>) -> i64 {
  if def.is_none() {
    return get(key).unwrap_or_else(|_| panic!("settings.{} not found", key));
  }
  get(key).unwrap_or_else(|_| def.unwrap_or_else(|| 0i64))
}
pub fn get_usize(key: &str, def: Option<usize>) -> usize {
  if def.is_none() {
    return get(key).unwrap_or_else(|_| panic!("settings.{} not found", key));
  }
  get(key).unwrap_or_else(|_| def.unwrap_or_else(|| 0usize))
}

pub fn get_bool(key: &str, def: Option<bool>) -> bool {
  if def.is_none() {
    return get(key).unwrap_or_else(|_| panic!("settings.{} not found", key));
  }
  get(key).unwrap_or_else(|_| def.unwrap_or_else(|| false))
}

pub fn get_array(key: &str) -> Vec<String> {
  let value = get_string(key, Some(""));

  if value.is_empty() {
    Vec::new()
  } else {
    value.split(',').map(|f| f.to_string()).collect::<Vec<String>>()
  }
}
 