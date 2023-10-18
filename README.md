# file_env_const

[![Crates.io](https://img.shields.io/crates/v/file_env_const.svg?style=for-the-badge)](https://crates.io/crates/file_env_const)
[![Documentation](https://img.shields.io/docsrs/file_env_const?style=for-the-badge)](https://docs.rs/file_env_const/)
[![Build status](https://img.shields.io/github/actions/workflow/status/tveness/file_env_const/rust.yml?label=Tests&style=for-the-badge)](https://github.com/tveness/file_env_const/actions/workflows/rust.yml)
[![License](https://img.shields.io/github/license/tveness/file_env_const?style=for-the-badge)](https://opensource.org/license/MIT/)

This crate provides macros for loading data at compile time from a file or environment
variable, with fallback options.

This is useful in the case of building a binary with hard-coded strings which you may wish to
inject as environment variables in a build environment (say in CI), or store locally in a file
when building offline, with a fallback option in either case.

# Examples

## Loading a file, with environment fallback

```rust
use file_env_const::file_env;
// Read data from file first
const FILE_DATA: &'static str = file_env!("Cargo.toml", "CARGO_PKG_NAME");
let f = std::fs::read_to_string("Cargo.toml").unwrap();
assert_eq!(FILE_DATA, f);

// Tries to read data from file, falls back to environment variable which is the package name
const FALL_BACK_TO_ENV: &'static str = file_env!("file_does_not_exist", "CARGO_PKG_NAME");
assert_eq!(FALL_BACK_TO_ENV, "file_env_const");

// Tries to read data from file, falls back to environment variable which is the package name
const FALL_BACK_TO_DEFAULT: &'static str =
    file_env!("file_does_not_exist", "ENV_NOT_FOUND", "fallback string");
assert_eq!(FALL_BACK_TO_DEFAULT, "fallback string");
```

## Loading an environment variable, with file fallback

```rust
use file_env_const::env_file;
// Read data from file first
const ENV_DATA: &'static str = env_file!("CARGO_PKG_NAME", "Cargo.toml");
assert_eq!(ENV_DATA, "file_env_const");

// Tries to read data from file, falls back to environment variable which is the package name
const FALL_BACK_TO_FILE: &'static str = env_file!("ENV_NOT_FOUND", "Cargo.toml");
let f = std::fs::read_to_string("Cargo.toml").unwrap();
assert_eq!(FALL_BACK_TO_FILE, f);

// Tries to read data from file, falls back to environment variable which is the package name
const FALL_BACK_TO_DEFAULT: &'static str =
    env_file!( "ENV_NOT_FOUND", "file_does_not_exist", "fallback string");
assert_eq!(FALL_BACK_TO_DEFAULT, "fallback string");
```
