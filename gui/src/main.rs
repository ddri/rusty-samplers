use eframe::egui;
use rfd::FileDialog;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use rusty_samplers::OutputFormat;

#[derive(Default)]
pub struct RustySamplersApp {
    // File selection
    selected_files: Vec<PathBuf>,
    output_format: OutputFormat,

    // UI state
    conversion_status: String,
    is_converting: bool,
    show_about: bool,

    // Progress tracking
    progress_receiver: Option<mpsc::Receiver<ConversionProgress>>,
    conversion_results: Vec<ConversionResult>,
    current_progress: f32,

    // Batch mode
    batch_mode: bool,
    output_directory: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub enum ConversionProgress {
    Started(String),
    Progress(String, f32),
    FileResult(ConversionResult),
    Completed(String),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct ConversionResult {
    pub input_file: PathBuf,
    pub output_file: Option<PathBuf>,
    pub success: bool,
    pub message: String,
}

impl eframe::App for RustySamplersApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for progress updates
        if let Some(receiver) = &self.progress_receiver {
            while let Ok(progress) = receiver.try_recv() {
                match progress {
                    ConversionProgress::Started(msg) => {
                        self.conversion_status = msg;
                        self.is_converting = true;
                        self.current_progress = 0.0;
                    }
                    ConversionProgress::Progress(msg, progress) => {
                        self.conversion_status = msg;
                        self.current_progress = progress;
                    }
                    ConversionProgress::FileResult(result) => {
                        self.conversion_results.push(result);
                    }
                    ConversionProgress::Completed(msg) => {
                        self.conversion_status = msg;
                        self.is_converting = false;
                        self.current_progress = 1.0;
                    }
                    ConversionProgress::Error(msg) => {
                        self.conversion_status = format!("Error: {}", msg);
                        self.is_converting = false;
                    }
                }
            }
        }

        // Handle drag & drop
        ctx.input(|i| {
            for file in &i.raw.dropped_files {
                if let Some(path) = &file.path {
                    if path.extension().and_then(|e| e.to_str()) == Some("akp") {
                        if !self.selected_files.contains(path) {
                            self.selected_files.push(path.clone());
                        }
                    }
                }
            }
        });

        // Detect drag-drop hover state
        let is_hovering = ctx.input(|i| !i.raw.hovered_files.is_empty());

        // Menu bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open AKP Files...").clicked() {
                        self.open_file_dialog();
                        ui.close_menu();
                    }
                    if ui.button("Select Output Directory...").clicked() {
                        self.select_output_directory();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        self.show_about = true;
                        ui.close_menu();
                    }
                });
            });
        });

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Rusty Samplers - Multi-Format Converter");
            ui.separator();

            // File selection section
            ui.group(|ui| {
                ui.label("Input Files");

                ui.horizontal(|ui| {
                    if ui.button("Select AKP Files").clicked() {
                        self.open_file_dialog();
                    }

                    ui.checkbox(&mut self.batch_mode, "Batch Mode");
                });

                // Drag and drop area
                let drop_area = ui.allocate_response(
                    egui::Vec2::new(ui.available_width(), 80.0),
                    egui::Sense::hover()
                );

                ui.allocate_ui_at_rect(drop_area.rect, |ui| {
                    ui.centered_and_justified(|ui| {
                        if is_hovering {
                            ui.colored_label(egui::Color32::from_rgb(100, 150, 255), "Drop files to add them...");
                        } else if self.selected_files.is_empty() {
                            ui.colored_label(egui::Color32::GRAY, "Drop AKP files here or click 'Select AKP Files'");
                        } else {
                            ui.label(format!("{} file(s) selected", self.selected_files.len()));
                        }
                    });
                });

                // Draw border around drop area — bright blue + thicker when hovering
                let (border_color, border_width) = if is_hovering {
                    (egui::Color32::from_rgb(80, 140, 255), 3.0)
                } else {
                    (egui::Color32::from_gray(100), 2.0)
                };
                ui.painter().rect_stroke(
                    drop_area.rect,
                    5.0,
                    egui::Stroke::new(border_width, border_color)
                );

                if !self.selected_files.is_empty() {
                    ui.separator();
                    let mut remove_index = None;
                    egui::ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
                        for (i, file) in self.selected_files.iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label("*");
                                ui.label(file.file_name().unwrap().to_string_lossy());
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.small_button("x").clicked() {
                                        remove_index = Some(i);
                                    }
                                });
                            });
                        }
                    });
                    if let Some(i) = remove_index {
                        self.selected_files.remove(i);
                    }
                }
            });

            ui.add_space(15.0);

            // Format selection
            ui.group(|ui| {
                ui.label("Output Format");
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.output_format, OutputFormat::Sfz, "SFZ Format");
                    ui.radio_value(&mut self.output_format, OutputFormat::DecentSampler, "Decent Sampler XML");
                });

                ui.separator();
                match self.output_format {
                    OutputFormat::Sfz => {
                        ui.colored_label(egui::Color32::from_rgb(70, 130, 180), "Standard SFZ sampler format - compatible with most samplers");
                    }
                    OutputFormat::DecentSampler => {
                        ui.colored_label(egui::Color32::from_rgb(34, 139, 34), "Decent Sampler XML format - includes UI controls and effects");
                    }
                }
            });

            ui.add_space(15.0);

            // Output directory (for batch mode)
            if self.batch_mode {
                ui.group(|ui| {
                    ui.label("Output Directory");
                    ui.horizontal(|ui| {
                        if ui.button("Select Directory").clicked() {
                            self.select_output_directory();
                        }

                        if let Some(dir) = &self.output_directory {
                            ui.label(format!("{}", dir.display()));
                        } else {
                            ui.colored_label(egui::Color32::GRAY, "Same as input files");
                        }
                    });
                });
                ui.add_space(15.0);
            }

            // Conversion button and status
            ui.group(|ui| {
                ui.label("Conversion");

                ui.horizontal(|ui| {
                    let can_convert = !self.selected_files.is_empty() && !self.is_converting;

                    let button_text = if self.is_converting {
                        "Converting..."
                    } else {
                        "Convert Files"
                    };

                    let button = egui::Button::new(button_text).min_size(egui::Vec2::new(120.0, 30.0));

                    if ui.add_enabled(can_convert, button).clicked() {
                        self.start_conversion();
                    }

                    if !self.selected_files.is_empty() {
                        if ui.button("Clear Files").clicked() {
                            self.selected_files.clear();
                            self.conversion_results.clear();
                            self.conversion_status.clear();
                        }
                    }
                });

                if !self.conversion_status.is_empty() {
                    ui.separator();
                    ui.label(&self.conversion_status);
                }

                if self.is_converting {
                    ui.add(egui::ProgressBar::new(self.current_progress).animate(true).text("Converting..."));
                }
            });

            ui.add_space(15.0);

            // Results section
            if !self.conversion_results.is_empty() {
                ui.group(|ui| {
                    ui.label("Conversion Results");

                    let success_count = self.conversion_results.iter().filter(|r| r.success).count();
                    let total_count = self.conversion_results.len();

                    ui.horizontal(|ui| {
                        ui.colored_label(egui::Color32::GREEN, format!("Success: {}", success_count));
                        ui.colored_label(egui::Color32::RED, format!("Failed: {}", total_count - success_count));
                    });

                    ui.separator();
                    egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                        for result in &self.conversion_results {
                            ui.horizontal(|ui| {
                                if result.success {
                                    ui.colored_label(egui::Color32::GREEN, "OK");
                                } else {
                                    ui.colored_label(egui::Color32::RED, "FAIL");
                                }
                                ui.label(result.input_file.file_name().unwrap().to_string_lossy());
                                ui.label("->");
                                if let Some(output) = &result.output_file {
                                    ui.label(output.file_name().unwrap().to_string_lossy());
                                }
                            });
                            if !result.message.is_empty() {
                                ui.indent("result_msg", |ui| {
                                    ui.small(&result.message);
                                });
                            }
                        }
                    });
                });
            }
        });

        // About dialog
        if self.show_about {
            egui::Window::new("About Rusty Samplers")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Rusty Samplers v1.0");
                        ui.label("Multi-Format Sampler Converter");
                    });
                    ui.separator();

                    ui.group(|ui| {
                        ui.label("Converts Akai AKP files to:");
                        ui.label("  - SFZ format");
                        ui.label("  - Decent Sampler XML format");
                    });

                    ui.group(|ui| {
                        ui.label("Features:");
                        ui.label("  - Advanced parameter mapping");
                        ui.label("  - Envelope, filter & LFO conversion");
                        ui.label("  - Modulation routing support");
                        ui.label("  - Batch processing");
                        ui.label("  - Modern GUI interface");
                    });

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Close").clicked() {
                            self.show_about = false;
                        }
                    });
                });
        }
    }
}

