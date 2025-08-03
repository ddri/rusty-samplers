// Rusty Samplers Library
// High-performance AKP to SFZ converter

//! # Rusty Samplers
//!
//! A high-performance AKP to SFZ converter written in Rust for musicians and audio producers
//! working with Akai and SFZ-compatible samplers.
//!
//! ## Features
//!
//! - Professional-grade conversion with full parameter preservation
//! - Robust error handling with graceful degradation for malformed files
//! - High-performance streaming architecture for large files
//! - Industry-standard parameter conversion algorithms
//!
//! ## Example
//!
//! ```rust,no_run
//! use rusty_samplers::conversion::convert_akp_to_sfz;
//! use std::fs::File;
//!
//! let input = File::open("program.akp")?;
//! let sfz_content = convert_akp_to_sfz(input)?;
//! std::fs::write("program.sfz", sfz_content)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod error;
pub mod formats;
pub mod conversion;

// Re-export commonly used types
pub use error::{ConversionError, Result};
pub use formats::common::*;