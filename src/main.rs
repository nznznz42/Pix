use ::image::imageops::FilterType;

use crate::colour::SelectionStrategy;
use crate::ditherer::{DitherMode, floyd_steinberg_dither};
use crate::image::{Image};
use crate::palette::Palette;

mod palette;
mod colour;
mod image;
mod utils;
mod ditherer;

fn main() {
    let inpath = "./input/road.jpg";
    let outpath = "./output/road_non.png";
    let palname = "slso8.hex";
    let px_factor = 5;
    let sel_fac = SelectionStrategy::Random;
    let interpolfilter = FilterType::Nearest;
    let mode = DitherMode::FLOYDSTEINBERG;
    px_std(inpath, outpath, palname, px_factor, interpolfilter, mode);

}

fn px_std(inputfilepath: &str, outputfilepath: &str, palette: &str, pxfactor: u32, interpolfilter: FilterType, mode: DitherMode) {
    let mut img = Image::new(inputfilepath);
    let pal = Palette::new(&palette);
    let palc = pal.clone();
    img.pixelate(pxfactor, interpolfilter);
    //img.apply_palette(pal);
    //img.dither(mode);
    floyd_steinberg_dither(&mut img.data);
    img.save_image(Some(outputfilepath));
}

fn px_gen_pal(inputfilepath: &str, palettename: &str, numcolours: usize, selection_strategy: SelectionStrategy) {
    let palname = palettename.to_string();
    let pal = Palette::generate_palette(inputfilepath, palname, numcolours, selection_strategy);
    pal.save_palette(None);
    println!("GENERATED PALETTE SIZE: {}", &pal.colours.len())
}

fn px_pix_test(inputfilepath: &str, outputfilepath: &str, pxfactor: u32, interpolfilter: FilterType) {
    let mut img = Image::new(inputfilepath);
    img.pixelate(pxfactor, interpolfilter);
    img.save_image(Some(outputfilepath));
}
