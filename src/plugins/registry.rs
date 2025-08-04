// Plugin registry for automatic format detection and management
// Handles plugin registration, discovery, and format routing

use super::traits::{FormatPlugin, FormatReader, FormatWriter, InternalProgram};
use crate::error::{ConversionError, Result};
use std::collections::HashMap;
use std::io::Read;
use std::sync::{Arc, RwLock};
use log::{info, debug, warn};

/// Thread-safe plugin registry for format management
pub struct FormatRegistry {
    plugins: RwLock<HashMap<String, Arc<dyn FormatPlugin>>>,
    extension_map: RwLock<HashMap<String, Vec<String>>>, // extension -> plugin names
    magic_bytes_map: RwLock<Vec<(Vec<u8>, String)>>,    // (magic bytes, plugin name)
}

impl FormatRegistry {
    /// Create a new format registry
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
            extension_map: RwLock::new(HashMap::new()),
            magic_bytes_map: RwLock::new(Vec::new()),
        }
    }

    /// Register a format plugin
    pub fn register_plugin(&self, plugin: Arc<dyn FormatPlugin>) -> Result<()> {
        let name = plugin.name().to_string();
        
        info!("Registering plugin: {} v{}", name, plugin.version());
        debug!("Plugin capabilities: {:?}", plugin.capabilities());

        // Store plugin
        {
            let mut plugins = self.plugins.write().unwrap();
            if plugins.contains_key(&name) {
                warn!("Plugin '{}' already registered, replacing", name);
            }
            plugins.insert(name.clone(), plugin.clone());
        }

        // Register file extensions
        {
            let mut ext_map = self.extension_map.write().unwrap();
            for &extension in plugin.file_extensions() {
                let ext_lower = extension.to_lowercase();
                ext_map.entry(ext_lower).or_insert_with(Vec::new).push(name.clone());
            }
        }

        // Register magic bytes
        if let Some(magic) = plugin.magic_bytes() {
            let mut magic_map = self.magic_bytes_map.write().unwrap();
            magic_map.push((magic.to_vec(), name.clone()));
        }

        info!("Successfully registered plugin: {}", name);
        Ok(())
    }

    /// Get all registered plugin names
    pub fn list_plugins(&self) -> Vec<String> {
        let plugins = self.plugins.read().unwrap();
        plugins.keys().cloned().collect()
    }

    /// Get plugin by name
    pub fn get_plugin(&self, name: &str) -> Option<Arc<dyn FormatPlugin>> {
        let plugins = self.plugins.read().unwrap();
        plugins.get(name).cloned()
    }

    /// Auto-detect format from file data
    pub fn detect_format(&self, data: &[u8]) -> Option<String> {
        debug!("Attempting format detection on {} bytes", data.len());

        // Try magic bytes first (most reliable)
        {
            let magic_map = self.magic_bytes_map.read().unwrap();
            for (magic_bytes, plugin_name) in magic_map.iter() {
                if data.starts_with(magic_bytes) {
                    debug!("Format detected via magic bytes: {}", plugin_name);
                    return Some(plugin_name.clone());
                }
            }
        }

        // Fallback: try plugin-specific detection
        let plugins = self.plugins.read().unwrap();
        for (name, plugin) in plugins.iter() {
            if plugin.can_handle(data) {
                debug!("Format detected via plugin logic: {}", name);
                return Some(name.clone());
            }
        }

        debug!("No format detection succeeded");
        None
    }

    /// Get plugins by file extension
    pub fn plugins_for_extension(&self, extension: &str) -> Vec<String> {
        let ext_map = self.extension_map.read().unwrap();
        ext_map.get(&extension.to_lowercase())
            .cloned()
            .unwrap_or_default()
    }

    /// Get format reader for a plugin
    pub fn get_reader(&self, plugin_name: &str) -> Result<Box<dyn FormatReader>> {
        let plugin = self.get_plugin(plugin_name)
            .ok_or_else(|| ConversionError::Custom {
                message: format!("Plugin '{}' not found", plugin_name),
            })?;

        plugin.reader()
            .ok_or_else(|| ConversionError::Custom {
                message: format!("Plugin '{}' does not support reading", plugin_name),
            })
    }

    /// Get format writer for a plugin
    pub fn get_writer(&self, plugin_name: &str) -> Result<Box<dyn FormatWriter>> {
        let plugin = self.get_plugin(plugin_name)
            .ok_or_else(|| ConversionError::Custom {
                message: format!("Plugin '{}' not found", plugin_name),
            })?;

        plugin.writer()
            .ok_or_else(|| ConversionError::Custom {
                message: format!("Plugin '{}' does not support writing", plugin_name),
            })
    }

    /// Get registry statistics
    pub fn stats(&self) -> RegistryStats {
        let plugins = self.plugins.read().unwrap();
        let ext_map = self.extension_map.read().unwrap();
        let magic_map = self.magic_bytes_map.read().unwrap();

        let mut readers = 0;
        let mut writers = 0;
        let mut quality_scores = Vec::new();

        for plugin in plugins.values() {
            let caps = plugin.capabilities();
            if caps.can_read { readers += 1; }
            if caps.can_write { writers += 1; }
            quality_scores.push(caps.quality_rating);
        }

        let avg_quality = if quality_scores.is_empty() {
            0.0
        } else {
            quality_scores.iter().sum::<u8>() as f64 / quality_scores.len() as f64
        };

        RegistryStats {
            total_plugins: plugins.len(),
            readers,
            writers,
            supported_extensions: ext_map.len(),
            magic_byte_detectors: magic_map.len(),
            average_quality: avg_quality,
        }
    }
}

