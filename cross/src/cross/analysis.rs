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

impl ColorPoint {
    fn dist_sqd(&self, other: &Self) -> f32 {
        (other.c.r() - self.c.r()).powi(2) + 
        (other.c.g() - self.c.g()).powi(2) + 
        (other.c.b() - self.c.b()).powi(2)
    }
}

pub struct MergedPoints {
    points: Vec<ColorPoint>,
    avg_point: [f32; 3],
    parent: i32,
}

fn avg_lengths(a: f32, b: f32, a_len: usize, b_len: usize) -> f32 {
    (a * a_len as f32 + b * b_len as f32) / (a_len + b_len) as f32
}

impl MergedPoints {
    fn new(point: ColorPoint) -> Self {
        let mut merged_points = MergedPoints {
            points: Vec::new(),
            avg_point: [point.c.r(), point.c.g(), point.c.b()], 
            parent: -1
        };

        merged_points.points.push(point);
        merged_points
    }

    fn is_parent(&self) -> bool {
        self.parent == -1
    }

    fn expand(&self, flat: &mut Vec<ColorPoint>) {
        for point in &self.points {
            let mut point_copy = point.clone();
            point_copy.c = Rgba::from_rgb(self.avg_point[0], self.avg_point[1], self.avg_point[2]);
            flat.push(point_copy);
        }
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
    // Using a naive closest-pairs-merging algorithm is O(n^3), too slow for use.

    // Make list of merged points 
    let mut merged_points = HashMap::new();

    for i in 0..points.len() {
        let merged_point = MergedPoints::new(points[i].clone());
        merged_points.insert(i, merged_point);
    }

    // Figure out all cross-point distance pairs
    print!("Loaded {} points\n", merged_points.len());
    let mut pairs_with_distances = Vec::new();
    for i in 0..(points.len() - 1) {
        // Early-exit if any data received.
        match canceller.try_recv() {
            Ok(_) => return,
            Err(_) => {}
        }

        for j in (i + 1)..points.len() {
            let distance = points[i].dist_sqd(&points[j]);
            pairs_with_distances.push((distance, i, j));
        }
    }

    pairs_with_distances.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());

    // This doesn't work because it doesn't follow chains
    // Merge closest pairs until the total number of colors is reduced
    //for i in 0..(pairs_with_distances.len() - config.num_colors as usize) {
    //    let (_, first, second) = pairs_with_distances[i];
    //    let first_point = merged_points.remove(&first).unwrap();
    //    let second_point = merged_points.remove(&second).unwrap();
//
    //    if first_point.is_parent() { // Parent == -1 index
    //        if second_point.is_parent() {
    //            // Merge 1 into 2 and redirect 1->2 (No parent)
    //            second_point.merge(&first_point);
    //            first_point.parent = second as i32;
    //        } else {
    //            // Merge 1 into 2's parent. Redirect 1 to 2's parent.
    //            let second_parent_idx = second_point.parent as usize;
    //            let second_parent = merged_points.remove(&second_parent_idx).unwrap();
    //            second_parent.merge(&first_point);
    //            first_point.parent = second_point.parent;
//
    //            merged_points.insert(second_parent_idx, second_parent);
    //        }
    //    } else {
    //        // Either 2 is or is not a top-level node.
    //        // In either case, Merge 2 into 1's parent. Redirect 2 to 1's parent.
    //        let first_parent_idx = second_point.parent as usize;
    //        let first_parent = merged_points.remove(&(second_point.parent as usize)).unwrap();
    //    }
//
    //    merged_points.insert(first, first_point);
    //    merged_points.insert(second, first_point);
    //}
    //    
    //    let removed_points = merged_points.remove(&nearest_points[1]).unwrap();
    //    let nearest_point = merged_points.get_mut(&nearest_points[0]).unwrap();
    //    nearest_point.merge(&removed_points);
    //}

    print!("Combined down to {} points.\n", merged_points.len());
   // points.clear();
    for (index, merged_point) in &merged_points {
        if merged_point.is_parent() {
           // merged_point.expand(points);
        }
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
