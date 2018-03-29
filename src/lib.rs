//! Bulletproof memory access.
//!
//! You don't know whether a memory location is valid? Don't worry, Here comes the bulletproof
//! memory access!
//!
//! # Examples
//!
//! ```
//! use bulletproof::Bulletproof;
//! use std::ptr;
//!
//! let mut x = 42usize;
//! let y = 42u8;
//!
//! unsafe {
//!     let bulletproof = Bulletproof::new();
//!
//!     assert_eq!(bulletproof.load_usize(&x), Ok(42));
//!     assert_eq!(bulletproof.load_usize(ptr::null()), Err(()));
//!
//!     assert_eq!(bulletproof.store_usize(&mut x, 37), Ok(()));
//!     assert_eq!(bulletproof.store_usize(ptr::null_mut(), 37), Err(()));
//!     assert_eq!(bulletproof.load_usize(&x), Ok(37));
//!     assert_eq!(ptr::read(&x), 37);
//!
//!     assert_eq!(bulletproof.load(&y), Ok(42));
//!     assert_eq!(bulletproof.load::<[usize; 32]>(ptr::null()), Err(()));
//! }
//! ```
//!
//! # How?
//!
//! Internally, `Bulletproof::new()` installs a signal handler for segmentation fault (`SIGSEGV`),
//! which recovers from the fault using `siglongjmp()`.
//!
//! # Safe?
//!
//! Even if a location is deallocated, it may still be accessible because it is not returned to the
//! OS yet.
//!
//! Since `Bulletproof::new()` registers a `SIGSEGV` signal handler, it may disrupt the existing or
//! future signal handlers. Most notably, [Rust installs a `SIGSEGV` signal
//! handler](https://github.com/rust-lang/rust/blob/e7e982ac03b496dd4d4b5c182fdcd5fb4f2b5470/src/libstd/sys/unix/stack_overflow.rs#L76)
//! for protecting stack from overflow at initialization. By creating a `Bulletproof`, stack is no
//! longer protected.
//!
//! # Why?
//!
//! You PROBABLY should not use this library: instead of relying on bulletproof access, remove your
//! segmentation faults! However, if you want to build low-level systems such as virtual machine or
//! garbage collectors, bulletproof load can be a versatile tool for an additional bit of
//! efficiency. For example, see [the `ThreadCrashProtection`
//! class](http://hg.openjdk.java.net/jdk10/jdk10/hotspot/file/tip/src/os/posix/vm/os_posix.hpp#l115)
//! in Java HotSpot virtual machine.

#![warn(missing_docs, missing_debug_implementations)]

extern crate libc;

use std::mem;

use libc::{size_t, c_void};

extern {
    fn bulletproof_register() -> size_t;
    fn bulletproof_load(loc: *const size_t, dst: *mut size_t) -> size_t;
    fn bulletproof_store(loc: *const size_t, val: size_t) -> size_t;
    fn bulletproof_load_bytes(loc: *const c_void, dst: *mut c_void, size: size_t) -> size_t;
    fn bulletproof_store_bytes(loc: *mut c_void, src: *const c_void, size: size_t) -> size_t;
}

/// Bulletproof loader.
#[derive(Debug, Clone, Copy)]
pub struct Bulletproof {}

impl Bulletproof {
    /// Creates a new bulletproof memory access manager.
    ///
    /// # Safety
    ///
    /// It registers a new signal handler for `SIGSEGV`. See [`README.md`](/README.md) for more
    /// details on its impact.
    #[inline]
    pub unsafe fn new() -> Self {
        assert_eq!(
            bulletproof_register(),
            0,
            "bulletproof_register() failed",
        );
        Self {}
    }

    /// Loads a usize from the location.
    ///
    /// Returns `Ok(v)` if `location` contains `v`, and `Err(())` if the location is invalid.
    ///
    /// # Safety
    ///
    /// The location should satisfy the safety guarantee of
    /// [`std::ptr::read()`](https://doc.rust-lang.org/stable/std/ptr/fn.read.html), except that it
    /// can be an invalid pointer.
    #[inline]
    pub unsafe fn load_usize(self, location: *const usize) -> Result<usize, ()> {
        let mut result: usize = mem::uninitialized();
        if bulletproof_load(location, &mut result) != 0 {
            return Err(());
        }

        Ok(result)
    }

    /// Loads a value of type `T` from the location.
    ///
    /// Returns `Ok(v)` if `location` contains `v`, and `Err(())` if the location is invalid.
    ///
    /// # Safety
    ///
    /// The location should satisfy the safety guarantee of
    /// [`std::ptr::read()`](https://doc.rust-lang.org/stable/std/ptr/fn.read.html), except that it
    /// can be an invalid pointer.
    #[inline]
    pub unsafe fn load<T>(self, location: *const T) -> Result<T, ()> {
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

    /// Stores a usize to the location.
    ///
    /// Returns `Ok(v)` if `location` contains `v`, and `Err(())` if the location is invalid.
    ///
    /// # Safety
    ///
    /// The location should satisfy the safety guarantee of
    /// [`std::ptr::write()`](https://doc.rust-lang.org/stable/std/ptr/fn.write.html), except that
    /// it can be an invalid pointer.
    #[inline]
    pub unsafe fn store_usize(self, location: *mut usize, val: usize) -> Result<(), ()> {
        if bulletproof_store(location, val) != 0 {
            return Err(());
        }

        Ok(())
    }

    /// Stores a value of type `T` to the location.
    ///
    /// Returns `Ok(())` if `location` is valid, and `Err(())` if the location is invalid.
    ///
    /// # Safety
    ///
    /// The location should satisfy the safety guarantee of
    /// [`std::ptr::write()`](https://doc.rust-lang.org/stable/std/ptr/fn.write.html), except that
    /// it can be an invalid pointer.
    #[inline]
    pub unsafe fn store<T>(self, location: *mut T, src: &T) -> Result<(), ()> {
        if bulletproof_store_bytes(
            location as *mut c_void,
            src as *const T as *const c_void,
            mem::size_of::<T>(),
        ) != 0 {
            return Err(());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::ptr;
    use super::*;

    #[test]
    fn bulletproof() {
        let mut x = 42usize;
        let y = 42u8;

        unsafe {
            let bulletproof = Bulletproof::new();

            assert_eq!(bulletproof.load_usize(&x), Ok(42));
            assert_eq!(bulletproof.load_usize(ptr::null()), Err(()));

            assert_eq!(bulletproof.store_usize(&mut x, 37), Ok(()));
            assert_eq!(bulletproof.store_usize(ptr::null_mut(), 37), Err(()));
            assert_eq!(bulletproof.load_usize(&x), Ok(37));
            assert_eq!(ptr::read(&x), 37);

            assert_eq!(bulletproof.load(&y), Ok(42));
            assert_eq!(bulletproof.load::<[usize; 32]>(ptr::null()), Err(()));
        }
    }
}