impl Default for FormatRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry statistics
#[derive(Debug)]
pub struct RegistryStats {
    pub total_plugins: usize,
    pub readers: usize,
    pub writers: usize,
    pub supported_extensions: usize,
    pub magic_byte_detectors: usize,
    pub average_quality: f64,
}

impl RegistryStats {
    pub fn summary(&self) -> String {
        format!(
            "Registry: {} plugins ({} readers, {} writers), {} extensions, {} magic detectors, {:.1} avg quality",
            self.total_plugins,
            self.readers,
            self.writers,
            self.supported_extensions,
            self.magic_byte_detectors,
            self.average_quality
        )
    }
}

/// High-level plugin registry for the entire application
pub struct PluginRegistry {
    registry: FormatRegistry,
}

impl PluginRegistry {
    /// Create registry and load default plugins
    pub fn with_defaults() -> Self {
        let registry = FormatRegistry::new();
        
        // Auto-register built-in plugins based on features
        #[cfg(feature = "akp-plugin")]
        {
            if let Ok(akp_plugin) = crate::plugins::akp::AkpPlugin::new() {
                let _ = registry.register_plugin(Arc::new(akp_plugin));
            }
        }

        #[cfg(feature = "pgm-plugin")]
        {
            if let Ok(pgm_plugin) = crate::plugins::pgm::PgmPlugin::new() {
                let _ = registry.register_plugin(Arc::new(pgm_plugin));
            }
        }

        Self { registry }
    }

    /// Convert between formats using plugin system
    pub fn convert(
        &self,
        data: &[u8],
        source_format: Option<&str>,
    ) -> Result<InternalProgram> {
        // Detect source format if not specified
        let source_plugin_name = if let Some(format) = source_format {
            format.to_string()
        } else {
            self.registry.detect_format(data)
                .ok_or_else(|| ConversionError::Custom {
                    message: "Could not detect source format".to_string(),
                })?
        };

        // Get reader and perform conversion
        let format_reader = self.registry.get_reader(&source_plugin_name)?;
        let program = format_reader.read(data)?;

        // Validate the program
        program.validate()?;

        info!("Converted from {} to internal format: {}", 
              source_plugin_name, program.stats());

        Ok(program)
    }

    /// Write internal program to specific format
    pub fn write_format(
        &self,
        program: &InternalProgram,
        target_format: &str,
    ) -> Result<Vec<u8>> {
        let format_writer = self.registry.get_writer(target_format)?;
        
        // Validate program can be written to target format
        format_writer.can_write(program)?;
        
        // Get conversion info/warnings
        let warnings = format_writer.conversion_info(program);
        if !warnings.is_empty() {
            info!("Conversion warnings for {}: {:?}", target_format, warnings);
        }

        let data = format_writer.write(program)?;
        
        info!("Successfully wrote program to {} format", target_format);
        Ok(data)
    }

