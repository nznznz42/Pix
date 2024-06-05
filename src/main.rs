use ::image::imageops::FilterType;
use crate::colour::SelectionStrategy;

use crate::image::{apply_palette, loadImage, pixelateImage, saveImage};
use crate::palette::{Palette};

mod palette;
mod colour;
mod image;
mod utils;

fn main() {
    let inpath = "./input/creek.jpg";
    let outpath = "./output/Mountain.png";
    let palname = "hollow.hex";
    let pxFactor = 2;
    let selFac = SelectionStrategy::Random;
    //px_std(inpath, outpath, palname, pxFactor, FilterType::Nearest)
    px_gen_pal(outpath, "test.hex", 12, selFac)
}

fn px_std(inputfilepath: &str, outputfilepath: &str, palette: &str, pxfactor: u32, interpolfilter: FilterType) {
    let img = loadImage(inputfilepath);
    let pal = Palette::new(&palette);
    let pix = pixelateImage(&img, pxfactor, interpolfilter);
    let fin = apply_palette(pix, pal);
    saveImage(&fin, "png", outputfilepath)
}

fn px_gen_pal(inputfilepath: &str, palettename: &str, numcolours: usize, selection_strategy: SelectionStrategy) {
    let palname = palettename.to_string();
    let pal = Palette::generatePalette(inputfilepath, palname, numcolours, selection_strategy);
    pal.savePalette();
    println!("GENERATED PALETTE SIZE: {}", &pal.colours.len())
}