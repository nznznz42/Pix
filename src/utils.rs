use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::thread;

use image::{DynamicImage, Rgb, Rgba, RgbaImage};
use rand::Rng;

use crate::ditherer::BlueNoiseThreshold;

pub fn available_threads() -> usize {
    return thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
}

pub fn hex_to_rgb(hex_code: &str) -> Result<Rgb<u8>, &'static str> {
    let hex = hex_code.trim_start_matches("#");
    if hex.len() != 6 {
        return Err("Hex value must be exactly 6 characters long");
    }

    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid hex value")?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid hex value")?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid hex value")?;

    return Ok(Rgb([r, g, b]));
}

pub fn rgb_to_hex(rgb: Rgb<u8>) -> String {
    format!("{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2])
}

pub fn hex_to_u32(hex: String) -> Result<u32, String> {
    if hex.len() == 7 && &hex[0..1] == "#" {
        match u32::from_str_radix(&hex[1..], 16) {
            Ok(value) => Ok(value),
            Err(_) => Err("Invalid hex number".to_string()),
        }
    } else {
        Err("Invalid hex format".to_string())
    }
}

pub fn sum_fold_and_count(cluster: &Vec<Rgb<u8>>) -> (u32, u32, u32, u32) {
    cluster.iter().fold((0u32, 0u32, 0u32, 0u32), |acc, color| {
        (
            acc.0 + color[0] as u32,
            acc.1 + color[1] as u32,
            acc.2 + color[2] as u32,
            acc.3 + 1,
        )
    })
}

pub fn find_closest_color(color: &Rgb<u8>, palette: &HashSet<Rgb<u8>>) -> Rgb<u8> {
    let (r, g, b) = (color[0], color[1], color[2]);
    let mut min_distance = f32::MAX;
    let pal: Vec<Rgb<u8>> = palette.iter().cloned().collect();
    let mut closest_color = Rgb([0, 0, 0]);

    for palette_color in pal {
        let distance = ((r as f32 - palette_color[0] as f32).powi(2)
            + (g as f32 - palette_color[1] as f32).powi(2)
            + (b as f32 - palette_color[2] as f32).powi(2)).sqrt();
        if distance < min_distance {
            min_distance = distance;
            closest_color = palette_color;
        }
    }

    Rgb([closest_color[0], closest_color[1], closest_color[2]])
}

pub fn calculate_error(old_color: &Rgb<u8>, new_color: &Rgb<u8>) -> Rgb<i16> {
    Rgb([
        old_color[0] as i16 - new_color[0] as i16,
        old_color[1] as i16 - new_color[1] as i16,
        old_color[2] as i16 - new_color[2] as i16,
        //old_color[3] as i16 - new_color[3] as i16,
    ])
}

pub fn distribute_error(image: &mut RgbaImage, x: u32, y: u32, error: &Rgba<i16>, coefficients: &[(i32, i32, f32)]) {
    let mut rgba_image = image.clone();
    let (width, height) = rgba_image.dimensions();

    for &(dx, dy, factor) in coefficients {
        let nx = x as i32 + dx;
        let ny = y as i32 + dy;

        if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
            let nx = nx as u32;
            let ny = ny as u32;
            let mut pixel = rgba_image.get_pixel(nx, ny).clone();

            for i in 0..3 {
                let value = pixel[i] as i16 + (error[i] as f32 * factor) as i16;
                pixel[i] = clamp(value, 0, 255) as u8;
            }

            rgba_image.put_pixel(nx, ny, pixel);
        }
    }

    *image = rgba_image;
}

pub fn clamp(value: i16, min: i16, max: i16) -> i16 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

pub fn generate_raw_palette(img: &DynamicImage) -> HashSet<Rgb<u8>> {
    let rgb_img = img.to_rgb8();
    let mut colours = HashSet::new();

    for pixel in rgb_img.pixels() {
        colours.insert(*pixel);
    }

    return colours;
}

pub fn gen_blue_noise_threshold(threshold: BlueNoiseThreshold) -> u8 {
    let noise_threshold;
    let mut rng = rand::thread_rng();
    match threshold {
        BlueNoiseThreshold::LOW => { noise_threshold = rng.gen_range(0u8..86u8) }
        BlueNoiseThreshold::MEDIUM => { noise_threshold = rng.gen_range(86u8..171u8) }
        BlueNoiseThreshold::HIGH => { noise_threshold = rng.gen_range(171u8..255u8) }
    }
    return noise_threshold;
}
