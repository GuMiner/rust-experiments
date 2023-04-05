use crate::egui::ColorImage;


// https://docs.rs/egui/latest/egui/struct.ColorImage.html#method.from_rgba_unmultiplied
pub fn load_image_from_path(path: &std::path::Path) -> Result<ColorImage, image::ImageError> {
    let image = image::io::Reader::open(path)?.decode()?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();

    Ok(ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    ))
}
