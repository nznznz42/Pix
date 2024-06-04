use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

use image::{DynamicImage, ExtendedColorType, GenericImageView, ImageFormat, Rgb, save_buffer_with_format};
use image::imageops::FilterType;

use crate::colour::euclideanDistance;
use crate::utils::available_threads;

struct Image {
    path: Path,
}

pub fn loadImage(filePath: &str) -> DynamicImage {
    let img = image::open(filePath).expect("ERROR: Unable to open image");
    return img;
}

pub fn saveImage(img: &DynamicImage, extension: &str, filePath: &str) {
    let imgbuf = img.as_bytes();
    let width = img.width();
    let height = img.height();
    let colour = ExtendedColorType::Rgb8;
    let format = ImageFormat::from_extension(extension);
    save_buffer_with_format(filePath, imgbuf, width, height, colour, format.unwrap()).expect("ERROR: Unable to save image");
}

pub fn pixelateImage(img: &DynamicImage, scale: u32, filter: FilterType) -> DynamicImage {
    let (width, height) = img.dimensions();

    let new_width = width / scale;
    let new_height = height / scale;

    let resized_down = img.resize_exact(new_width, new_height, filter);

    let pixelated = resized_down.resize_exact(width, height, filter);

    return pixelated;
}

fn apply_palette_partial(image: &mut DynamicImage, palette: &Vec<Rgb<u8>>, start_row: u32, end_row: u32) {
    let (width, height) = image.dimensions();
    let mut rgb_image = image.to_rgb8();

    for y in start_row..end_row.min(height) {
        for x in 0..width {
            let mut pixel = rgb_image.get_pixel(x, y);
            let mut min_distance = f64::MAX;
            let mut best_match = *pixel;

            for color in palette {
                let distance = euclideanDistance(pixel, color);
                if distance < min_distance as f32 {
                    min_distance = distance as f64;
                    best_match = *color;
                }
            }

            rgb_image.put_pixel(x, y, best_match);
        }
    }

    *image = DynamicImage::ImageRgb8(rgb_image);
}


pub fn apply_palette(image: DynamicImage, palette: Vec<Rgb<u8>>) -> DynamicImage {
    let num_threads = available_threads();
    let (width, height) = image.dimensions();
    let rows_per_thread = (height as usize + num_threads - 1) / num_threads;

    let image = Arc::new(Mutex::new(image));
    let palette = Arc::new(palette);

    let mut handles = vec![];

    for i in 0..num_threads {
        let image = Arc::clone(&image);
        let palette = Arc::clone(&palette);
        let start_row = (i * rows_per_thread) as u32;
        let end_row = ((i + 1) * rows_per_thread).min(height as usize) as u32;

        let handle = thread::spawn(move || {
            let mut image = image.lock().unwrap();
            apply_palette_partial(&mut image, &palette, start_row, end_row);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    return Arc::try_unwrap(image).unwrap().into_inner().unwrap();
}


