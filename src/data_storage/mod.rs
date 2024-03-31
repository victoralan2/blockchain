use std::convert::Into;
use std::env;
use std::string::ToString;
use std::sync::Mutex;

use lazy_static::lazy_static;

use crate::core::parameters::COIN_NAME;

pub mod blockchain_storage;
pub mod node_config_storage;

lazy_static! {
	static ref BASE_DIRECTORY: Mutex<String> = {
		Mutex::new(
			if cfg!(target_os = "windows") {
				format!("{}/{}/", dirs::config_dir().expect("Unable to get config directory").to_str().unwrap(), COIN_NAME.to_lowercase()) // AppData/Roaming/**
			} else if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
				format!("{}/.{}/", dirs::home_dir().expect("Unable to get home directory").to_str().unwrap(), COIN_NAME.to_lowercase()) // /home/../.** ...
			} else {
				format!("{}/{}/", dirs::data_dir().expect("Unable to get data directory").to_str().unwrap(), COIN_NAME.to_lowercase())
			}
		)
	};
}

pub struct BaseDirectory;
impl BaseDirectory {
	pub fn get_base_directory() -> String {
		let lock = BASE_DIRECTORY.lock().expect("Unable to acquire base directory lock");
		lock.clone()
	}

	pub fn set_base_directory(new_basedir: &str) {
		let mut mut_lock = BASE_DIRECTORY.lock().expect("Unable to acquire base directory mutable lock");
		*mut_lock = new_basedir.to_string();
	}
}
