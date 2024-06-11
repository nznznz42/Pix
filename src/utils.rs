use std::thread;

use image::Rgb;

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