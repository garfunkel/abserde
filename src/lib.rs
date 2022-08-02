//! Simple platform-agnostic Rust crate for managing application settings/preferences.
//!
//! # Installation
//!
//! Install the crate as a dependency in your app's Cargo.toml file:
//! ```text
//! [dependencies]
//! abserde = "0.1.0"
//! ```
//!
//! # Usage
//! Import Abserde, [serde::Serialize], and [serde::Deserialize]:
//! ```
//! use abserde::*;
//! use serde::{Serialize, Deserialize};
//! ```
//!
//! Define a struct to store your app settings/data.
//! You must derive your struct from [serde::Serialize] and [serde::Deserialize] traits.
//! ```text
//! #[derive(Serialize, Deserialize)]
//! struct MyConfig {
//! 	window_width: usize,
//! 	window_height: usize,
//! 	window_x: usize,
//! 	window_y: usize,
//! 	theme: String,
//! 	user_data: HashMap<String, String>,
//! }
//! ```
//!
//! Create an Abserde instance to manage how your configuration is stored on disk:
//! ```text
//! let my_abserde = Abserde {
//!		app: "MyApp".to_string(),
//!		location: Location::Auto,
//!		format: Format::Json,
//!	};
//! ```
//!
//! Load data into a `MyConfig` object:
//! ```text
//! let my_config = MyConfig::load_config(&my_abserde);
//! ```
//!
//! Save config to disk:
//! ```text
//! my_config.save_config(&my_abserde);
//! ```
//!
//! Delete config from disk:
//! ```text
//! my_abserde.delete();
//! ```

#![deny(missing_docs)]

use std::fs::{create_dir_all, remove_dir, remove_file, File};
use std::path::PathBuf;
use std::{error, io, result};

use serde::{de::DeserializeOwned, Serialize};

const MSG_NO_SYSTEM_CONFIG_DIR: &str = "no system config directory detected";

/// Alias for generic Error type.
pub type Error = Box<dyn error::Error>;

/// Alias for Result type wrapping generic Error type.
pub type Result<T> = result::Result<T, Error>;

/// Storage format for app config.
///
/// Each format is enabled as a feature. The json feature is included by default.
/// All other format features are disabled by default.
#[derive(Debug, PartialEq, Clone)]
pub enum Format {
	/// JSON format using the serde_json crate.
	#[cfg(feature = "json")]
	Json,

	/// YAML format using the serde_yaml crate.
	#[cfg(feature = "yaml")]
	Yaml,

	/// Pickle (Python) format using the serde-pickle crate.
	#[cfg(feature = "pickle")]
	Pickle,

	/// INI (Windows) format using the serde_ini crate.
	#[cfg(feature = "ini")]
	Ini,

	/// TOML format using the toml crate.
	#[cfg(feature = "toml")]
	Toml,
}

impl Format {
	/// Return default file name of config file for this format.
	pub fn default_name(&self) -> String {
		format!("config.{:?}", self).to_lowercase()
	}
}

/// Represents the location of a config file.
#[derive(Debug, PartialEq, Clone)]
pub enum Location {
	/// Automatically determines location of config file based on platform/OS.
	Auto,

	/// Provides the full path to the config file.
	Path(String),

	/// Automatically determines config directory, with file name specified manually.
	File(String),

	/// Automatically determines config file name, with directory specified manually.
	Dir(String),
}

/// Represents an Abserde app, specifying how app settings are to be managed.
#[derive(Debug, PartialEq, Clone)]
pub struct Abserde {
	/// App name under which app settings are typically to be stored.
	pub app: String,

	/// Location specification for where app settings are physically kept.
	pub location: Location,

	/// Format for app setting storage and serialisation.
	pub format: Format,
}

