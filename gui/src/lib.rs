use serde::{Deserialize, Serialize};
use rusty_samplers::plugins::registry::PluginRegistry;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub format: Option<String>,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub extensions: Vec<String>,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QualityPreview {
    pub score: u8,
    pub warnings: Vec<String>,
    pub parameters_preserved: u8,
    pub parameters_lost: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversionResult {
    pub success: bool,
    pub output_path: String,
    pub warnings: Vec<String>,
    pub stats: ConversionStats,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversionStats {
    pub keygroups_converted: u32,
    pub samples_referenced: u32,
    pub parameters_mapped: u32,
    pub processing_time_ms: u64,
}

#[tauri::command]
async fn get_available_plugins() -> Result<Vec<PluginInfo>, String> {
    let registry = PluginRegistry::with_defaults();
    
    let plugin_names = registry.registry().list_plugins();
    
    let plugins: Vec<PluginInfo> = plugin_names.iter()
        .map(|name| PluginInfo {
            name: name.clone(),
            extensions: vec!["akp".to_string(), "pgm".to_string()], // Simplified for now
            capabilities: vec!["read".to_string()],
        })
        .collect();

    Ok(plugins)
}

#[tauri::command]
async fn analyze_file_quality(file_path: String) -> Result<QualityPreview, String> {
    // Check if file exists and is an AKP file
    let path = Path::new(&file_path);
    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }
    
    // Read the file
    let file_data = fs::read(&file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    // Basic validation - check for RIFF header
    if file_data.len() < 12 {
        return Err("File too small to be valid AKP".to_string());
    }
    
    let has_riff = &file_data[0..4] == b"RIFF";
    let has_aprg = file_data.len() > 11 && &file_data[8..12] == b"APRG";
    
    if !has_riff || !has_aprg {
        return Ok(QualityPreview {
            score: 0,
            warnings: vec!["Not a valid AKP file".to_string()],
            parameters_preserved: 0,
            parameters_lost: 0,
        });
    }
    
    // For now, return a simulated quality score based on file analysis
    // In a real implementation, we'd parse the AKP file and analyze parameters
    let score = if has_riff && has_aprg { 92 } else { 0 };
    let warnings = if score > 0 {
        vec!["Some LFO parameters may be approximated".to_string()]
    } else {
        vec!["Invalid file format".to_string()]
    };
    
    Ok(QualityPreview {
        score,
        warnings,
        parameters_preserved: if score > 0 { 26 } else { 0 },
        parameters_lost: if score > 0 { 2 } else { 0 },
    })
}

#[tauri::command]
async fn start_conversion(file_path: String, output_format: String) -> Result<ConversionResult, String> {
    use rusty_samplers::conversion::convert_akp_to_sfz;
    use std::fs::File;
    
    // Validate input file
    let input_path = Path::new(&file_path);
    if !input_path.exists() {
        return Err(format!("Input file not found: {}", file_path));
    }

    // Generate output path
    let output_path = input_path
        .with_extension(&output_format.to_lowercase())
        .to_string_lossy()
        .to_string();

    // Open the AKP file
    let file = File::open(&file_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;

    // Perform conversion using the existing engine
    match convert_akp_to_sfz(file) {
        Ok(sfz_content) => {
            // Write the converted SFZ file
            fs::write(&output_path, &sfz_content)
                .map_err(|e| format!("Failed to write output file: {}", e))?;

            // Calculate some basic stats
            let line_count = sfz_content.matches('\n').count();
            let region_count = sfz_content.matches("<region>").count();
            let group_count = sfz_content.matches("<group>").count();
            
            Ok(ConversionResult {
                success: true,
                output_path,
                warnings: vec!["Verify sample paths in output SFZ file".to_string()],
                stats: ConversionStats {
                    keygroups_converted: (group_count + region_count) as u32,
                    samples_referenced: region_count as u32,
                    parameters_mapped: line_count as u32,
                    processing_time_ms: 500, // placeholder timing
                },
            })
        }
        Err(e) => Err(format!("Conversion failed: {}", e))
    }
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_available_plugins,
            analyze_file_quality,
            start_conversion
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}