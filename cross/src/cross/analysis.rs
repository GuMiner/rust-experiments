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
    cluster_id: i32,
}

fn avg_lengths(a: f32, b: f32, a_len: usize, b_len: usize) -> f32 {
    (a * a_len as f32 + b * b_len as f32) / (a_len + b_len) as f32
}

impl MergedPoints {
    fn new(point: ColorPoint) -> Self {
        let mut merged_points = MergedPoints {
            points: Vec::new(),
            avg_point: [point.c.r(), point.c.g(), point.c.b()], 
            cluster_id: -1
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

    fn expand_color(&self, flat: &mut Vec<ColorPoint>, color: [f32;3]) {
        for point in &self.points {
            let mut point_copy = point.clone();
            point_copy.c = Rgba::from_rgb(color[0], color[1], color[2]);
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

fn clusters_equal(first: i32, second: i32) -> bool {
    first != -1 && second != -1 && first == second
}

fn get_merge_cluster(node_idx: usize, node_cluster_id: i32) -> i32 {
    if node_cluster_id == -1 {
        node_idx as i32
    } else {
        node_cluster_id
    }
}


fn add_child(set: &mut HashMap<i32, Vec<i32>>, parent: i32, child: i32) {
    set.get_mut(&parent).expect("must exist").push(child);
}

fn get_children(set: &HashMap<i32, Vec<i32>>, parent: i32) -> &Vec<i32> {
    set.get(&parent).expect("must exist")
}

fn limit_colors(config: &Config, points: &mut Vec<ColorPoint>, canceller: &mpsc::Receiver<bool>) {
    // Config If: Find-closest and merge
    // No need to reinvent the wheel, can use kdtrees
    // Edit: None of the kdtree packages are sufficient to track points based on ID.
    // Using a naive closest-pairs-merging algorithm is O(n^3), too slow for use.

    // Make list of merged points 
    let mut merged_points = Vec::new();

    for i in 0..points.len() {
        let merged_point = MergedPoints::new(points[i].clone());
        merged_points.push(merged_point);
    }


    // Figure out all cross-point distance pairs
    print!("Loaded {} points\n", merged_points.len());
    // This can be zero if cancellation happens elsewhere. TODO fix.
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

    // Custom comparator required because Rust doesn't know how to sort floats by default
    pairs_with_distances.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    print!("Points sorted into {}\n", pairs_with_distances.len());

    let mut children_of: HashMap<i32, Vec<i32>> = HashMap::new();
    for i in 0..points.len() {
        children_of.insert(i as i32, Vec::new());
    }

    let mut total_clusters: i32 = points.len() as i32;
    // Iterate through point distances, slowly connecting clusters together
    // Future variations of this could merge close points, even if it drops the cluster count < config.num_colors
    for i in 0..pairs_with_distances.len() {
        
        // Early-exit if any data received.
        match canceller.try_recv() {
            Ok(_) => return,
            Err(_) => {}
        }

        let (_, first, second) = pairs_with_distances[i];

        // TODO this merging isn't quite correct. Runs fast enough though. TODO fix.
        let first_cluster = merged_points[first].cluster_id;
        let second_cluster = merged_points[second].cluster_id;
        if clusters_equal(first_cluster, second_cluster) {
            // Clusters equal, do nothing
        } else {
            // Merge first cluster into second
            let merge_cluster = get_merge_cluster(second, second_cluster);
            let source_cluster = get_merge_cluster(first, first_cluster);

            // Move children to the new cluster. TODO move children list
            for child in get_children(&children_of, source_cluster) {
                merged_points[*child as usize].cluster_id = merge_cluster;
            }

            // Move this specific point to the new cluster
            merged_points[source_cluster as usize].cluster_id = merge_cluster;
            add_child(&mut children_of, merge_cluster, source_cluster);
            
            total_clusters = total_clusters - 1;
            if total_clusters == config.num_colors {
                break;
            }
        }
    }

    // TODO -- average out puints
    // Using the parent clusters, return the existing points with their parent colors

    print!("Combined down to {} points.\n", merged_points.len());
    points.clear();
    for merged_point in &merged_points {
        if merged_point.cluster_id == -1 {
           merged_point.expand(points);
        } else {
            merged_point.expand_color(points, merged_points[merged_point.cluster_id as usize].avg_point);
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
