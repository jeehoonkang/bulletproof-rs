//! Bulletproof load from memory.
//!
//! You don't know whether a memory location is valid? Don't worry, Here's the bulletproof load!
//!
//! # Examples
//!
//! ```
//! use bulletproof::Loader;
//!
//! unsafe {
//!     let loader = Loader::new();
//!     assert_eq!(loader.load_usize(&42), Ok(42));
//!     assert_eq!(loader.load_usize(::std::ptr::null()), Err(()));
//!
//!     assert_eq!(loader.load(&42u8), Ok(42));
//!     assert_eq!(loader.load::<[usize; 32]>(::std::ptr::null()), Err(()));
//! }
//! ```
//!
//! # How?
//!
//! Internally, `Loader::new()` installs a signal handler for segmentation fault (`SIGSEGV`), which
//! recovers from the fault using `longjmp()`.
//!
//! # Why?
//!
//! You PROBABLY should not use this library: instead of relying on bulletproof load, remove your
//! segmentation faults! However, if you want to build low-level systems such as virtual machine or
//! garbage collectors, bulletproof load can be a versatile tool for an additional bit of
//! efficiency. For example, see the `ThreadCrashProtection` class in Java HotSpot virtual machine:
//! http://hg.openjdk.java.net/jdk10/jdk10/hotspot/file/tip/src/os/posix/vm/os_posix.hpp#l115

#![warn(missing_docs, missing_debug_implementations)]

extern crate libc;

use std::marker::PhantomData;
use std::fmt;
use std::mem;

use libc::{size_t, c_void};

extern {
    fn bulletproof_section_begin() -> size_t;
    fn bulletproof_section_end() -> size_t;
    fn bulletproof_load(from: *const size_t, to: *mut size_t) -> size_t;
    fn bulletproof_load_bytes(from: *const c_void, to: *mut c_void, size: size_t) -> size_t;
}

/// Bulletproof loader.
pub struct Loader {
    _marker: PhantomData<*mut ()>, // !Send + !Sync
}

impl fmt::Debug for Loader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Loader").finish()
    }
}

impl Loader {
    /// Creates a new bulletproof loader.
    ///
    /// # Safety
    ///
    /// You should not change the `SIGSEGV` handler during the lifetime of a `Loader`.
    pub unsafe fn new() -> Self {
        assert_eq!(
            bulletproof_section_begin(),
            0,
            "bulletproof_section_begin() failed",
        );

        Self {
            _marker: PhantomData,
        }
    }

    /// Loads a usize from the location.
    ///
    /// Returns `Ok(v)` if `location` contains `v`, and `Err(())` if the location is invalid.
    pub fn load_usize(&self, location: *const usize) -> Result<usize, ()> {
        unsafe {
            let mut result: usize = mem::uninitialized();
            if bulletproof_load(location, &mut result) != 0 {
                return Err(());
            }

            Ok(result)
        }
    }

    /// Loads a value of type `T` from the location.
    ///
    /// Returns `Ok(v)` if `location` contains `v`, and `Err(())` if the location is invalid.
    ///
    /// # Safety
    ///
    /// If `location` is valid, it should contain a valid value of `T`.
    pub unsafe fn load<T>(&self, location: *const T) -> Result<T, ()> {
        let mut result: T = mem::uninitialized();
        let buffer = &mut result as *mut T as *mut c_void;
        if bulletproof_load_bytes(
            location as *const c_void,
            buffer,
            mem::size_of::<T>(),
        ) != 0 {
            return Err(());
        }

        Ok(result)
    }
}

impl Drop for Loader {
    fn drop(&mut self) {
        assert_eq!(
            unsafe { bulletproof_section_end() },
            0,
            "bulletproof_section_end() failed",
        );
    }
}

#[cfg(test)]
mod tests {
    use std::ptr;
    use super::*;

    #[test]
    fn bulletproof() {
        unsafe {
            // a loader.
            let loader1 = Loader::new();
            assert_eq!(loader1.load_usize(&42), Ok(42));
            assert_eq!(loader1.load_usize(ptr::null()), Err(()));

            // a nested loader.
            let loader2 = Loader::new();
            assert_eq!(loader1.load_usize(&42), Ok(42));
            assert_eq!(loader1.load_usize(ptr::null()), Err(()));
            assert_eq!(loader2.load(&42u8), Ok(42));
            assert_eq!(loader2.load::<[usize; 32]>(ptr::null()), Err(()));
        }
    }
}
