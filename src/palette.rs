use std::collections::HashSet;
use std::fs::{File, read_dir, read_to_string};
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};

use image::{DynamicImage, GenericImageView, Rgb};

use crate::colour::{selectAverage, SelectionStrategy, selectKMeans, selectMedian, selectRandomly};
use crate::image::Image;
use crate::utils::{hexToRGB, rgb_to_hex};

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
        let img = Image::new(imagefilepath);
        let raw_pal = generateRawPalette(&img.data);
        let raw_vec: Vec<Rgb<u8>> = raw_pal.into_iter().collect();

        let gen_pal = match selection_strategy {
            SelectionStrategy::Average => selectAverage(&raw_vec, numcolours),
            SelectionStrategy::Random => selectRandomly(&raw_vec, numcolours),
            SelectionStrategy::KMeans => selectKMeans(&raw_vec, numcolours),
            SelectionStrategy::Median => selectMedian(&raw_vec, numcolours)
        };

        return Palette {
            name: palettename,
            colours: gen_pal,
        };
    }

    pub fn savePalette(&self, filepath: Option<&str>) {
        let pathstr = filepath.unwrap_or_else(|| "./palettes");
        let path = Path::new(pathstr).join(&self.name);
        let mut file = File::create(path).expect("ERROR: COULD NOT CREATE PALETTE FILE.");

        for colour in &self.colours {
            let hex = rgb_to_hex(*colour);
            writeln!(file, "{}", hex).expect("ERROR: UNABLE TO WRITE TO FILE.")
        }
        println!("INFO: Palette Saved Successfully.");
    }

    pub fn listPalettes() {
        let paletteDir = Path::new("./palettes");
        let files = read_dir(paletteDir).unwrap();
        for pals in files {
            println!("{:?}", pals.unwrap().file_name())
        }
    }
}

fn loadPalette(path: &str) -> Vec<Rgb<u8>> {
    let filePath = if Path::new(path).is_absolute() || Path::new(path).parent().is_some() {
        PathBuf::from(path)
    } else {
        Path::new("./palettes").join(path)
    };
    let mut palette = Vec::new();

    for colour in read_to_string(filePath).unwrap().lines() {
        let rgb = hexToRGB(colour).unwrap();
        palette.push(rgb)
    }
    return palette;
}


fn generateRawPalette(img: &DynamicImage) -> HashSet<Rgb<u8>> {
    let rgb_img = img.to_rgb8();
    let mut colours = HashSet::new();

    for pixel in rgb_img.pixels() {
        colours.insert(*pixel);
    }

    return colours;
}