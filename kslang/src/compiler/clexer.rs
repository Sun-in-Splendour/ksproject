use super::Source;
use std::{ffi::c_char, path::PathBuf};

macro_rules! set {
    ($n:ident, $v:expr) => {
        unsafe {
            $n = $v;
        }
    };
}

/// Source
#[repr(C)]
pub struct KSCSource;

pub type KSCSourceKind = usize;

// Source kinds
pub const KSC_SRC_STDIN: KSCSourceKind = 0;
pub const KSC_SRC_STRING: KSCSourceKind = 1;
pub const KSC_SRC_FILE: KSCSourceKind = 2;

pub type KSCSourceErr = usize;

// Error flag
static mut KSC_SRC_ERR: KSCSourceErr = 0;

pub const KSC_SRC_ERR_OK: KSCSourceErr = 0;
pub const KSC_SRC_ERR_EMPTY: KSCSourceErr = 1;
pub const KSC_SRC_ERR_UTF8: KSCSourceErr = 2;

/// # Safety
#[unsafe(no_mangle)]
pub unsafe extern "C" fn newKSCSource(
    src_type: KSCSourceKind,
    src_data: *const c_char,
    src_len: usize,
    src_path: *const c_char,
    src_path_len: usize,
) -> *const KSCSource {
    if src_len == 0 {
        set!(KSC_SRC_ERR, KSC_SRC_ERR_EMPTY);
        return std::ptr::null();
    }

    let src_bytes = unsafe { std::slice::from_raw_parts(src_data as *const u8, src_len) };
    let src_string = if let Ok(src_str) = std::str::from_utf8(src_bytes) {
        src_str.to_string()
    } else {
        set!(KSC_SRC_ERR, KSC_SRC_ERR_UTF8);
        return std::ptr::null();
    };

    let src = match src_type {
        KSC_SRC_STDIN => Source::Stdin(src_string),
        KSC_SRC_STRING => Source::String(src_string),
        KSC_SRC_FILE => {
            if src_path_len == 0 {
                set!(KSC_SRC_ERR, KSC_SRC_ERR_EMPTY);
                return std::ptr::null();
            }
            let src_path_bytes =
                unsafe { std::slice::from_raw_parts(src_path as *const u8, src_path_len) };
            let src_path_str = if let Ok(src_path_str) = std::str::from_utf8(src_path_bytes) {
                src_path_str.to_string()
            } else {
                set!(KSC_SRC_ERR, KSC_SRC_ERR_UTF8);
                return std::ptr::null();
            };
            let src_path = PathBuf::from(src_path_str);
            Source::File {
                path: src_path,
                contents: src_string,
            }
        }
        _ => return std::ptr::null(),
    };

    Box::into_raw(Box::new(src)) as *const KSCSource
}

/// # Safety
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getKSCSourceError() -> KSCSourceErr {
    unsafe { KSC_SRC_ERR }
}

/// # Safety
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getKSCSourceText(src: *const KSCSource) -> *const c_char {
    if src.is_null() {
        return std::ptr::null();
    }

    let src = unsafe { &*(src as *const Source) };
    match src {
        Source::Stdin(s) | Source::String(s) => s.as_ptr() as *const c_char,
        Source::File { contents, .. } => contents.as_ptr() as *const c_char,
    }
}

/// # Safety
#[unsafe(no_mangle)]
pub unsafe extern "C" fn freeKSCSource(src: *const KSCSource) {
    if !src.is_null() {
        unsafe { _ = Box::from_raw(src as *mut KSCSource) };
    }
}
