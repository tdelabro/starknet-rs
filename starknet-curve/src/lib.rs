#![cfg_attr(not(feature = "std"), no_std)]
#![doc = include_str!("../README.md")]

#[cfg(feature = "std")]
include!("./with_std.rs");

#[cfg(not(feature = "std"))]
include!("./without_std.rs");

mod ec_point;

pub mod curve_params;

pub use ec_point::{AffinePoint, ProjectivePoint};
