use std::path::Path;
use ::image::imageops::FilterType;
use clap::{Arg, command};

use crate::image::{apply_palette, loadImage, pixelateImage, saveImage};
use crate::palette::{listPalettes, loadPalette};

mod palette;
mod colour;
mod image;

fn main() {
    let inpath = "./input/creek.jpg";
    let outpath = "./output/test.png";
    let palname = "playpal.hex";
    let pxFactor = 5;
    px_std(inpath, outpath, palname, pxFactor, FilterType::Nearest)
}

fn px_std(inputfilepath: &str, outputfilepath: &str, palette: &str, pxfactor: u32, interpolfilter: FilterType) {
    let img = loadImage(inputfilepath);
    let pal = loadPalette(palette);
    let pix = pixelateImage(&img, pxfactor, interpolfilter);
    let fin = apply_palette(pix, pal);
    saveImage(&fin, "png", outputfilepath)
}