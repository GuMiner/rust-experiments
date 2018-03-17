extern crate byteorder;
extern crate nalgebra;

use std::io::{BufReader, Read};
use std::fs::File;
use std::str;

// The byteorder crate seems to be implemented rather unusually.
use objects::loader::byteorder::{ByteOrder, LittleEndian};

use nalgebra::{Point3, Point4};

use objects::object::{Voxel, SubObject, Object};
use objects::palette::Palette;

enum ChunkType {
    Main,
    Pack(i32), // Number of models in file
    Size(Point3<i32>), // XYZ size
    Voxels(SubObject), // Voxel listing
    Palette([Point4<u8>; 256]), // An optional palette
    Unknown(String) // An unsupported chunk. 
}

impl ChunkType {
    fn is_not_main(&self) -> bool {
        match *self {
            ChunkType::Main => false,
            _ => true
        }
    }

    fn is_not_size(&self) -> bool {
        match *self {
            ChunkType::Size(xyzi) => false,
            _ => true
        }
    }

    fn chunk_name(&self) -> String {
        match  *self {
            ChunkType::Main => String::from("Main"),
            ChunkType::Pack(models) => String::from("Pack"),
            ChunkType::Size(size) => String::from("Size"),
            ChunkType::Voxels(ref voxels) => String::from("Voxels"),
            ChunkType::Palette(palette) => String::from("Palette"),
            ChunkType::Unknown(ref name) => format!("Unknown of type {}", name)
        }
    }
}

fn parse_chunk(data: &[u8]) -> (ChunkType, usize) {
    // Validate the chunk id exists
    let mut bytes_read: usize = 4;
    check_length(0, 4, &data);

    // Validate the byte sizes exist
    check_length(4, 8, &data);
    bytes_read += 8;

    let chunk_id = str::from_utf8(&data[0..4]).unwrap(); // Should be guaranteed to be safe.
    match chunk_id {
        "MAIN" => {
            (ChunkType::Main, bytes_read)
        },
        "PACK" => {
            // Validate we have a count of chunks in this file.
            bytes_read += 4;
            check_length(12, 4, &data);

            (ChunkType::Pack(LittleEndian::read_i32(&data[12..16])), bytes_read)
        },
        "SIZE" => {
            bytes_read += 12;
            check_length(12, 12, &data);

            let x = LittleEndian::read_i32(&data[12..16]);
            let y = LittleEndian::read_i32(&data[16..20]);
            let z = LittleEndian::read_i32(&data[20..24]);
            (ChunkType::Size(Point3::new(x, y, z)), bytes_read)
        },
        "XYZI" => {
            bytes_read += 4;
            check_length(12, 4, &data);
            let voxel_count = LittleEndian::read_i32(&data[12..16]);
            
            bytes_read += 4 * (voxel_count as usize);
            check_length(16, 4 * (voxel_count as usize), &data);

            let mut voxels: Vec<Voxel> = Vec::new();
            let mut data_idx = 16;
            for voxel in 0..voxel_count {
                let x = *&data[data_idx] as i32;
                let y = *&data[data_idx + 1] as i32;
                let z = *&data[data_idx + 2] as i32;
                let c = *&data[data_idx + 3];
                data_idx += 4;

                voxels.push(Voxel {
                    position: Point3::new(x, y, z),
                    color: c
                });
            }

            let sub_object = SubObject {
                voxels: voxels
            };

            (ChunkType::Voxels(sub_object), bytes_read)
        },
        "RGBA" => {
            let mut colors: [Point4<u8>; 256] = [Point4::new(0xFF, 255, 0, 255); 256];
            
            bytes_read += 4 * 256;
            check_length(12, 4 * 256, &data);
            for i in 0..256 {
                let r = *&data[12 + i * 4];
                let g = *&data[12 + i * 4 + 1];
                let b = *&data[12 + i * 4 + 2];
                let a = *&data[12 + i * 4 + 3];

                colors[(i + 1) % 256] = Point4::new(a, b, g, r);
            }

            (ChunkType::Palette(colors), bytes_read)
        }
        _ => {
            // Skip to the end of this chunk.
            bytes_read = 12 + (LittleEndian::read_i32(&data[4..8]) as usize);
            (ChunkType::Unknown(String::from(chunk_id)), bytes_read)
        }
    }
}

