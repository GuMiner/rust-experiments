//! Performs separate-threaded analysis of images
use crate::egui::ColorImage;
use crate::egui::Rgba;

use std::sync::mpsc;
use clustering;

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

impl clustering::Elem for ColorPoint {
    fn dimensions(&self) -> usize {
        3
    }

    fn at(&self, i: usize)  -> f64 {
        self.c[i] as f64
    }
}

pub struct AvgColor {
    pub avg: [f32;3],
    pub num: i32,
}

impl AvgColor {
    fn new() -> Self {
        AvgColor {
            avg: [0.0, 0.0, 0.0],
            num: 0,
        }
    }

    fn add_color(&mut self, c: Rgba) {
        self.avg[0] += c[0];
        self.avg[1] += c[1];
        self.avg[2] += c[2];
        self.num += 1;
    }

    fn compute_average(&mut self) {
        if self.num != 0 {
            self.avg[0] /= self.num as f32;
            self.avg[1] /= self.num as f32;
            self.avg[2] /= self.num as f32;
        }
    }
}

pub fn update_pattern(image: ColorImage, config: Config, canceller: mpsc::Receiver<bool>) -> Vec<ColorPoint> {
    let mut points = Vec::new();

    // Config If: Take points then constrict to color limit.
    pass_through(image, &config, &mut points, &canceller);
    if points.len() > 0 {
        let limited_points = limit_colors(&config, &mut points, &canceller);
        limited_points
    } else {
        points
    }
}

fn limit_colors(config: &Config, points: &mut Vec<ColorPoint>, canceller: &mpsc::Receiver<bool>) -> Vec<ColorPoint> {
    // Config If: Find-closest and merge
    // No need to reinvent the wheel, can use kmeans clustering

    // Cluster
    let mut limited_points = Vec::new();
    let clusters = clustering::kmeans(config.num_colors as usize, &points, 100); // max-iters
    match canceller.try_recv() {
        Ok(_) => return limited_points,
        Err(_) => {}
    }
    // print!("Computed a total of {} clusters\n", clusters.centroids.len());

    // Find average colors for each cluster
    let mut avg_cluster_colors = vec![];
    for _ in 0..clusters.centroids.len() {
        avg_cluster_colors.push(AvgColor::new());
    }

    for i in 0..clusters.membership.len() {
        let cluster_id = clusters.membership[i];
        avg_cluster_colors[cluster_id].add_color(clusters.elements[i].c);
    }
    
    for i in 0..clusters.centroids.len() {
        avg_cluster_colors[i].compute_average();
    }

    // Expand out the clusters into a new set of colors
    for i in 0..clusters.elements.len() {
        let cluster_id = clusters.membership[i];
        let cluster_color = avg_cluster_colors[cluster_id].avg;
        limited_points.push(ColorPoint { 
            x: clusters.elements[i].x,
            y: clusters.elements[i].y,
            c: Rgba::from_rgb(cluster_color[0], cluster_color[1], cluster_color[2])});
    }

    limited_points
}

fn pass_through(image: ColorImage, config: &Config, points: &mut Vec<ColorPoint>, canceller: &mpsc::Receiver<bool>) {
    if image.size[0] == 0 {
        return
    }

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
