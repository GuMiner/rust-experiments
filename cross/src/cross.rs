
use eframe::egui;
use egui::Ui;
use std::thread;

mod analysis;
use analysis::ColorPoint;

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
    has_finished: bool,

    // Result
    chart_data: ChartData,

    // ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
}

impl Default for Cross {
    fn default() -> Self {
        Self {
            image: egui::ColorImage::default(),
            has_finished: false,
            texture: None,
            process_handle: None,
            chart_data: ChartData::default(),
        }
    }
}


impl Cross {
    fn load_image(&mut self, path: std::path::PathBuf, ui: &mut Ui) {
        let loaded_image = input::load_image_from_path(&path);
        match loaded_image {
            Ok(image) =>
            {
                // Copy the image thrice (future use, texture, and backtround threading)
                self.image = image.clone();

                let copied_image = image.clone();
                self.process_handle = Some(thread::spawn(|| { analysis::update_pattern(copied_image) }));

                self.texture = Some(ui.ctx().load_texture(
                    "loaded-image",
                    image,
                    Default::default()));
        
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
                // Image loading and rendering        
                ui.vertical(|ui| { 
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
                });

                // Cross-stitch chart
                ui.vertical(|ui| {
                    if self.has_finished {
                        renderer::render_chart(ui, &self.chart_data);
                    }
                    else
                    {
                        if let Some(handle) = &self.process_handle {
                            if handle.is_finished() {
                                match self.process_handle.take().expect("make less confusing.").join() {
                                    Ok(points) =>
                                    {
                                        self.chart_data = ChartData { points: points };
                                        self.has_finished = true; 
                                    },
                                    Err(_) => {}
                                };
                            }
                        }
                    }
                });
            });
        });
    }
}
