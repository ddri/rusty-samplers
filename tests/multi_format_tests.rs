// Multi-format plugin system integration tests
// Validates that multiple format plugins work together

use rusty_samplers::plugins::{PluginRegistry, FormatRegistry, FormatPlugin};

#[test]
fn test_multi_format_plugin_loading() {
    let plugin_registry = PluginRegistry::with_defaults();
    let registry = plugin_registry.registry();
    
    let plugins = registry.list_plugins();
    println!("Loaded plugins: {:?}", plugins);
    
    // Should have both AKP and PGM plugins loaded
    #[cfg(feature = "akp-plugin")]
    assert!(plugins.contains(&"akp".to_string()), "AKP plugin should be loaded");
    
    #[cfg(feature = "pgm-plugin")]
    assert!(plugins.contains(&"pgm".to_string()), "PGM plugin should be loaded");
    
    // Should have at least 1 plugin (could be more depending on features)
    assert!(!plugins.is_empty(), "At least one plugin should be loaded");
}

#[test]
fn test_format_detection_with_multiple_plugins() {
    let registry = FormatRegistry::new();
    
    // Register plugins manually for testing
    #[cfg(feature = "akp-plugin")]
    {
        use std::sync::Arc;
        let akp_plugin = rusty_samplers::plugins::akp::AkpPlugin::new().unwrap();
        registry.register_plugin(Arc::new(akp_plugin)).unwrap();
    }
    
    #[cfg(feature = "pgm-plugin")]
    {
        use std::sync::Arc;
        let pgm_plugin = rusty_samplers::plugins::pgm::PgmPlugin::new().unwrap();
        registry.register_plugin(Arc::new(pgm_plugin)).unwrap();
    }
    
    // Test AKP detection
    #[cfg(feature = "akp-plugin")]
    {
        let akp_data = b"RIFF\x00\x00\x00\x00APRG";
        let detected = registry.detect_format(akp_data);
        assert_eq!(detected, Some("akp".to_string()));
    }
    
    // Test PGM detection (using our test data)
    #[cfg(feature = "pgm-plugin")]
    {
        let mut pgm_data = vec![
            0x01, 0x00, 0x10, 0x00, // Version 1, 16 pads
            b'M', b'P', b'C', b' ', b'P', b'r', b'o', b'g', // "MPC Prog"
            b'r', b'a', b'm', 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        pgm_data.resize(64, 0);
        
        let detected = registry.detect_format(&pgm_data);
        assert_eq!(detected, Some("pgm".to_string()));
    }
}

#[test]
fn test_plugin_capabilities_comparison() {
    #[cfg(all(feature = "akp-plugin", feature = "pgm-plugin"))]
    {
        let akp_plugin = rusty_samplers::plugins::akp::AkpPlugin::new().unwrap();
        let pgm_plugin = rusty_samplers::plugins::pgm::PgmPlugin::new().unwrap();
        
        let akp_caps = akp_plugin.capabilities();
        let pgm_caps = pgm_plugin.capabilities();
        
        // Compare capabilities
        println!("AKP capabilities: {:?}", akp_caps);
        println!("PGM capabilities: {:?}", pgm_caps);
        
        // Both should support reading
        assert!(akp_caps.can_read);
        assert!(pgm_caps.can_read);
        
        // AKP should have higher quality rating (more mature)
        assert!(akp_caps.quality_rating >= pgm_caps.quality_rating);
        
        // Different max samples
        assert_eq!(akp_caps.max_samples, 128); // Akai S5000/S6000 limit
        assert_eq!(pgm_caps.max_samples, 64);  // MPC2000XL limit
        
        // Different format support
        assert!(akp_caps.supports_velocity_layers);  // Akai supports velocity layers
        assert!(!pgm_caps.supports_velocity_layers); // MPC doesn't use velocity layers
    }
}

#[test]
fn test_registry_statistics_with_multiple_plugins() {
    let plugin_registry = PluginRegistry::with_defaults();
    let registry = plugin_registry.registry();
    
    let stats = registry.stats();
    println!("Registry stats: {}", stats.summary());
    
    // Should have plugins loaded
    assert!(stats.total_plugins > 0);
    assert!(stats.readers > 0);
    
    // Should have reasonable quality average
    assert!(stats.average_quality >= 3.0);
    
    // Should support extensions
    #[cfg(feature = "akp-plugin")]
    {
        let akp_plugins = registry.plugins_for_extension("akp");
        assert!(!akp_plugins.is_empty());
    }
    
    #[cfg(feature = "pgm-plugin")]
    {
        let pgm_plugins = registry.plugins_for_extension("pgm");
        assert!(!pgm_plugins.is_empty());
    }
}

#[test]
fn test_supported_formats_api() {
    let plugin_registry = PluginRegistry::with_defaults();
    
    let input_formats = plugin_registry.supported_input_formats();
    let output_formats = plugin_registry.supported_output_formats();
    
    println!("Supported input formats: {:?}", input_formats);
    println!("Supported output formats: {:?}", output_formats);
    
    // Should have at least one input format
    assert!(!input_formats.is_empty());
    
    // Check for expected formats
    #[cfg(feature = "akp-plugin")]
    assert!(input_formats.contains(&"akp".to_string()));
    
    #[cfg(feature = "pgm-plugin")]
    assert!(input_formats.contains(&"pgm".to_string()));
}

#[test]
fn test_plugin_registry_info() {
    let plugin_registry = PluginRegistry::with_defaults();
    let info = plugin_registry.info();
    
    println!("Plugin registry info:\n{}", info);
    
    // Should contain registry information
    assert!(info.contains("Registry:"));
    assert!(info.contains("Input formats:"));
    assert!(info.contains("Output formats:"));
}