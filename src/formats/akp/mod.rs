// AKP format handling
// Support for both legacy parsing and new binrw-based parsing

#[cfg(feature = "legacy-parser")]
pub mod legacy;

#[cfg(feature = "binrw-parser")]
pub mod binrw_parser;

pub mod types;

// Re-export the appropriate parser based on features
#[cfg(all(feature = "legacy-parser", not(feature = "binrw-parser")))]
pub use legacy::*;

#[cfg(feature = "binrw-parser")]
pub use binrw_parser::*;

// Common types are always available
pub use types::*;