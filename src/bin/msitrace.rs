// Copyright 2022 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use clap::error::ErrorKind;
use clap::Parser;
use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    if !args.path.exists() {
        let err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        return Err(Box::new(err));
    }

    let path = args.path.canonicalize()?;
    let path = path.to_string_lossy();
    let path = path.strip_prefix(r"\\?\").unwrap_or_else(|| path.as_ref());

    let mut log: Option<String> = None;
    if args.log.is_some() {
        let log_path = std::env::current_dir()?.join(args.log.unwrap());
        let log_path = log_path.to_string_lossy();

        log = Some(String::from(log_path));
    }

    msitrace::install(path, log, args.ui, args.properties)?;

    Ok(())
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the package to install.
    path: PathBuf,

    #[arg(long)]
    log: Option<PathBuf>,

    /// The user interface level to show.
    #[arg(long, value_enum, default_value_t)]
    ui: msitrace::UILevel,

    /// Properties to pass to the install.
    #[arg(last = true, value_parser = validate_property)]
    properties: Vec<String>,
}

fn validate_property(value: &str) -> clap::error::Result<String> {
    type Error = clap::Error;

    if value.is_empty() {
        return Err(Error::raw(
            ErrorKind::ValueValidation,
            "property cannot be empty",
        ));
    }

    if value.match_indices('=').count() != 1 {
        return Err(Error::raw(
            ErrorKind::ValueValidation,
            "requires PROP= or PROP=VALUE",
        ));
    }

    Ok(value.to_owned())
}
