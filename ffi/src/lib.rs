//! Packages the C interface of `gbeed-core` (`gbeed_core::ffi`) as `libgbeed`.
//!
//! With the default `std` feature the emulator memories live in the heap. Without it the library
//! is `no_std` and heapless, for embedded targets, and provides its own panic handler.
//! The header is generated from `gbeed-core` into `ffi/include/gbeed.h`.
//!
//! A panic never unwinds into C: `extern "C"` functions abort when a panic reaches them,
//! and embedded targets already use `panic=abort`.

#![no_std]

#[cfg(feature = "std")]
extern crate std;

pub use gbeed_core::ffi::*;

/// No unwinding nor output available, halt
#[cfg(all(not(feature = "std"), not(test)))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { loop {} }
