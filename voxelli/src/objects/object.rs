extern crate nalgebra;

use nalgebra::Point3;
use objects::palette::Palette;

#[derive(Debug)]
pub struct Voxel {
    pub position: Point3<i32>,
    pub color: u8
}

// The sub-objects making up a voxel-object
#[derive(Debug)]
pub struct SubObject {
    pub voxels: Vec<Voxel>
}

// A total voxel-based object.
pub struct Object {
    pub objects: Vec<SubObject>,
    pub min_bounds: Point3<i32>,
    pub max_bounds: Point3<i32>,
    pub palette: Palette
}