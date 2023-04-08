
use eframe::egui;
use egui::Ui;

use std::sync::mpsc;
use std::thread;

mod analysis;
use analysis::ColorPoint;

mod config;
use config::Config;

mod input;

mod renderer;
use renderer::ChartData;

// Squashed together samples to test UI tech
// https://github.com/emilk/egui/blob/master/crates/egui_demo_lib/src/demo/plot_demo.rs
pub struct Cross {
    // Visual display
    image: egui::ColorImage,
    texture: Option<egui::TextureHandle>,

    // Analysis subthread
    process_handle: Option<std::thread::JoinHandle<Vec<ColorPoint>>>,
    cancel_sender: Option<mpsc::Sender<bool>>,
    has_finished: bool,
    
    // Analysis settings
    config: Config,

    // Result
    chart_data: Option<ChartData>,
}

impl Default for Cross {
    fn default() -> Self {
        Self {
            image: egui::ColorImage::default(),
            texture: None,
            process_handle: None,
            cancel_sender: None,
            has_finished: false,
            chart_data: None,
            config: Config::default(),
        }
    }
}


impl Cross {
    fn run_analysis(&mut self) {
        let copied_image = self.image.clone();

        // 
        if let Some(sender) = &self.cancel_sender {
            _ = sender.send(true);
        }
        if self.process_handle.is_some() {
            _ = self.process_handle.take().expect("should exist, get rid of this silliness").join();
        }

        self.has_finished = false;

        let (tx, rx): (mpsc::Sender<bool>, mpsc::Receiver<bool>) = mpsc::channel();
        self.cancel_sender = Some(tx);

        let copied_config = self.config.clone();
        self.process_handle = Some(thread::spawn(|| { analysis::update_pattern(copied_image, copied_config, rx) }));
    }

    fn load_image(&mut self, path: std::path::PathBuf, ui: &mut Ui) {
        let loaded_image = input::load_image_from_path(&path);
        match loaded_image {
            Ok(image) =>
            {
                // Copy the image for future use and the texture
                self.image = image.clone();
                self.texture = Some(ui.ctx().load_texture(
                    "loaded-image",
                    image,
                    Default::default()));
                self.run_analysis();
            },
            Err(err) => {
                print!("Unable to load image: {}", err)
            }
        };
    }
}

impl eframe::App for Cross {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal_centered(|ui| {    
                ui.vertical(|ui| { 
                    // Image loading and rendering    
                    if ui.button("Select Image file...").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("images", &["jpg", "jpeg", "png"])
                            .pick_file() {
                                self.load_image(path, ui);
                        }
                    }

                    if self.image.size[0] != 0 {
                        ui.label(format!("Image size: [{},{}]", self.image.size[0], self.image.size[1]));
                    }

                    if let Some(texture) = &self.texture {
                        // Scale image down to 200x(aspect-ratio)
                        let width = 200.0;
                        let height = width *
                            (self.image.size[1] as f32 / self.image.size[0] as f32);
                        ui.image(texture, egui::Vec2::new(width, height));
                    }

                    // Generation controls
                    ui.add(egui::Slider::new(&mut self.config.num_width, 10..=200).text("Width"));
                    ui.add(egui::Slider::new(&mut self.config.num_height, 10..=200).text("Height"));
                    ui.add(egui::Slider::new(&mut self.config.num_colors, 2..=50).text("Colors"));
                    ui.add(egui::Slider::new(&mut self.config.num_days, 1..=365).text("Days"));
                    if self.config.recalculate_columns() {
                        self.run_analysis();
                    }

                    // TODO add aspect ratio lock
                    // TODO add config for the output generation
                });

                // Cross-stitch chart
                ui.vertical(|ui| {
                    if let Some(chart_data) = &self.chart_data {
                        renderer::render_chart(ui, chart_data);
                    }

                    if let Some(handle) = &self.process_handle {
                        if handle.is_finished() {
                            self.cancel_sender = None;
                            match self.process_handle.take().expect("make less confusing.").join() {
                                Ok(points) =>
                                {
                                    self.chart_data = Some(ChartData { points: points });
                                    self.has_finished = true; 
                                },
                                Err(_) => {}
                            };

                            self.process_handle = None;
                        }
                    }
                });
            });
        });
    }
}
