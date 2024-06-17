use ::image::imageops::FilterType;

use crate::colour::SelectionStrategy;
use crate::ditherer::{BlueNoiseThreshold, DitherMode};
use crate::image::{Image};
use crate::palette::Palette;

mod palette;
mod colour;
mod image;
mod utils;
mod ditherer;
mod consts;

fn main() {
    let inpath = "./input/angry_bird.jpg";
    let outpath = "./output/angry_bird4.png";
    let palname = "2bit-demichrome.hex";
    let px_factor = 5;
    let sel_fac = SelectionStrategy::Random;
    let interpolfilter = FilterType::Nearest;
    px_std(inpath, outpath, palname, px_factor, interpolfilter);

}

fn px_std(inputfilepath: &str, outputfilepath: &str, palette: &'static str, pxfactor: u32, interpolfilter: FilterType) {
    let mut img = Image::new(inputfilepath);
    let pal = Palette::new(&palette);
    img.pixelate(5, interpolfilter);
    let mode= DitherMode::STEVENSONARCE(palette);
    img.dither(mode);
    img.save_image(Some(outputfilepath));
}

fn px_gen_pal(inputfilepath: &str, palettename: &str, numcolours: usize, selection_strategy: SelectionStrategy) {
    let palname = palettename.to_string();
    let pal = Palette::generate_palette(inputfilepath, palname, numcolours, selection_strategy);
    pal.save_palette(None);
    println!("GENERATED PALETTE SIZE: {}", &pal.colours.len())
}

