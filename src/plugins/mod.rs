// Multi-format plugin architecture for Rusty Samplers
// Extensible system for supporting various sampler formats

pub mod traits;
pub mod registry;
pub mod akp;

#[cfg(feature = "pgm-plugin")]
pub mod pgm;

// Re-export commonly used types
pub use traits::{FormatPlugin, FormatReader, FormatWriter, FormatCapabilities};
pub use registry::{FormatRegistry, PluginRegistry};