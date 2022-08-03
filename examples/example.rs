use std::collections::HashMap;
use std::error::Error;
use std::result::Result;

use abserde::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
struct MyConfig {
	window_width: usize,
	window_height: usize,
	window_x: usize,
	window_y: usize,
	theme: String,
	user_data: HashMap<String, String>,
}

fn main() -> Result<(), Box<dyn Error>> {
	let my_abserde = Abserde {
		app: "MyApp".to_string(),
		location: Location::Auto,
		format: Format::Json,
	};

	let mut my_config = MyConfig {
		..Default::default()
	};

	my_config.save_config(&my_abserde)?;

	my_config = MyConfig::load_config(&my_abserde)?;

	println!("{:#?}", my_config);

	Ok(my_abserde.delete()?)
}
