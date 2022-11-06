// Copyright 2022 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use crate::{Error, Result};
use std::ffi::{c_char, c_void, CString};
use std::fmt::Display;
use std::ops::{BitOr, Deref, Not};

pub const ERROR_SUCCESS: u32 = 0;
pub const ERROR_MORE_DATA: u32 = 234;
pub const MSI_NULL_INTEGER: i32 = -0x8000_0000;
pub type LPSTR = *mut c_char;
pub type LPCSTR = *const c_char;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct MsiHandle(u32);

impl MsiHandle {
    pub fn to_owned(self) -> OwnedMsiHandle {
        OwnedMsiHandle(self)
    }
}

impl Deref for MsiHandle {
    type Target = u32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for MsiHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // cspell:ignore MSIHANDLE
        write!(f, "MSIHANDLE ({})", self.0)
    }
}

#[derive(Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct OwnedMsiHandle(MsiHandle);

impl Deref for OwnedMsiHandle {
    type Target = MsiHandle;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for OwnedMsiHandle {
    fn drop(&mut self) {
        unsafe {
            MsiCloseHandle(**self);
        }
    }
}

/// A collection of fields containing strings and integers.
#[derive(Debug)]
pub struct Record(OwnedMsiHandle);

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
                MsiHandle::default(),
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
                MsiHandle::default(),
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
    type Target = MsiHandle;
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
pub struct Win32Bool(i32);

impl From<bool> for Win32Bool {
    #[inline]
    fn from(value: bool) -> Self {
        match value {
            true => Win32Bool(1),
            false => Win32Bool(0),
        }
    }
}

impl Into<bool> for Win32Bool {
    fn into(self) -> bool {
        self.0 != 0
    }
}

impl Not for Win32Bool {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self.0 {
            0 => Win32Bool(1),
            _ => Win32Bool(0),
        }
    }
}

impl PartialEq<bool> for Win32Bool {
    fn eq(&self, other: &bool) -> bool {
        let this = self.0 != 0;
        this == *other
    }
}

#[repr(u32)]
pub enum HandlerResult {
    Default = 0u32,
    OK,
    Cancel,
}

#[derive(Debug)]
#[repr(u32)]
pub enum MessageType {
    FatalExit = 0x00000000,
    Error = 0x01000000,
    Warning = 0x02000000,
    User = 0x03000000,
    Info = 0x04000000,
    ActionStart = 0x08000000,
    ActionData = 0x09000000,
    CommonData = 0x0B000000,
    Initialize = 0x0C000000,
    Terminate = 0x0D000000,
    InstallStart = 0x1A000000,
    InstallEnd = 0x1B000000,
}

impl BitOr<MessageType> for u32 {
    type Output = u32;
    fn bitor(self, rhs: MessageType) -> Self::Output {
        self | (rhs as u32)
    }
}

impl BitOr for MessageType {
    type Output = u32;
    fn bitor(self, rhs: Self) -> Self::Output {
        (self as u32) | (rhs as u32)
    }
}

#[derive(clap::ValueEnum, Clone, Debug)]
#[repr(u32)]
pub enum UILevel {
    Default = 1,
    None,
    Basic,
    Reduced,
    Full,
}

impl Default for UILevel {
    fn default() -> Self {
        UILevel::Default
    }
}

pub fn set_external_handler<F>(handler: F) -> Result<()>
where
    F: Fn(MessageType, &Record) -> HandlerResult,
{
    #[derive(Copy, Clone)]
    struct Context<'a> {
        handler: &'a dyn Fn(MessageType, &Record) -> HandlerResult,
    }
    let context = Context { handler: &handler };

    extern "C" fn proc(context: *mut c_void, message: MessageType, handle: MsiHandle) -> u32 {
        let context = unsafe { *(context as *const Context) };
        let record = Record(handle.to_owned());
        (context.handler)(message, &record) as u32
    }

    // All MessageTypes we want to support.
    let filter: u32 = MessageType::FatalExit
        | MessageType::Error
        | MessageType::Warning
        | MessageType::User
        | MessageType::Info
        | MessageType::ActionStart
        | MessageType::ActionData
        | MessageType::CommonData
        | MessageType::Initialize
        | MessageType::Terminate
        | MessageType::InstallStart
        | MessageType::InstallEnd;

    unsafe {
        let previous_handler: *mut c_void = std::ptr::null_mut();
        let ret = MsiSetExternalUIRecord(
            proc,
            filter,
            &context as *const Context as *const c_void,
            previous_handler,
        );
        if ret != ERROR_SUCCESS {
            return Err(Error::from(ret));
        }
        Ok(())
    }
}

pub fn set_internal_ui(ui: UILevel) {
    let handle: *mut c_void = std::ptr::null_mut();
    unsafe {
        MsiSetInternalUI(ui, handle);
    }
}

pub fn enable_log(path: &str) -> Result<()> {
    const VERBOSE: u32 = 0x1000;
    let path = CString::new(path)?;

    unsafe {
        match MsiEnableLog(VERBOSE, path.as_ptr(), 0) {
            ERROR_SUCCESS => Ok(()),
            err => Err(crate::Error::from(err)),
        }
    }
}

pub fn install_package(path: &str, command_line: &str) -> Result<()> {
    let path = CString::new(path)?;
    let command_line = CString::new(command_line)?;

    unsafe {
        match MsiInstallProduct(path.as_ptr(), command_line.as_ptr()) {
            ERROR_SUCCESS => Ok(()),
            err => Err(crate::Error::from(err)),
        }
    }
}

type UIRecordHandler = extern "C" fn(*mut c_void, MessageType, MsiHandle) -> u32;

#[link(name = "msi")]
extern "C" {

    fn MsiRecordGetFieldCount(h: MsiHandle) -> u32;

    #[link_name = "MsiRecordGetStringA"]
    fn MsiRecordGetString(h: MsiHandle, index: u32, value: LPSTR, value_len: *mut u32) -> u32;

    fn MsiRecordGetInteger(h: MsiHandle, index: u32) -> i32;

    fn MsiRecordIsNull(h: MsiHandle, index: u32) -> Win32Bool;

    fn MsiCloseHandle(h: MsiHandle) -> u32;

    fn MsiSetExternalUIRecord(
        handler: UIRecordHandler,
        filter: u32,
        context: *const c_void,
        previous_handler: *mut c_void,
    ) -> u32;

    fn MsiSetInternalUI(level: UILevel, parent: *mut c_void) -> UILevel;

    #[link_name = "MsiEnableLogA"]
    fn MsiEnableLog(mode: u32, path: LPCSTR, attributes: u32) -> u32;

    #[link_name = "MsiInstallProductA"]
    fn MsiInstallProduct(packagePath: LPCSTR, commandLine: LPCSTR) -> u32;

    #[link_name = "MsiFormatRecordA"]
    fn MsiFormatRecord(
        install: MsiHandle,
        record: MsiHandle,
        value: LPSTR,
        value_len: *mut u32,
    ) -> u32;
}
