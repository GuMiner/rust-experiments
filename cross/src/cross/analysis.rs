//! Performs separate-threaded analysis of images
use crate::egui::ColorImage;
use crate::egui::Rgba;

use std::sync::mpsc;
use std::collections::HashMap;

use super::config::Config;

// Doc comments: https://doc.rust-lang.org/reference/comments.html#:~:text=Comments%20in%20Rust%20code%20follow%20the%20general%20C%2B%2B,comments%20are%20interpreted%20as%20a%20form%20of%20whitespace.

// Might need to derive a few traits here
// https://doc.rust-lang.org/rust-by-example/trait/derive.html
#[derive(Clone, PartialEq)]
pub struct ColorPoint {
    pub x: f64,
    pub y: f64,
    pub c: Rgba,
}

pub struct MergedPoints {
    points: Vec<ColorPoint>,
    avg_point: [f32; 3]
}

fn avg_lengths(a: f32, b: f32, a_len: usize, b_len: usize) -> f32 {
    (a * a_len as f32 + b * b_len as f32) / (a_len + b_len) as f32
}

impl MergedPoints {
    fn new(point: ColorPoint) -> Self {
        let mut merged_points = MergedPoints {
            points: Vec::new(),
            avg_point: [point.c.r(), point.c.g(), point.c.b()]
        };

        merged_points.points.push(point);
        merged_points
    }

    fn expand(&self, flat: &mut Vec<ColorPoint>) {
        for point in &self.points {
            let mut point_copy = point.clone();
            point_copy.c = Rgba::from_rgb(self.avg_point[0], self.avg_point[1], self.avg_point[2]);
            flat.push(point_copy);
        }
    }

    fn dist_sqd(&self, other: &Self) -> f32 {
        (other.avg_point[0] - self.avg_point[0]).powi(2) + 
        (other.avg_point[1] - self.avg_point[1]).powi(2) + 
        (other.avg_point[2] - self.avg_point[2]).powi(2)
    }

    fn merge(&mut self, other: &Self) {
        let old_point_length = self.points.len();
        let new_point_length = other.points.len();
        for point in &other.points {
            self.points.push(point.clone());
        }

        // Recalculate the new average for this position
        let new_average_r = avg_lengths(self.avg_point[0], other.avg_point[0], old_point_length, new_point_length);
        let new_average_g = avg_lengths(self.avg_point[1], other.avg_point[1], old_point_length, new_point_length);
        let new_average_b = avg_lengths(self.avg_point[2], other.avg_point[2], old_point_length, new_point_length);
        self.avg_point[0] = new_average_r;
        self.avg_point[1] = new_average_g;
        self.avg_point[2] = new_average_b;
    }
}

pub fn update_pattern(image: ColorImage, config: Config, canceller: mpsc::Receiver<bool>) -> Vec<ColorPoint> {
    let mut points = Vec::new();

    // Config If: Take points then constrict to color limit.
    pass_through(image, &config, &mut points, &canceller);
    limit_colors(&config, &mut points, &canceller);

    points
}

fn limit_colors(config: &Config, points: &mut Vec<ColorPoint>, canceller: &mpsc::Receiver<bool>) {
    // Config If: Find-closest and merge
    // No need to reinvent the wheel, can use kdtrees
    // Edit: None of the kdtree packages are sufficient to track points based on ID. 
    let mut merged_points = HashMap::new();

    let mut idx = 0;
    for point in points.iter() {
        let merged_point = MergedPoints::new(point.clone());
        merged_points.insert(idx, merged_point);
        idx += 1;
    }

    print!("Loaded {} points\n", merged_points.len());
    while merged_points.len() > config.num_colors as usize {
        let mut nearest_distance = f32::MAX;
        let mut nearest_points = [0, 0];

        // O(n^2) inefficient search, but fast enough for this small scale app.   
        // ... this is not fast enough and will need edits.
        for (index, first) in &merged_points {
            // Early-exit if any data received.
            match canceller.try_recv() {
                Ok(_) => return,
                Err(_) => {}
            }

            for (second_index, second) in &merged_points {
                if index != second_index {
                    let distance = first.dist_sqd(second);
                    if distance < nearest_distance {
                        nearest_points[0] = *index;
                        nearest_points[1] = *second_index;
                        nearest_distance = distance;
                    }
                }
            }
        }
        
        let removed_points = merged_points.remove(&nearest_points[1]).unwrap();
        let nearest_point = merged_points.get_mut(&nearest_points[0]).unwrap();
        nearest_point.merge(&removed_points);
    }

    print!("Combined down to {} points.\n", merged_points.len());
    points.clear();
    for (index, merged_point) in &merged_points {
        merged_point.expand(points);
    }
}

fn pass_through(image: ColorImage, config: &Config, points: &mut Vec<ColorPoint>, canceller: &mpsc::Receiver<bool>) {
    // Iterate in floating point to avoid rounding errors that generate slightly more expected points.
    let y_step = image.size[1] as f64 / (config.num_height as f64);
    let x_step = image.size[0] as f64 / (config.num_width as f64);
    
    for y in 0..config.num_height {
        // Early-exit if any data received.
        match canceller.try_recv() {
            Ok(_) => return,
            Err(_) => {}
        }

        for x in 0..config.num_width {
            // Round steps to avoid scrolling issues at image ends.
            let x_eff = (x_step * (x as f64)) as usize;
            let y_eff = (y_step * (y as f64)) as usize;
            let color = image.pixels[x_eff + y_eff * image.size[0]];
            points.push(ColorPoint { 
                x: x_eff as f64,
                y: (y_eff as f64)*-1.0, 
                c: Rgba::from(color)});
        }
    }
}
