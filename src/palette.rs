use std::collections::HashSet;
use std::io::Write;
use std::fs::{File, OpenOptions, read_dir, read_to_string};
use std::io::Read;
use std::path::Path;

use image::{DynamicImage, GenericImageView, Rgb};

use crate::colour::{selectAverage, SelectionStrategy, selectKMeans, selectMedian, selectRandomly};
use crate::image::loadImage;

pub struct Palette {
    pub name: String,
    pub colours: Vec<Rgb<u8>>,
}

impl Palette {
    pub fn new(filename: &str) -> Palette {
        let colors = loadPalette(filename);
        return Palette {
            name: filename.to_string(),
            colours: colors,
        };
    }

    pub fn generatePalette(imagefilepath: &str, palettename: String, numcolours: usize, selection_strategy: SelectionStrategy) -> Palette {
        let img = loadImage(imagefilepath);
        let raw_pal = generateRawPalette(&img);
        println!("RAW PALETTE SIZE: {}", raw_pal.len());
        let raw_vec: Vec<Rgb<u8>> = raw_pal.into_iter().collect();

        let gen_pal = match selection_strategy {
            SelectionStrategy::Average => selectAverage(&raw_vec, numcolours),
            SelectionStrategy::Random => selectRandomly(&raw_vec, numcolours),
            SelectionStrategy::KMeans => selectKMeans(&raw_vec, numcolours),
            SelectionStrategy::Median => selectMedian(&raw_vec, numcolours)
        };

        return Palette {
            name: palettename.to_string(),
            colours: gen_pal,
        };
    }

    pub fn savePalette(&self) {
        let filepath = Path::new("./palettes").join(&self.name);
        let mut file = File::create(filepath).expect("ERROR: COULD NOT CREATE PALETTE FILE.");

        for colour in &self.colours {
            let hex = rgb_to_hex(*colour);
            writeln!(file, "{}", hex).expect("ERROR: UNABLE TO WRITE TO FILE.")
        }
        println!("INFO: Palette Saved Successfully.");
    }
}

pub fn listPalettes() {
    let paletteDir = Path::new("./palettes");
    let files = read_dir(paletteDir).unwrap();
    for pals in files {
        println!("{:?}", pals.unwrap().file_name())
    }
}

fn loadPalette(filename: &str) -> Vec<Rgb<u8>> {
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

fn rgb_to_hex(rgb: Rgb<u8>) -> String {
    format!("{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2])
}

fn generateRawPalette(img: &DynamicImage) -> HashSet<Rgb<u8>> {
    let rgb_img = img.to_rgb8();
    let mut colours = HashSet::new();

    for pixel in rgb_img.pixels() {
        colours.insert(*pixel);
    }

    return colours;
}