    /// Get supported input formats
    pub fn supported_input_formats(&self) -> Vec<String> {
        self.registry.list_plugins().into_iter()
            .filter(|name| {
                self.registry.get_plugin(name)
                    .map(|p| p.capabilities().can_read)
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Get supported output formats
    pub fn supported_output_formats(&self) -> Vec<String> {
        self.registry.list_plugins().into_iter()
            .filter(|name| {
                self.registry.get_plugin(name)
                    .map(|p| p.capabilities().can_write)
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Get registry for direct access
    pub fn registry(&self) -> &FormatRegistry {
        &self.registry
    }

    /// Get comprehensive registry information
    pub fn info(&self) -> String {
        let stats = self.registry.stats();
        let input_formats = self.supported_input_formats();
        let output_formats = self.supported_output_formats();

        format!(
            "{}\nInput formats: {}\nOutput formats: {}",
            stats.summary(),
            input_formats.join(", "),
            output_formats.join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::traits::{FormatCapabilities, FormatPlugin};

    // Mock plugin for testing
    struct MockPlugin {
        name: String,
        extensions: Vec<&'static str>,
        magic: Option<Vec<u8>>,
    }

    impl MockPlugin {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                extensions: vec!["mock"],
                magic: Some(b"MOCK".to_vec()),
            }
        }
    }

    impl FormatPlugin for MockPlugin {
        fn name(&self) -> &str { &self.name }
        fn version(&self) -> &str { "1.0.0" }
        fn description(&self) -> &str { "Mock plugin for testing" }
        fn file_extensions(&self) -> &[&str] { &self.extensions }
        fn magic_bytes(&self) -> Option<&[u8]> { 
            self.magic.as_ref().map(|v| v.as_slice()) 
        }
        fn capabilities(&self) -> FormatCapabilities {
            FormatCapabilities {
                can_read: true,
                can_write: false,
                quality_rating: 4,
                ..Default::default()
            }
        }
    }

    #[test]
    fn test_registry_creation() {
        let registry = FormatRegistry::new();
        assert_eq!(registry.list_plugins().len(), 0);
    }

    #[test]
    fn test_plugin_registration() {
        let registry = FormatRegistry::new();
        let plugin = Arc::new(MockPlugin::new("test"));
        
        assert!(registry.register_plugin(plugin).is_ok());
        assert_eq!(registry.list_plugins().len(), 1);
        assert!(registry.get_plugin("test").is_some());
    }

    #[test]
    fn test_format_detection() {
        let registry = FormatRegistry::new();
        let plugin = Arc::new(MockPlugin::new("mock_format"));
        registry.register_plugin(plugin).unwrap();

        let data = b"MOCKtest data";
        let detected = registry.detect_format(data);
        assert_eq!(detected, Some("mock_format".to_string()));

        let invalid_data = b"INVALID";
        let not_detected = registry.detect_format(invalid_data);
        assert_eq!(not_detected, None);
    }

    #[test]
    fn test_extension_mapping() {
        let registry = FormatRegistry::new();
        let plugin = Arc::new(MockPlugin::new("mock_format"));
        registry.register_plugin(plugin).unwrap();

        let plugins = registry.plugins_for_extension("mock");
        assert_eq!(plugins, vec!["mock_format"]);

        let no_plugins = registry.plugins_for_extension("unknown");
        assert!(no_plugins.is_empty());
    }

    #[test]
    fn test_registry_stats() {
        let registry = FormatRegistry::new();
        let plugin = Arc::new(MockPlugin::new("test"));
        registry.register_plugin(plugin).unwrap();

        let stats = registry.stats();
        assert_eq!(stats.total_plugins, 1);
        assert_eq!(stats.readers, 1);
        assert_eq!(stats.writers, 0);
        assert_eq!(stats.supported_extensions, 1);
        assert_eq!(stats.magic_byte_detectors, 1);
        assert_eq!(stats.average_quality, 4.0);
    }

    #[test]
    fn test_plugin_registry_creation() {
        let plugin_registry = PluginRegistry::with_defaults();
        let info = plugin_registry.info();
        assert!(info.contains("Registry:"));
    }
}