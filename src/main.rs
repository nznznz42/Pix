use ::image::imageops::FilterType;

use crate::image::{apply_palette, loadImage, pixelateImage, saveImage};
use crate::palette::{generateRawPalette, loadPalette};

mod palette;
mod colour;
mod image;
mod utils;

fn main() {
    let inpath = "./input/creek.jpg";
    let outpath = "./output/tiger.png";
    let palname = "hollow.hex";
    let pxFactor = 2;
    //px_std(inpath, outpath, palname, pxFactor, FilterType::Nearest)
    px_gen_raw_pal(outpath)
}

fn px_std(inputfilepath: &str, outputfilepath: &str, palette: &str, pxfactor: u32, interpolfilter: FilterType) {
    let img = loadImage(inputfilepath);
    let pal = loadPalette(palette);
    let pix = pixelateImage(&img, pxfactor, interpolfilter);
    let fin = apply_palette(pix, pal);
    saveImage(&fin, "png", outputfilepath)
}

fn px_gen_raw_pal(inputfilepath: &str) {
    let img = loadImage(inputfilepath);
    let raw = generateRawPalette(&img);
    println!("{}", raw.len())
}