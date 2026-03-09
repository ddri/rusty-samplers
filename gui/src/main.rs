use eframe::egui;
use rfd::FileDialog;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use rusty_samplers::OutputFormat;

// Color palette
const ACCENT: egui::Color32 = egui::Color32::from_rgb(90, 140, 255);
const ACCENT_DIM: egui::Color32 = egui::Color32::from_rgb(60, 100, 200);
const SUCCESS: egui::Color32 = egui::Color32::from_rgb(80, 200, 120);
const FAILURE: egui::Color32 = egui::Color32::from_rgb(240, 80, 80);
const MUTED: egui::Color32 = egui::Color32::from_rgb(140, 140, 150);
const SECTION_LABEL: egui::Color32 = egui::Color32::from_rgb(200, 200, 210);
const DROP_BG: egui::Color32 = egui::Color32::from_rgb(30, 32, 40);
const DROP_HOVER_BG: egui::Color32 = egui::Color32::from_rgb(35, 40, 60);

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

fn section_heading(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).color(SECTION_LABEL).size(13.0).strong());
    ui.add_space(4.0);
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
                        self.conversion_status = format!("Error: {msg}");
                        self.is_converting = false;
                    }
                }
            }
        }

        // Handle drag & drop
        ctx.input(|i| {
            for file in &i.raw.dropped_files {
                if let Some(path) = &file.path {
                    if path.extension().and_then(|e| e.to_str()) == Some("akp")
                        && !self.selected_files.contains(path)
                    {
                        self.selected_files.push(path.clone());
                    }
                }
            }
        });

        // Detect drag-drop hover state
        let is_hovering = ctx.input(|i| !i.raw.hovered_files.is_empty());

        // Menu bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(2.0);
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
            ui.add_space(2.0);
        });

        // Bottom bar with convert button
        egui::TopBottomPanel::bottom("bottom_panel")
            .min_height(60.0)
            .show(ctx, |ui| {
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.add_space(12.0);

                    let can_convert = !self.selected_files.is_empty() && !self.is_converting;
                    let button_text = if self.is_converting { "Converting..." } else { "Convert" };

                    let button = egui::Button::new(
                        egui::RichText::new(button_text).size(15.0).strong()
                    )
                    .min_size(egui::Vec2::new(140.0, 36.0))
                    .fill(if can_convert { ACCENT } else { egui::Color32::from_rgb(50, 55, 65) })
                    .rounding(6.0);

                    if ui.add_enabled(can_convert, button).clicked() {
                        self.start_conversion();
                    }

                    if !self.selected_files.is_empty() {
                        ui.add_space(8.0);
                        if ui.button(egui::RichText::new("Clear").color(MUTED)).clicked() {
                            self.selected_files.clear();
                            self.conversion_results.clear();
                            self.conversion_status.clear();
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(12.0);
                        if self.is_converting {
                            ui.add(
                                egui::ProgressBar::new(self.current_progress)
                                    .animate(true)
                                    .desired_width(200.0)
                            );
                        } else if !self.conversion_status.is_empty() {
                            ui.label(egui::RichText::new(&self.conversion_status).color(MUTED).size(12.0));
                        }
                    });
                });
                ui.add_space(8.0);
            });

        // Main content area
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(egui::Margin::symmetric(20.0, 16.0)))
            .show(ctx, |ui| {

            // ── Drop zone ──
            section_heading(ui, "INPUT FILES");

            let drop_height = if self.selected_files.is_empty() { 180.0 } else { 100.0 };
            let drop_area = ui.allocate_response(
                egui::Vec2::new(ui.available_width(), drop_height),
                egui::Sense::click(),
            );

            // Background fill
            let bg = if is_hovering { DROP_HOVER_BG } else { DROP_BG };
            ui.painter().rect_filled(drop_area.rect, 8.0, bg);

            // Border
            let (border_color, border_width) = if is_hovering {
                (ACCENT, 2.0)
            } else {
                (egui::Color32::from_rgb(60, 65, 75), 1.0)
            };
            ui.painter().rect_stroke(
                drop_area.rect,
                8.0,
                egui::Stroke::new(border_width, border_color),
            );

            // Drop zone text
            let center = drop_area.rect.center();
            if is_hovering {
                ui.painter().text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    "Drop to add files",
                    egui::FontId::proportional(16.0),
                    ACCENT,
                );
            } else if self.selected_files.is_empty() {
                ui.painter().text(
                    center - egui::Vec2::new(0.0, 10.0),
                    egui::Align2::CENTER_CENTER,
                    "Drop .akp files here",
                    egui::FontId::proportional(16.0),
                    MUTED,
                );
                ui.painter().text(
                    center + egui::Vec2::new(0.0, 12.0),
                    egui::Align2::CENTER_CENTER,
                    "or click to browse",
                    egui::FontId::proportional(12.0),
                    egui::Color32::from_rgb(100, 100, 110),
                );
            } else {
                ui.painter().text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    format!("{} file(s) selected — drop more or click to add", self.selected_files.len()),
                    egui::FontId::proportional(13.0),
                    MUTED,
                );
            }

            // Click to open file dialog
            if drop_area.clicked() {
                self.open_file_dialog();
            }

            // File list
            if !self.selected_files.is_empty() {
                ui.add_space(6.0);
                let mut remove_index = None;
                egui::ScrollArea::vertical()
                    .max_height(120.0)
                    .show(ui, |ui| {
                        for (i, file) in self.selected_files.iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("AKP").color(ACCENT_DIM).size(10.0).strong());
                                ui.add_space(4.0);
                                ui.label(file.file_name().unwrap_or(file.as_os_str()).to_string_lossy());
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.small_button(egui::RichText::new("x").color(MUTED)).clicked() {
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

            ui.add_space(20.0);

            // ── Format selection ──
            section_heading(ui, "OUTPUT FORMAT");

            ui.horizontal(|ui| {
                ui.add_space(4.0);
                ui.radio_value(&mut self.output_format, OutputFormat::Sfz, "SFZ");
                ui.add_space(12.0);
                ui.radio_value(&mut self.output_format, OutputFormat::DecentSampler, "Decent Sampler");
            });

            ui.add_space(4.0);
            let desc = match self.output_format {
                OutputFormat::Sfz => "Standard sampler format — compatible with most samplers",
                OutputFormat::DecentSampler => "Decent Sampler XML — includes UI controls and effects",
            };
            ui.label(egui::RichText::new(desc).color(MUTED).size(12.0));

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add_space(4.0);
                ui.checkbox(&mut self.batch_mode, egui::RichText::new("Custom output directory").color(MUTED));
            });

            // Output directory (for batch mode)
            if self.batch_mode {
                ui.horizontal(|ui| {
                    ui.add_space(24.0);
                    if ui.small_button("Browse...").clicked() {
                        self.select_output_directory();
                    }
                    ui.add_space(4.0);
                    if let Some(dir) = &self.output_directory {
                        ui.label(egui::RichText::new(format!("{}", dir.display())).size(12.0));
                    } else {
                        ui.label(egui::RichText::new("Default: same as input").color(MUTED).size(12.0));
                    }
                });
            }

            ui.add_space(20.0);

            // ── Results section ──
            if !self.conversion_results.is_empty() {
                section_heading(ui, "RESULTS");

                let success_count = self.conversion_results.iter().filter(|r| r.success).count();
                let fail_count = self.conversion_results.len() - success_count;

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("{success_count} converted")).color(SUCCESS).size(12.0));
                    if fail_count > 0 {
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new(format!("{fail_count} failed")).color(FAILURE).size(12.0));
                    }
                });

                ui.add_space(4.0);
                egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                    for result in &self.conversion_results {
                        ui.horizontal(|ui| {
                            if result.success {
                                ui.label(egui::RichText::new("OK").color(SUCCESS).size(11.0).strong());
                            } else {
                                ui.label(egui::RichText::new("FAIL").color(FAILURE).size(11.0).strong());
                            }
                            ui.add_space(4.0);
                            ui.label(egui::RichText::new(
                                result.input_file.file_name().unwrap_or(result.input_file.as_os_str()).to_string_lossy()
                            ).size(12.0));
                            if let Some(output) = &result.output_file {
                                ui.label(egui::RichText::new("->").color(MUTED).size(12.0));
                                ui.label(egui::RichText::new(
                                    output.file_name().unwrap_or(output.as_os_str()).to_string_lossy()
                                ).size(12.0));
                            }
                        });
                        if !result.message.is_empty() && !result.success {
                            ui.indent("result_msg", |ui| {
                                ui.label(egui::RichText::new(&result.message).color(MUTED).size(11.0));
                            });
                        }
                    }
                });
            }
        });

        // About dialog
        if self.show_about {
            egui::Window::new("About")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .min_width(300.0)
                .show(ctx, |ui| {
                    ui.add_space(8.0);
                    ui.vertical_centered(|ui| {
                        ui.label(egui::RichText::new("Rusty Samplers").size(20.0).strong());
                        ui.add_space(2.0);
                        ui.label(egui::RichText::new("Multi-Format Sampler Converter").color(MUTED).size(13.0));
                    });
                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(8.0);

                    ui.label("Converts Akai AKP sampler programs to:");
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new("  SFZ — universal sampler format").size(12.0));
                    ui.label(egui::RichText::new("  Decent Sampler — XML with UI and effects").size(12.0));

                    ui.add_space(12.0);
                    ui.vertical_centered(|ui| {
                        if ui.button("Close").clicked() {
                            self.show_about = false;
                        }
                    });
                    ui.add_space(4.0);
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
            for file in files {
                if !self.selected_files.contains(&file) {
                    self.selected_files.push(file);
                }
            }
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
                let display_name = file_path.file_name().unwrap_or(file_path.as_os_str()).to_string_lossy();
                let progress_msg = format!("Converting {display_name} ({}/{})",
                    i + 1,
                    files.len()
                );
                let _ = tx.send(ConversionProgress::Progress(progress_msg, i as f32 / files.len() as f32));

                let conversion_result = rusty_samplers::convert_file(file_path, format);
                let success = conversion_result.is_ok();

                let result = if success {
                    let output_file = if let Some(dir) = &output_dir {
                        let filename = file_path.file_stem().unwrap_or(file_path.as_os_str());
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
                    format!("Converted {display_name}")
                } else {
                    format!("Failed to convert {display_name}")
                };
                if file_success { success_count += 1; }
                let _ = tx.send(ConversionProgress::Progress(status_msg, (i + 1) as f32 / files.len() as f32));
            }

            let total_count = files.len();

            let final_message = if success_count == total_count {
                format!("All {total_count} files converted successfully!")
            } else if success_count > 0 {
                format!("{success_count} of {total_count} files converted successfully")
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
            .with_title("Rusty Samplers"),
        ..Default::default()
    };

    eframe::run_native(
        "Rusty Samplers",
        options,
        Box::new(|cc| {
            let mut style = (*cc.egui_ctx.style()).clone();

            // Tighter spacing
            style.spacing.item_spacing = egui::Vec2::new(8.0, 6.0);
            style.spacing.window_margin = egui::Margin::same(16.0);

            // Softer rounding
            style.visuals.window_rounding = egui::Rounding::same(8.0);
            style.visuals.widgets.noninteractive.rounding = egui::Rounding::same(4.0);
            style.visuals.widgets.inactive.rounding = egui::Rounding::same(4.0);
            style.visuals.widgets.hovered.rounding = egui::Rounding::same(4.0);
            style.visuals.widgets.active.rounding = egui::Rounding::same(4.0);

            style.visuals.button_frame = true;
            style.visuals.collapsing_header_frame = true;

            cc.egui_ctx.set_style(style);

            Ok(Box::new(RustySamplersApp::default()))
        }),
    )
}
