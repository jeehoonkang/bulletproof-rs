# Bulletproof load from memory

[![Build Status](https://travis-ci.org/jeehoonkang/bulletproof-rs.svg?branch=master)](https://travis-ci.org/jeehoonkang/bulletproof-rs)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/jeehoonkang/bulletproof-rs)
[![Cargo](https://img.shields.io/crates/v/bulletproof.svg)](https://crates.io/crates/bulletproof)
[![Documentation](https://docs.rs/bulletproof/badge.svg)](https://docs.rs/bulletproof)

You don't know whether a memory location is valid? Don't worry, Here's the bulletproof load!


## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
bulletproof = "0.1"
```

Next, use bulletproof loader as follows:

```rust
extern crate bulletproof;

fn main() {
    unsafe {
        let loader = bulletproof::Loader::new();
        assert_eq!(loader.load_usize(&42), Ok(42));
        assert_eq!(loader.load_usize(::std::ptr::null()), Err(()));

        assert_eq!(loader.load(&42u8), Ok(42));
        assert_eq!(loader.load::<[u8; 32]>(::std::ptr::null()), Err(()));
    }
}
```


## How?

Internally, `Loader::new()` installs a signal handler for segmentation fault (`SIGSEGV`), which
recovers from the fault using `siglongjmp()`.


## Why?

You PROBABLY should not use this library: instead of relying on bulletproof load, remove your
segmentation faults! However, if you want to build low-level systems such as virtual machine or
garbage collectors, bulletproof load can be a versatile tool for an additional bit of
efficiency. For example, see the [`ThreadCrashProtection`
class](http://hg.openjdk.java.net/jdk10/jdk10/hotspot/file/tip/src/os/posix/vm/os_posix.hpp#l115) in
Java HotSpot virtual machine.


## License

Licensed under the terms of MIT license and the Apache License (Version 2.0).

See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.