impl RustySamplersApp {
    fn open_file_dialog(&mut self) {
        if let Some(files) = FileDialog::new()
            .add_filter("AKP Files", &["akp"])
            .set_title("Select AKP Files")
            .pick_files()
        {
            self.selected_files = files;
            self.conversion_results.clear();
            self.conversion_status.clear();
        }
    }

    fn select_output_directory(&mut self) {
        if let Some(dir) = FileDialog::new()
            .set_title("Select Output Directory")
            .pick_folder()
        {
            self.output_directory = Some(dir);
        }
    }

    fn start_conversion(&mut self) {
        let (tx, rx) = mpsc::channel();
        self.progress_receiver = Some(rx);
        self.conversion_results.clear();

        let files = self.selected_files.clone();
        let format = self.output_format;
        let output_dir = self.output_directory.clone();

        thread::spawn(move || {
            let _ = tx.send(ConversionProgress::Started("Starting conversion...".to_string()));
            let mut success_count = 0usize;

            for (i, file_path) in files.iter().enumerate() {
                let progress_msg = format!("Converting {} ({}/{})",
                    file_path.file_name().unwrap().to_string_lossy(),
                    i + 1,
                    files.len()
                );
                let _ = tx.send(ConversionProgress::Progress(progress_msg, i as f32 / files.len() as f32));

                let conversion_result = rusty_samplers::convert_file(&file_path, format);
                let success = conversion_result.is_ok();

                let result = if success {
                    let output_file = if let Some(dir) = &output_dir {
                        let filename = file_path.file_stem().unwrap();
                        let extension = match format {
                            OutputFormat::Sfz => "sfz",
                            OutputFormat::DecentSampler => "dspreset",
                        };
                        dir.join(format!("{}.{}", filename.to_string_lossy(), extension))
                    } else {
                        match format {
                            OutputFormat::Sfz => file_path.with_extension("sfz"),
                            OutputFormat::DecentSampler => file_path.with_extension("dspreset"),
                        }
                    };

                    let write_result = if let Ok(content) = &conversion_result {
                        std::fs::write(&output_file, content).map_err(|e| e.to_string())
                    } else {
                        Err("Conversion failed".to_string())
                    };

                    let final_success = write_result.is_ok();
                    let message = if final_success {
                        format!("Converted to {} format", match format {
                            OutputFormat::Sfz => "SFZ",
                            OutputFormat::DecentSampler => "Decent Sampler",
                        })
                    } else {
                        write_result.err().unwrap_or_else(|| "Unknown error".to_string())
                    };

                    ConversionResult {
                        input_file: file_path.clone(),
                        output_file: if final_success { Some(output_file) } else { None },
                        success: final_success,
                        message,
                    }
                } else {
                    ConversionResult {
                        input_file: file_path.clone(),
                        output_file: None,
                        success: false,
                        message: conversion_result.err().unwrap_or_else(|| "Unknown conversion error".to_string()),
                    }
                };

                let file_success = result.success;
                let _ = tx.send(ConversionProgress::FileResult(result));

                let status_msg = if file_success {
                    format!("Converted {}", file_path.file_name().unwrap().to_string_lossy())
                } else {
                    format!("Failed to convert {}", file_path.file_name().unwrap().to_string_lossy())
                };
                if file_success { success_count += 1; }
                let _ = tx.send(ConversionProgress::Progress(status_msg, (i + 1) as f32 / files.len() as f32));
            }

            let total_count = files.len();

            let final_message = if success_count == total_count {
                format!("All {} files converted successfully!", total_count)
            } else if success_count > 0 {
                format!("{} of {} files converted successfully", success_count, total_count)
            } else {
                "No files were converted successfully".to_string()
            };

            let _ = tx.send(ConversionProgress::Completed(final_message));
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_min_inner_size([600.0, 500.0])
            .with_title("Rusty Samplers - Multi-Format Converter"),
        ..Default::default()
    };

    eframe::run_native(
        "Rusty Samplers",
        options,
        Box::new(|cc| {
            let mut style = (*cc.egui_ctx.style()).clone();
            style.visuals.button_frame = true;
            style.visuals.collapsing_header_frame = true;
            cc.egui_ctx.set_style(style);

            Ok(Box::new(RustySamplersApp::default()))
        }),
    )
}
