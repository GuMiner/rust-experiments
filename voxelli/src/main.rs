// Turn off warnings that keep adding lots of stuff to the terminal and hide errors
#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_variables)]

extern crate kiss3d;
extern crate nalgebra;
extern crate ncollide;

mod objects;
use objects::loader;
use objects::object::Object;

mod fps;
use fps::Fps;

use std::path::Path;
use std::time::Duration;
use std::thread;

use nalgebra::{Point2, Point3, Vector3, Translation3, UnitQuaternion};

use kiss3d::camera::FirstPerson;
use kiss3d::light::Light;
use kiss3d::scene::SceneNode;
use kiss3d::text::Font;
use kiss3d::window::Window;

fn add_object(node: &mut SceneNode, object: &Object, offset: Point3<f32>) {
    for sub_object in object.objects.iter() {
            for voxel in sub_object.voxels.iter() {
                let mut vx = node.add_cube(1.0, 1.0, 1.0);

                let voxel_pos = &Translation3::new(voxel.position.x as f32 + offset.x, voxel.position.y as f32 + offset.y, voxel.position.z as f32 + offset.z);
                vx.append_translation(voxel_pos);

                let color = object.palette.colors[voxel.color as usize];
                vx.set_color((color[3] as f32) / 255.0, (color[2] as f32) / 255.0, (color[1] as f32) / 255.0);
            }
        }
}

fn main() {
    let mut window = Window::new("Voxelli");

    // Ensure object destructors are called before the window is disposed of.
    {
        let mut camera = FirstPerson::new(Point3::new(-10.0, -10.0, 20.0), Point3::new(0.0, 0.0, 0.0));

        let font = Font::new(Path::new(r"./fonts/DejaVuSans.ttf"), 32);

        let road_model = loader::load(r"./models/Road_Straight.vox");
        let car_model = loader::load(r"./models/long_car.vox");

        // Load the object into the window
        let mut road_straight = window.add_group();
        let road_bodies = add_object(&mut road_straight, &road_model, Point3::new(0.0, 0.0, 2.0));

        let mut car = window.add_group();
        let car_bodies = add_object(&mut car, &car_model, Point3::new(0.0, 0.0, 5.0));

        window.set_light(Light::StickToCamera);

        let rot = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.0044);

        let mut fps = Fps::new();
        while window.render_with_camera(&mut camera) {
            window.draw_text(format!("FPS: {:.*}", 2, fps.fps).as_str(), &Point2::new(30.0, 30.0), &font, &Point3::new(1.0, 0.8, 0.7));

            let frame_delta = fps.update();
            // TODO: Use a different physics library as there are version mismatches between ncollide, nalgebra, kiss3d, and nphysics3d
        }
    }

    thread::sleep(Duration::from_millis(1000));
}
