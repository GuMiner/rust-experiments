extern crate nalgebra;

use std::io::BufReader;
use std::io::Error;
use std::io::Read;
use std::fs::File;

use nalgebra::Point3;

use objects::object::Voxel;
use objects::object::SubObject;
use objects::object::Object;
use objects::palette::Palette;

pub fn load(file_name: &str) -> Result<Object, Error> { // Too bad we cannot use a generic here...
    let file = File::open(file_name)?;

    let mut buf_reader = BufReader::new(file);

    // Slightly inefficient, but these are so small we don't need to stream them in.
    let mut bytes: Vec<u8> = Vec::new();
    buf_reader.read_to_end(&mut bytes)?;

    
    let mut voxels: Vec<Voxel> = Vec::new();
    voxels.push(Voxel {
        position: Point3::new(1,2,3),
        color: 1
    });

    let mut sub_objects: Vec<SubObject> = Vec::new();
    sub_objects.push(SubObject {
        voxels: voxels
    });

    let mut object: Object = Object {
        objects: sub_objects,
        min_bounds: Point3::new(-1, -1, -1),
        max_bounds: Point3::new(1, 1, 1),
        palette: Palette::magica_voxel_default()
    };
    
    return Ok(object);
}