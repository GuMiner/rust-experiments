// Turn off warnings that keep adding lots of stuff to the terminal and hide errors
#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_variables)]

extern crate kiss3d;
extern crate nalgebra;

mod objects;
use objects::loader;

use std::time::Duration;
use std::thread;

use nalgebra::{Vector3, Translation3, UnitQuaternion};
use kiss3d::window::Window;
use kiss3d::light::Light;

fn main() {
    let object = loader::load(r"C:\Users\Gustave\Desktop\Programs\MagixaVoxel-0.99\vox\chr_knight.vox");

    let mut window = Window::new("Kiss3d: cube");

    for sub_object in object.objects {
        for voxel in sub_object.voxels {
            let mut vx = window.add_cube(1.0, 1.0, 1.0);
            vx.append_translation(&Translation3::new(voxel.position.x as f32, voxel.position.y as f32, voxel.position.z as f32));
            
            let color = object.palette.colors[voxel.color as usize];
            vx.set_color((color[3] as f32) / 255.0, (color[2] as f32) / 255.0, (color[1] as f32) / 255.0);
        }
    }

    let mut c = window.add_cube(1.0, 1.0, 1.0);
    c.append_translation(&Translation3::new(1.0, 1.0, 1.0));
    c.set_color(0.0, 1.0, 0.0);

    window.set_light(Light::StickToCamera);

    let rot = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.0014);


    while window.render() {
        c.prepend_to_local_rotation(&rot);
    }


    thread::sleep(Duration::from_millis(1000));
}
