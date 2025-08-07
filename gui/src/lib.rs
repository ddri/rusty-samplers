use serde::{Deserialize, Serialize};
use rusty_samplers::plugins::registry::PluginRegistry;

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
async fn analyze_file_quality(_file_path: String) -> Result<QualityPreview, String> {
    // This is a placeholder - we'll implement real quality analysis
    Ok(QualityPreview {
        score: 95,
        warnings: vec!["LFO rate may be approximated".to_string()],
        parameters_preserved: 28,
        parameters_lost: 2,
    })
}

#[tauri::command]
async fn start_conversion(input_path: String, _output_path: String, format: String) -> Result<String, String> {
    // This is a placeholder - we'll implement real conversion
    Ok(format!("Converted {} to {} format", input_path, format))
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