impl Abserde {
	/// Delete settings file related to this app.
	pub fn delete(&self) -> Result<()> {
		let system_config_dir = dirs::config_dir()
			.ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, MSG_NO_SYSTEM_CONFIG_DIR))?;

		let config_path = match &self.location {
			Location::Auto => system_config_dir
				.join(&self.app)
				.join(&self.format.default_name()),
			Location::Path(path) => PathBuf::from(path),
			Location::Dir(dir) => PathBuf::from(dir).join(&self.format.default_name()),
			Location::File(file) => system_config_dir.join(&self.app).join(file),
		};

		remove_file(&config_path)?;

		match &self.location {
			// Don't attempt to delete folder if manually specifying folder.
			Location::Dir(_) => {}
			// Attempt to delete parent folder if it is empty.
			_ => {
				let config_dir = config_path.parent().ok_or_else(|| {
					io::Error::new(io::ErrorKind::NotFound, MSG_NO_SYSTEM_CONFIG_DIR)
				})?;

				// Ignore any errors here, as they are sometimes expected.
				_ = remove_dir(config_dir);
			}
		}

		Ok(())
	}
}

/// Trait that apps can implement to store app settings.
///
/// Implementing types must also implement [serde::Serialize] and [serde::Deserialize] traits.
pub trait Config {
	/// Type of implementation.
	type T;

	/// Load a config from disk into the implementing type.
	fn load_config(abserde: &Abserde) -> Result<Self::T>;

	/// Save a config from the implementing type to disk.
	fn save_config(&self, abserde: &Abserde) -> Result<()>;
}

