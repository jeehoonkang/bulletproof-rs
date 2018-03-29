# Bulletproof memory access

[![Build Status](https://travis-ci.org/jeehoonkang/bulletproof-rs.svg?branch=master)](https://travis-ci.org/jeehoonkang/bulletproof-rs)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/jeehoonkang/bulletproof-rs)
[![Cargo](https://img.shields.io/crates/v/bulletproof.svg)](https://crates.io/crates/bulletproof)
[![Documentation](https://docs.rs/bulletproof/badge.svg)](https://docs.rs/bulletproof)

You don't know whether a memory location is valid? Don't worry, Here comes the bulletproof memory
access!


## Examples

Add this to your `Cargo.toml`:

```toml
[dependencies]
bulletproof = "0.2"
```

Next, enjoy bulletproof memory access as follows:

```rust
use bulletproof::Bulletproof;
use std::ptr;

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
```


## How?

Internally, `Bulletproof::new()` installs a signal handler for segmentation fault (`SIGSEGV`), which
recovers from the fault using `siglongjmp()`.


## Safe?

Even if a location is deallocated, it may still be accessible because it is not returned to the OS
yet.

Since `Bulletproof::new()` registers a `SIGSEGV` signal handler, it may disrupt the existing or
future signal handlers. Most notably, [Rust installs a `SIGSEGV` signal
handler](https://github.com/rust-lang/rust/blob/e7e982ac03b496dd4d4b5c182fdcd5fb4f2b5470/src/libstd/sys/unix/stack_overflow.rs#L76)
for protecting stack from overflow at initialization. By creating a `Bulletproof`, stack is no
longer protected.


## Why?

You PROBABLY should not use this library: instead of relying on bulletproof access, remove your
segmentation faults! However, if you want to build low-level systems such as virtual machine or
garbage collectors, bulletproof load can be a versatile tool for an additional bit of
efficiency. For example, see [the `ThreadCrashProtection`
class](http://hg.openjdk.java.net/jdk10/jdk10/hotspot/file/tip/src/os/posix/vm/os_posix.hpp#l115) in
Java HotSpot virtual machine.


## License

Licensed under the terms of MIT license and the Apache License (Version 2.0).

See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.
