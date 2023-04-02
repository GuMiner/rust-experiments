
use eframe::egui;
use egui::plot::{Legend, MarkerShape, Plot, Points};
use egui::{Color32, Ui, Response};
use egui_extras::RetainedImage;

// Squashed together samples to test UI tech
// https://github.com/emilk/egui/blob/master/crates/egui_demo_lib/src/demo/plot_demo.rs
pub struct Cross {
    image: RetainedImage,    
    name: String,
    age: u32,
    dropped_files: Vec<egui::DroppedFile>,
    picked_path: Option<String>,
    chart: MarkerDemo
}

impl Default for Cross {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            age: 42,
            image: RetainedImage::from_image_bytes(
                "TestImage.jpg",
                include_bytes!("TestImage.jpg"),
            )
            .unwrap(),
            dropped_files: Vec::new(),
            picked_path: None,
            chart: MarkerDemo::default(),
        }
    }
}

#[derive(PartialEq)]
struct MarkerDemo {
    fill_markers: bool,
    marker_radius: f32,
    automatic_colors: bool,
    marker_color: Color32,
}

impl Default for MarkerDemo {
    fn default() -> Self {
        Self {
            fill_markers: true,
            marker_radius: 5.0,
            automatic_colors: true,
            marker_color: Color32::GREEN,
        }
    }
}

impl MarkerDemo {
    fn markers(&self) -> Vec<Points> {
        MarkerShape::all()
            .enumerate()
            .map(|(i, marker)| {
                let y_offset = i as f64 * 0.5 + 1.0;
                let mut points = Points::new(vec![
                    [1.0, 0.0 + y_offset],
                    [2.0, 0.5 + y_offset],
                    [3.0, 0.0 + y_offset],
                    [4.0, 0.5 + y_offset],
                    [5.0, 0.0 + y_offset],
                    [6.0, 0.5 + y_offset],
                ])
                .name(format!("{:?}", marker))
                .filled(self.fill_markers)
                .radius(self.marker_radius)
                .shape(marker);

                if !self.automatic_colors {
                    points = points.color(self.marker_color);
                }

                points
            })
            .collect()
    }

    fn ui(&mut self, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.fill_markers, "Fill");
            ui.add(
                egui::DragValue::new(&mut self.marker_radius)
                    .speed(0.1)
                    .clamp_range(0.0..=f64::INFINITY)
                    .prefix("Radius: "),
            );
            ui.checkbox(&mut self.automatic_colors, "Automatic colors");
            if !self.automatic_colors {
                ui.color_edit_button_srgba(&mut self.marker_color);
            }
        });

        let markers_plot = Plot::new("markers_demo")
            .data_aspect(1.0)
            .legend(Legend::default());
        markers_plot
            .show(ui, |plot_ui| {
                for marker in self.markers() {
                    plot_ui.points(marker);
                }
            })
            .response
    }
}



impl eframe::App for Cross {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");

            if ui.button("Open file…").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.picked_path = Some(path.display().to_string());
                }
            }
            if let Some(picked_path) = &self.picked_path {
                ui.horizontal(|ui| {
                    ui.label("Picked file:");
                    ui.monospace(picked_path);
                });
            }

            // Show dropped files (if any):
            if !self.dropped_files.is_empty() {
                ui.group(|ui| {
                    ui.label("Dropped files:");

                    for file in &self.dropped_files {
                        let mut info = if let Some(path) = &file.path {
                            path.display().to_string()
                        } else if !file.name.is_empty() {
                            file.name.clone()
                        } else {
                            "???".to_owned()
                        };
                        if let Some(bytes) = &file.bytes {
                            use std::fmt::Write as _;
                            write!(info, " ({} bytes)", bytes.len()).ok();
                        }
                        ui.label(info);
                    }
                });
            }

            self.image.show_scaled(ui, 0.1);

            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name)
                    .labelled_by(name_label.id);
            });
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            if ui.button("Click each year").clicked() {
                self.age += 1;
            }
            ui.label(format!("Hello '{}', age {}", self.name, self.age));
            
            self.chart.ui(ui);
        });

        
        preview_files_being_dropped(ctx);

        // Collect dropped files:
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                self.dropped_files = i.raw.dropped_files.clone();
            }
        });
    }
}

/// Preview hovering files:
fn preview_files_being_dropped(ctx: &egui::Context) {
    use egui::*;
    use std::fmt::Write as _;

    if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
        let text = ctx.input(|i| {
            let mut text = "Dropping files:\n".to_owned();
            for file in &i.raw.hovered_files {
                if let Some(path) = &file.path {
                    write!(text, "\n{}", path.display()).ok();
                } else if !file.mime.is_empty() {
                    write!(text, "\n{}", file.mime).ok();
                } else {
                    text += "\n???";
                }
            }
            text
        });

        let painter =
            ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

        let screen_rect = ctx.screen_rect();
        painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
        painter.text(
            screen_rect.center(),
            Align2::CENTER_CENTER,
            text,
            TextStyle::Heading.resolve(&ctx.style()),
            Color32::WHITE,
        );
    }
}