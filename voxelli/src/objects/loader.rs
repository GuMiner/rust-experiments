extern crate nalgebra;

use std::io::BufReader;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::fs::File;

use nalgebra::Point3;

use objects::object::Voxel;
use objects::object::SubObject;
use objects::object::Object;
use objects::palette::Palette;

fn check_header(data: &[u8]) -> Result<(), Error> {
    // TODO: Implement

    return Ok(());
}

fn check_length<T>(start: usize, length: usize, vec: &Vec<T>) -> Result<(), Error> {
    let vec_length = vec.len();
    if start + length >= vec_length {
        return Err(Error::new(ErrorKind::InvalidData, format!("Input voxel object not long enough. Start: {}. Length: {}. Total Length: {}", start, length, vec_length).as_str()));
    }

    return Ok(());
}

pub fn load(file_name: &str) -> Result<Object, Error> { // Too bad we cannot use a generic here...
    let file = File::open(file_name)?;

    let mut buf_reader = BufReader::new(file);

    // Slightly inefficient, but these are so small we don't need to stream them in.
    let mut bytes: Vec<u8> = Vec::new();
    buf_reader.read_to_end(&mut bytes)?;

    check_length(0, 4, &bytes)?;
    check_header(&bytes[0..3])?;

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