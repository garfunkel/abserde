# abserde

[![Crates.io](https://img.shields.io/crates/v/abserde)](https://crates.io/crates/abserde)
[![Crates.io](https://img.shields.io/crates/d/abserde)](https://crates.io/crates/abserde)
[![Crates.io](https://img.shields.io/crates/l/abserde)](https://crates.io/crates/abserde)
[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/garfunkel/abserde/Rust)](https://github.com/garfunkel/abserde/actions/workflows/rust.yml)
[![docs.rs](https://img.shields.io/docsrs/abserde)](https://docs.rs/abserde/latest/abserde)

Simple platform-agnostic Rust crate for managing application settings/preferences.

## Installation

Install the crate as a dependency in your app's Cargo.toml file:

```toml
[dependencies]
abserde = "0.4.0"
```

## Usage

Import [Abserde](https://docs.rs/abserde/latest/abserde/struct.Abserde.html), associated definitions, and [serde::Serialize](https://docs.rs/serde/latest/serde/trait.Serialize.html), and [serde::Deserialize](https://docs.rs/serde/latest/serde/trait.Deserialize.html):

```rust
use abserde::*;
use serde::{Serialize, Deserialize};
```

Define a struct to store your app config.
You must derive your struct from [serde::Serialize](https://docs.rs/serde/latest/serde/trait.Serialize.html) and [serde::Deserialize](https://docs.rs/serde/latest/serde/trait.Deserialize.html) traits.

```rust
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct MyConfig {
	window_width: usize,
	window_height: usize,
	window_x: usize,
	window_y: usize,
	theme: String,
	user_data: HashMap<String, String>,
}
```

Create an [Abserde](https://docs.rs/abserde/latest/abserde/struct.Abserde.html) instance to manage how your configuration is stored on disk:

```no_run
let my_abserde = Abserde::default();
```

Using [Abserde](https://docs.rs/abserde/latest/abserde/struct.Abserde.html) in this way will use your crate as the name for the app config directory.

Alternatively, you can also pass options to [Abserde](https://docs.rs/abserde/latest/abserde/struct.Abserde.html) to change the location or format of your config file:

```rust
let my_abserde = Abserde {
	app: "MyApp".to_string(),
	location: Location::Auto,
	format: Format::Json,
};
```

Load data into config from disk:

```rust
let my_config = MyConfig::load_config(&my_abserde)?;
```

Save config data to disk:

```rust
my_config.save_config(&my_abserde)?;
```

Delete config file from disk:

```rust
my_abserde.delete()?;
```
