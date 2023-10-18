//! This crate provides macros for loading data at compile time from a file or environment
//! variable, with fallback options.
//!
//! This is useful in the case of building a binary with hard-coded strings which you may wish to
//! inject as environment variables in a build environment (say in CI), or store locally in a file
//! when building offline, with a fallback option in either case.
//!
//! # Examples
//!
//! ## Loading a file, with environment fallback
//! ```
//!# use file_env_const::file_env;
//! // Read data from file first
//! const FILE_DATA: &'static str = file_env!("Cargo.toml", "CARGO_PKG_NAME");
//! let f = std::fs::read_to_string("Cargo.toml").unwrap();
//! assert_eq!(FILE_DATA, f);
//!
//! // Tries to read data from file, falls back to environment variable which is the package name
//! const FALL_BACK_TO_ENV: &'static str = file_env!("file_does_not_exist", "CARGO_PKG_NAME");
//! assert_eq!(FALL_BACK_TO_ENV, "file_env_const");
//!
//! // Tries to read data from file, falls back to env variable, and then falls back to string
//! const FALL_BACK_TO_DEFAULT: &'static str =
//!     file_env!("file_does_not_exist", "ENV_NOT_FOUND", "fallback string");
//! assert_eq!(FALL_BACK_TO_DEFAULT, "fallback string");
//! ```
//!
//! ## Loading an environment variable, with file fallback
//! ```
//!# use file_env_const::env_file;
//! // Read data from environment variable first
//! const ENV_DATA: &'static str = env_file!("CARGO_PKG_NAME", "Cargo.toml");
//! assert_eq!(ENV_DATA, "file_env_const");
//!
//! // Tries to read data from env variable, falls back to file which
//! const FALL_BACK_TO_FILE: &'static str = env_file!("ENV_NOT_FOUND", "Cargo.toml");
//! let f = std::fs::read_to_string("Cargo.toml").unwrap();
//! assert_eq!(FALL_BACK_TO_FILE, f);
//!
//! // Tries to read data from env variable, falls back to file, and then falls back to string
//! const FALL_BACK_TO_DEFAULT: &'static str =
//!     env_file!( "ENV_NOT_FOUND", "file_does_not_exist", "fallback string");
//! assert_eq!(FALL_BACK_TO_DEFAULT, "fallback string");
//! ```

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::LitStr;
use syn::Token;

enum Kind {
    Data(LitStr),
    Name(String),
}

/// Loads an environment variable, falling back to a file, falling back to a default value, all at
/// compile time
///
/// The first argument is an environment variable, the second is a filename, and the third
/// (optional) is a fallback string
///
/// # Examples
///
/// ## Without fallback string
/// ```
///# use file_env_const::env_file;
/// const ENV_DATA: &'static str = env_file!("CARGO_PKG_NAME", "Cargo.toml");
/// assert_eq!(ENV_DATA, "file_env_const");
///
/// const FILE_DATA: &'static str = env_file!("ENV_NOT_FOUND", "Cargo.toml");
/// let f = std::fs::read_to_string("Cargo.toml").unwrap();
/// assert_eq!(FILE_DATA, f);
/// ```
///
/// ## With fallback string
/// ```
///# use file_env_const::env_file;
/// const ENV_DATA: &'static str = env_file!("ENV_NOT_FOUND", "no_such_file", "fallback_string");
/// assert_eq!(ENV_DATA, "fallback_string");
/// ```
#[proc_macro]
pub fn env_file(input: TokenStream) -> TokenStream {
    let parser = Punctuated::<LitStr, Token![,]>::parse_separated_nonempty;
    let mut l = parser.parse(input.clone()).unwrap().into_iter();

    match read_from_env(&mut l) {
        Kind::Data(data) => return data.into_token_stream().into(),
        Kind::Name(name) => eprintln!(
            "No environment variable found with name {}, trying default",
            name
        ),
    }

    match read_file(&mut l) {
        Kind::Data(data) => return data.into_token_stream().into(),
        Kind::Name(name) => eprintln!("No file found at {}, trying environment variable", name),
    };

    if let Some(data) = l.next() {
        data.into_token_stream().into()
    } else {
        panic!(
            r#"No filename argument supplied, try file_env!("filename", "ENV_NAME", "default_value")"#
        );
    }
}

/// Loads a file, falling back to an environment variable, falling back to a default value, all at
/// compile time
///
/// The first argument is a filename, the second is an environment variable, and the third
/// (optional) is a fallback string
///
/// # Examples
///
/// ## Without fallback string
/// ```
///# use file_env_const::file_env;
/// const FILE_DATA: &'static str = file_env!("Cargo.toml", "CARGO_PKG_NAME");
/// let f = std::fs::read_to_string("Cargo.toml").unwrap();
/// assert_eq!(FILE_DATA, f);
///
/// const ENV_DATA: &'static str = file_env!("no_such_file", "CARGO_PKG_NAME");
/// assert_eq!(ENV_DATA, "file_env_const");
///
/// ```
///
/// ## With fallback string
/// ```
///# use file_env_const::file_env;
/// const FILE_DATA: &'static str = file_env!("no_such_file", "ENV_NOT_FOUND", "fallback_string");
/// assert_eq!(FILE_DATA, "fallback_string");
/// ```
#[proc_macro]
pub fn file_env(input: TokenStream) -> TokenStream {
    let parser = Punctuated::<LitStr, Token![,]>::parse_separated_nonempty;
    let mut l = parser.parse(input.clone()).unwrap().into_iter();

    match read_file(&mut l) {
        Kind::Data(data) => return data.into_token_stream().into(),
        Kind::Name(name) => eprintln!("No file found at {}, trying environment variable", name),
    };

    match read_from_env(&mut l) {
        Kind::Data(data) => return data.into_token_stream().into(),
        Kind::Name(name) => eprintln!(
            "No environment variable found with name {}, trying default",
            name
        ),
    }

    if let Some(data) = l.next() {
        data.into_token_stream().into()
    } else {
        panic!(
            r#"No filename argument supplied, try file_env!("filename", "ENV_NAME", "default_value")"#
        );
    }
}

fn read_file<I>(parser_list: &mut I) -> Kind
where
    I: Iterator<Item = LitStr>,
{
    if let Some(x) = parser_list.next() {
        let filename = x.value();
        match std::fs::read_to_string(filename.clone()) {
            Ok(d) => Kind::Data(LitStr::new(&d, x.span())),

            Err(_) => Kind::Name(filename),
        }
    } else {
        panic!("No filename argument supplied");
    }
}

fn read_from_env<I>(parser_list: &mut I) -> Kind
where
    I: Iterator<Item = LitStr>,
{
    if let Some(x) = parser_list.next() {
        let env_var_name = x.value();
        match std::env::var(env_var_name.clone()) {
            Ok(s) => Kind::Data(LitStr::new(&s, x.span())),
            Err(_) => Kind::Name(env_var_name),
        }
    } else {
        panic!("No env argument supplied");
    }
}
