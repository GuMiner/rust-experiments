extern crate vulkano;
extern crate vulkano_win;
extern crate winit;

use vulkano::instance::Instance;
use vulkano::instance::PhysicalDevice;

fn main() {
    let extensions = vulkano_win::required_extensions();
    let instance = Instance::new(None, &extensions, None)
        .expect("Failed to create the Vulkan instance for display.");

    let mut device_count = 0;
    for device in PhysicalDevice::enumerate(&instance) {
        println!("Found device '{}' of type '{:?}'.", device.name(), device.ty());
        device_count += 1;
    }

    println!("Enumerated {} device{}.", device_count, if device_count == 1 { "" } else { "s" })
}
