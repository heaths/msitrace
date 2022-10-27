// Copyright 2022 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use crate::{Error, Result};
use std::ffi::{c_char, CString};
use std::fmt::Display;
use std::ops::{Deref, Not};

pub const ERROR_SUCCESS: u32 = 0;
pub const ERROR_MORE_DATA: u32 = 234;
pub const MSI_NULL_INTEGER: i32 = -0x8000_0000;
pub const IDOK: u32 = 1;
pub const IDCANCEL: u32 = 2;

pub type LPSTR = *mut c_char;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct Handle(u32);

impl Handle {
    pub fn to_owned(self) -> OwnedHandle {
        OwnedHandle(self)
    }
}

impl Default for Handle {
    fn default() -> Self {
        Handle(0)
    }
}

impl Deref for Handle {
    type Target = u32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Handle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // cspell:ignore MSIHANDLE
        write!(f, "MSIHANDLE ({})", self.0)
    }
}

#[derive(Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct OwnedHandle(Handle);

impl Deref for OwnedHandle {
    type Target = Handle;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        unsafe {
            MsiCloseHandle(**self);
        }
    }
}

/// A collection of fields containing strings and integers.
#[derive(Debug)]
pub struct Record(Handle);

impl Record {
    /// Gets the count of fields in the record.
    pub fn field_count(&self) -> u32 {
        unsafe { MsiRecordGetFieldCount(**self) }
    }

    /// Gets a string field from a [`Record`].
    ///
    /// Field indices are 1-based, though you can get a template string from field 0.
    pub fn string_data(&self, field: u32) -> Result<String> {
        unsafe {
            let mut value_len = 0u32;
            let value = CString::default();

            let mut ret = MsiRecordGetString(
                **self,
                field,
                value.as_ptr() as LPSTR,
                &mut value_len as *mut u32,
            );
            if ret != ERROR_MORE_DATA {
                return Err(Error::from(ret));
            }

            let mut value_len = value_len + 1u32;
            let mut value: Vec<u8> = vec![0; value_len as usize];

            ret = MsiRecordGetString(
                **self,
                field,
                value.as_mut_ptr() as LPSTR,
                &mut value_len as *mut u32,
            );
            if ret != ERROR_SUCCESS {
                return Err(Error::from(ret));
            }

            value.truncate(value_len as usize);
            let text = String::from_utf8(value)?;

            Ok(text)
        }
    }

    /// Gets an integer field from a [`Record`].
    ///
    /// Field indices are 1-based.
    pub fn integer_data(&self, field: u32) -> Option<i32> {
        unsafe {
            match MsiRecordGetInteger(**self, field) {
                i if i == MSI_NULL_INTEGER => None,
                i => Some(i),
            }
        }
    }

    /// Gets whether a field is null in a [`Record`].
    ///
    /// Field indices are 1-based.
    pub fn is_null(&self, field: u32) -> bool {
        unsafe { MsiRecordIsNull(**self, field).into() }
    }

    fn format_text(&self) -> Result<String> {
        unsafe {
            let mut value_len = 0u32;
            let value = CString::default();

            let mut ret = MsiFormatRecord(
                Handle::default(),
                **self,
                value.as_ptr() as LPSTR,
                &mut value_len as *mut u32,
            );
            if ret != ERROR_MORE_DATA {
                return Err(Error::from(ret));
            }

            let mut value_len = value_len + 1u32;
            let mut value: Vec<u8> = vec![0; value_len as usize];

            ret = MsiFormatRecord(
                Handle::default(),
                **self,
                value.as_mut_ptr() as LPSTR,
                &mut value_len as *mut u32,
            );
            if ret != ERROR_SUCCESS {
                return Err(Error::from(ret));
            }

            value.truncate(value_len as usize);
            let text = String::from_utf8(value)?;

            Ok(text)
        }
    }
}

impl Deref for Record {
    type Target = Handle;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.format_text().unwrap_or_else(|_| "(record)".to_owned());
        write!(f, "{}", s)
    }
}

#[derive(Copy, Clone, Debug, Default)]
#[repr(transparent)]
pub struct BOOL(i32);

impl From<bool> for BOOL {
    #[inline]
    fn from(value: bool) -> Self {
        match value {
            true => BOOL(1),
            false => BOOL(0),
        }
    }
}

impl Into<bool> for BOOL {
    fn into(self) -> bool {
        self.0 != 0
    }
}

impl Not for BOOL {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self.0 {
            0 => BOOL(1),
            _ => BOOL(0),
        }
    }
}

impl PartialEq<bool> for BOOL {
    fn eq(&self, other: &bool) -> bool {
        let this = self.0 != 0;
        this == *other
    }
}

#[repr(u32)]
pub enum MessageType {
    FatalExit = 0x00000000,
    Error = 0x01000000,
    Warning = 0x02000000,
    User = 0x03000000,
    Info = 0x04000000,
    OutOfDiskSpace = 0x07000000,
    ActionStart = 0x08000000,
    ActionData = 0x09000000,
    CommonData = 0x0B000000,
    Initialize = 0x0C000000,
    Terminate = 0x0D000000,
    InstallStart = 0x1A000000,
    InstallEnd = 0x1B000000,
}

type UIRecordHandler = extern "C" fn(i32, MessageType, Record) -> u32;

#[link(name = "msi")]
extern "C" {

    fn MsiRecordGetFieldCount(h: Handle) -> u32;

    #[link_name = "MsiRecordGetStringA"]
    fn MsiRecordGetString(h: Handle, index: u32, value: LPSTR, value_len: *mut u32) -> u32;

    fn MsiRecordGetInteger(h: Handle, index: u32) -> i32;

    fn MsiRecordIsNull(h: Handle, index: u32) -> BOOL;

    fn MsiCloseHandle(h: Handle) -> u32;

    fn MsiSetExternalUIRecord(
        handler: UIRecordHandler,
        context: i32,
        previous_handler: *mut UIRecordHandler,
    ) -> u32;

    #[link_name = "MsiFormatRecordA"]
    fn MsiFormatRecord(install: Handle, record: Handle, value: LPSTR, value_len: *mut u32) -> u32;
}