fn check_header(data: &[u8]) {
    let header = "VOX ";
    check_length(0, header.len(), &data);
    for (i, byte) in header.bytes().enumerate() {
        if byte != data[i] {
            panic!("Expected {}, found {} in position {} for '{}'", byte, data[i], i, header);
        }
    }
}

fn check_length(start: usize, length: usize, vec: &[u8]) {
    let vec_length = vec.len();
    if start + length > vec_length {
        panic!("Input voxel object not long enough. Start: {}. Length: {}. Total Length: {}", start, length, vec_length);
    }
}

pub fn load(file_name: &str) -> Object {
    let file = File::open(file_name)
        .expect(format!("Could not open {}", file_name).as_str());

    let mut buf_reader = BufReader::new(file);

    // Slightly inefficient, but these are so small we don't need to stream them in.
    let mut bytes: Vec<u8> = Vec::new();
    buf_reader.read_to_end(&mut bytes)
        .expect(format!("Could not read in {} as a byte array.", file_name).as_str());

    // Validate this is a VOX file
    check_header(&bytes[0..]);

    // Read the file version tag
    check_length(4, 4, &bytes[4..]);
    println!("VOX file {} is version {}", file_name, LittleEndian::read_i32(&bytes[4..8]));

    // Validate we start with MAIN
    let mut byte_idx: usize = 8;
    let (mut chunk_type, mut bytes_read) = parse_chunk(&bytes[byte_idx..]);
    println!("Read in a {} chunk", chunk_type.chunk_name());

    if chunk_type.is_not_main() {
        panic!("Expected the file to start with a main chunk");
    }

    byte_idx += bytes_read;
    let (mut chunk_type, mut bytes_read) = parse_chunk(&bytes[byte_idx..]);

    // This is either a PACK or a SIZE chunk. 
    println!("Read in a {} chunk", chunk_type.chunk_name());
    let model_count = match chunk_type {
        ChunkType::Pack(count) => {
            byte_idx += bytes_read; // We don't call this if it was a SIZE chunk, so parsing is identical from now-on.
            count
        },
        _ => 1
    };

    println!("Models in file: {}", model_count);

    let mut sub_objects: Vec<SubObject> = Vec::new();
    for model in 0..model_count {
        let (mut chunk_type, mut bytes_read) = parse_chunk(&bytes[byte_idx..]);
        println!("Read in a {} chunk", chunk_type.chunk_name());

        if chunk_type.is_not_size() {
            panic!("Expected a size chunk for this model!");
        }
        
        byte_idx += bytes_read;
        let (mut chunk_type, mut bytes_read) = parse_chunk(&bytes[byte_idx..]);
        println!("Read in a {} chunk", chunk_type.chunk_name());

        match chunk_type {
            ChunkType::Voxels(sub_object) => sub_objects.push(sub_object),
            _ => panic!("Expected a voxels (XYZI) chunk for this model!")
        }

        byte_idx += bytes_read;
    }

    // Parse out the optional chunks. Right now, this only includes our palette.
    let mut object_palette = Palette::magica_voxel_default();
    while byte_idx < bytes.len() {
        let (mut chunk_type, mut bytes_read) = parse_chunk(&bytes[byte_idx..]);
        println!("Read in a {} chunk", chunk_type.chunk_name());

        match chunk_type {
            ChunkType::Palette(palette) =>
            {
                object_palette = Palette {
                    colors: palette
                }
            },
            _ => println!("Support for this optional chunk is not implemented yet.")
        }
        
        byte_idx += bytes_read;
    }

    let mut object: Object = Object {
        objects: sub_objects,
        min_bounds: Point3::new(-1, -1, -1), // TODO: compute these bounds
        max_bounds: Point3::new(1, 1, 1),
        palette: object_palette
    };
    
    return object;
}