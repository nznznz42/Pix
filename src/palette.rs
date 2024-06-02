use std::fs::{read_dir, read_to_string};
use std::io::Read;
use std::path::Path;

use image::Rgb;

struct Palette {
    path: Path
}
pub fn listPalettes() {
    let paletteDir = Path::new("./palettes");
    let files = read_dir(paletteDir).unwrap();
    for pals in files {
        println!("{:?}", pals.unwrap().file_name())
    }
}

pub fn loadPalette(filename: &str) -> Vec<Rgb<u8>> {
    let filePath = Path::new("./palettes").join(filename);
    let mut palette = Vec::new();

    for colour in read_to_string(filePath).unwrap().lines() {
        let rgb = hexToRGB(colour).unwrap();
        palette.push(rgb)
    }
    return palette;
}

fn hexToRGB(hexCode: &str) -> Result<Rgb<u8>, &'static str> {
    let hex = hexCode.trim_start_matches("#");
    if hex.len() != 6 {
        return Err("Hex value must be exactly 6 characters long");
    }

    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid hex value")?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid hex value")?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid hex value")?;

    return Ok(Rgb([r, g, b]));
}