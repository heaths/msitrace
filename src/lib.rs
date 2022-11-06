// Copyright 2022 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use std::ffi::NulError;
use std::fmt::Display;
use std::string::FromUtf8Error;
use time::OffsetDateTime;

mod ffi;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
enum ErrorKind {
    ErrorCode(u32),
    Other(Box<dyn std::error::Error>),
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ErrorKind::ErrorCode(code) => write!(f, "{}", code),
            ErrorKind::Other(err) => write!(f, "{:?}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            ErrorKind::Other(err) => err.source(),
            _ => None,
        }
    }
}

impl From<u32> for Error {
    fn from(code: u32) -> Self {
        Error {
            kind: ErrorKind::ErrorCode(code),
        }
    }
}

impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Self {
        Error {
            kind: ErrorKind::Other(Box::new(err)),
        }
    }
}

impl From<NulError> for Error {
    fn from(err: NulError) -> Self {
        Error {
            kind: ErrorKind::Other(Box::new(err)),
        }
    }
}

pub use ffi::UILevel;
pub fn install(
    path: &str,
    log: Option<String>,
    ui: UILevel,
    properties: Vec<String>,
) -> Result<()> {
    let command_line = properties.join(" ");

    ffi::set_internal_ui(ui);
    if let Some(log) = log {
        ffi::enable_log(log.as_str())?;
    }

    ffi::set_external_handler(|message, record| {
        let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
        println!("{:?} ({:?}) {}", now, message, record);

        ffi::HandlerResult::Default
    })?;

    ffi::install_package(path, command_line.as_str())
}
