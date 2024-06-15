use std::fs::{File, read_dir, read_to_string};
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};

use image::Rgb;

use crate::colour::{select_average, select_kmeans, select_median, select_randomly, SelectionStrategy};
use crate::image::Image;
use crate::utils::{generate_raw_palette, hex_to_rgb, rgb_to_hex};

#[derive(Clone)]
pub struct Palette {
    pub name: String,
    pub colours: Vec<Rgb<u8>>,
}

impl Palette {
    pub fn new(filename: &str) -> Palette {
        let colors = load_palette(filename);
        return Palette {
            name: filename.to_string(),
            colours: colors,
        };
    }

    pub fn generate_palette(imagefilepath: &str, palettename: String, numcolours: usize, selection_strategy: SelectionStrategy) -> Palette {
        let img = Image::new(imagefilepath);
        let raw_pal = generate_raw_palette(&img.data);
        let raw_vec: Vec<Rgb<u8>> = raw_pal.into_iter().collect();

        let gen_pal = match selection_strategy {
            SelectionStrategy::Average => select_average(&raw_vec, numcolours),
            SelectionStrategy::Random => select_randomly(&raw_vec, numcolours),
            SelectionStrategy::KMeans => select_kmeans(&raw_vec, numcolours),
            SelectionStrategy::Median => select_median(&raw_vec, numcolours)
        };

        return Palette {
            name: palettename,
            colours: gen_pal,
        };
    }

    pub fn save_palette(&self, filepath: Option<&str>) {
        let pathstr = filepath.unwrap_or_else(|| "./palettes");
        let path = Path::new(pathstr).join(&self.name);
        let mut file = File::create(path).expect("ERROR: COULD NOT CREATE PALETTE FILE.");

        for colour in &self.colours {
            let hex = rgb_to_hex(*colour);
            writeln!(file, "{}", hex).expect("ERROR: UNABLE TO WRITE TO FILE.")
        }
        println!("INFO: Palette Saved Successfully.");
    }

    pub fn list_palettes() {
        let palette_dir = Path::new("./palettes");
        let files = read_dir(palette_dir).unwrap();
        for pals in files {
            println!("{:?}", pals.unwrap().file_name())
        }
    }
}

fn load_palette(path: &str) -> Vec<Rgb<u8>> {
    let file_path = if Path::new(path).is_absolute() {
        PathBuf::from(path)
    } else {
        Path::new("./palettes").join(path)
    };
    let mut palette = Vec::new();

    for colour in read_to_string(file_path).unwrap().lines() {
        let rgb = hex_to_rgb(colour).unwrap();
        palette.push(rgb)
    }
    return palette;
}




