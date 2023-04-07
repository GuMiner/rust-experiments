//! Performs separate-threaded analysis of images
use crate::egui::ColorImage;
use crate::egui::Rgba;

use std::sync::mpsc;

use super::config::Config;

// Doc comments: https://doc.rust-lang.org/reference/comments.html#:~:text=Comments%20in%20Rust%20code%20follow%20the%20general%20C%2B%2B,comments%20are%20interpreted%20as%20a%20form%20of%20whitespace.

// Might need to derive a few traits here
// https://doc.rust-lang.org/rust-by-example/trait/derive.html
pub struct ColorPoint {
    pub x: f64,
    pub y: f64,
    pub c: Rgba,
}

pub fn update_pattern(image: ColorImage, config: Config, canceller: mpsc::Receiver<bool>) -> Vec<ColorPoint> {
    let mut points = Vec::new();

    pass_through(image, config, &mut points, canceller);

    points
}

fn pass_through(image: ColorImage, config: Config, points: &mut Vec<ColorPoint>, canceller: mpsc::Receiver<bool>) {
    let y_step = image.size[1] as i32 / config.num_height;
    let x_step = image.size[0] as i32 / config.num_width;
    
    for y in 0..image.size[1] {
        // Early-exit if any data received.
        match canceller.try_recv() {
            Ok(_) => return,
            Err(_) => {}
        }

        for x in 0..image.size[0] {
            let color = image.pixels[x + y*image.size[0]];
            if x as i32 % x_step == 0 && y as i32 % y_step == 0 {
                points.push(ColorPoint { 
                    x: x as f64,
                    y: (y as f64)*-1.0, 
                    c: Rgba::from(color)});
            }
        }
    }
}