impl<T> Config for T
where
	T: Serialize,
	T: DeserializeOwned,
{
	type T = T;

	fn load_config(abserde: &Abserde) -> Result<Self::T> {
		let system_config_dir = dirs::config_dir()
			.ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, MSG_NO_SYSTEM_CONFIG_DIR))?;

		let config_path = match &abserde.location {
			Location::Auto => system_config_dir
				.join(&abserde.app)
				.join(&abserde.format.default_name()),
			Location::Path(path) => PathBuf::from(path),
			Location::Dir(dir) => PathBuf::from(dir).join(&abserde.format.default_name()),
			Location::File(file) => system_config_dir.join(&abserde.app).join(file),
		};

		Ok(match abserde.format {
			#[cfg(feature = "json")]
			Format::Json => {
				let file = File::open(config_path)?;

				serde_json::from_reader(io::BufReader::new(file))?
			}
			#[cfg(feature = "yaml")]
			Format::Yaml => {
				let file = File::open(config_path)?;

				serde_yaml::from_reader(io::BufReader::new(file))?
			}
			#[cfg(feature = "pickle")]
			Format::Pickle => {
				let file = File::open(config_path)?;

				serde_pickle::from_reader(io::BufReader::new(file), serde_pickle::DeOptions::new())?
			}
			#[cfg(feature = "ini")]
			Format::Ini => {
				let file = File::open(config_path)?;

				serde_ini::from_read(io::BufReader::new(file))?
			}
			#[cfg(feature = "toml")]
			Format::Toml => {
				use io::Read;

				let mut file = File::open(config_path)?;
				let mut buf = String::new();

				file.read_to_string(&mut buf)?;

				toml::from_str(&buf)?
			}
		})
	}

	fn save_config(&self, abserde: &Abserde) -> Result<()> {
		let system_config_dir = dirs::config_dir()
			.ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, MSG_NO_SYSTEM_CONFIG_DIR))?;

		let config_path = match &abserde.location {
			Location::Auto => system_config_dir
				.join(&abserde.app)
				.join(&abserde.format.default_name()),
			Location::Path(path) => PathBuf::from(path),
			Location::Dir(dir) => PathBuf::from(dir).join(&abserde.format.default_name()),
			Location::File(file) => system_config_dir.join(&abserde.app).join(file),
		};

		let config_dir = config_path
			.parent()
			.ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, MSG_NO_SYSTEM_CONFIG_DIR))?;

		create_dir_all(config_dir)?;

		match abserde.format {
			#[cfg(feature = "json")]
			Format::Json => {
				serde_json::to_writer(File::create(&config_path)?, self)?;
			}
			#[cfg(feature = "yaml")]
			Format::Yaml => {
				serde_yaml::to_writer(File::create(&config_path)?, self)?;
			}
			#[cfg(feature = "pickle")]
			Format::Pickle => {
				serde_pickle::to_writer(
					&mut File::create(&config_path)?,
					self,
					serde_pickle::SerOptions::new(),
				)?;
			}
			#[cfg(feature = "ini")]
			Format::Ini => {
				serde_ini::to_writer(File::create(&config_path)?, self)?;
			}
			#[cfg(feature = "toml")]
			Format::Toml => {
				use io::Write;

				write!(File::create(&config_path)?, "{}", toml::to_string(self)?)?;
			}
		}

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use std::collections::HashMap;
	use std::fmt::Debug;

	use fake::{Dummy, Fake, Faker};
	use serde::{de::DeserializeOwned, Deserialize, Serialize};

	use crate::{Abserde, Config, Format, Location};

	const APP_NAME: &str = "rust_prefs_test";

	// Test config type for serialisation formats that only accept basic types.
	#[derive(Serialize, Deserialize, Debug, Default, Dummy, PartialEq)]
	struct TestConfigSimple {
		string_val: String,
		i8_val: i8,
		i16_val: i16,
		i32_val: i32,
		u8_val: u8,
		u16_val: u16,
		u32_val: u32,
		f32_val: f32,
	}

	// More complex config type for serialisation formats that support advanced types.
	#[derive(Serialize, Deserialize, Debug, Default, Dummy, PartialEq)]
	struct TestConfigComplex {
		string_val: String,
		i8_val: i8,
		i16_val: i16,
		i32_val: i32,
		i64_val: i64,
		i128_val: i128,
		u8_val: u8,
		u16_val: u16,
		u32_val: u32,
		u64_val: u64,
		u128_val: u128,
		f32_val: f32,
		f64_val: f64,
		vec_1_val: Vec<i64>,
		vec_2_val: Vec<(String, i8, i8, i32, i64, String, String, String)>,
		vec_3_val: Vec<(f32, f32, f32, f64, f64)>,
		hash_map_1_val: HashMap<String, String>,
		hash_map_2_val: HashMap<i16, i64>,
		hash_map_3_val: HashMap<i16, Vec<(String, i32, u8, HashMap<String, i32>)>>,
		hash_map_4_val: HashMap<String, (f64, f32, i8)>,
	}

	// Generic dispatch method.
	fn test_save_load<T>(abserde: &Abserde)
	where
		T: Serialize,
		T: DeserializeOwned,
		T: Dummy<Faker>,
		T: PartialEq,
		T: Debug,
	{
		let test_config_saved: T = Faker.fake();

		test_config_saved.save_config(&abserde).unwrap();

		let test_config_loaded = T::load_config(&abserde).unwrap();

		assert_eq!(test_config_saved, test_config_loaded);

		abserde.delete().unwrap();
	}

	#[cfg(feature = "json")]
	#[test]
	fn test_save_load_json() {
		test_save_load::<TestConfigComplex>(&Abserde {
			app: APP_NAME.to_string(),
			location: Location::Auto,
			format: Format::Json,
		});
	}

	#[cfg(feature = "yaml")]
	#[test]
	fn test_save_load_yaml() {
		test_save_load::<TestConfigComplex>(&Abserde {
			app: APP_NAME.to_string(),
			location: Location::Auto,
			format: Format::Yaml,
		});
	}

	#[cfg(feature = "pickle")]
	#[test]
	fn test_save_load_pickle() {
		test_save_load::<TestConfigSimple>(&Abserde {
			app: APP_NAME.to_string(),
			location: Location::Auto,
			format: Format::Pickle,
		});
	}

	#[cfg(feature = "ini")]
	#[test]
	fn test_save_load_ini() {
		test_save_load::<TestConfigSimple>(&Abserde {
			app: APP_NAME.to_string(),
			location: Location::Auto,
			format: Format::Ini,
		});
	}

	#[cfg(feature = "toml")]
	#[test]
	fn test_save_load_toml() {
		test_save_load::<TestConfigSimple>(&Abserde {
			app: APP_NAME.to_string(),
			location: Location::Auto,
			format: Format::Toml,
		});
	}
}
