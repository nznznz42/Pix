use ::image::imageops::FilterType;

use crate::colour::SelectionStrategy;
use crate::image::Image;
use crate::palette::Palette;

mod palette;
mod colour;
mod image;
mod utils;

fn main() {
    let inpath = "./input/creek.jpg";
    let outpath = "./output/creekTest.png";
    let palname = "hollow.hex";
    let pxFactor = 15;
    let selFac = SelectionStrategy::Random;
    let interpolfilter = FilterType::Nearest;
    //px_std(inpath, outpath, palname, pxFactor, interpolfilter)
    //px_gen_pal(outpath, "test.hex", 12, selFac)
    //px_pix_test(inpath, outpath, pxFactor, interpolfilter)
}

fn px_std(inputfilepath: &str, outputfilepath: &str, palette: &str, pxfactor: u32, interpolfilter: FilterType) {
    let mut img = Image::new(inputfilepath);
    let pal = Palette::new(&palette);
    img.pixelate(pxfactor, interpolfilter);
    img.applyPalette(pal);
    img.saveImage(Some(outputfilepath));
}

fn px_gen_pal(inputfilepath: &str, palettename: &str, numcolours: usize, selection_strategy: SelectionStrategy) {
    let palname = palettename.to_string();
    let pal = Palette::generatePalette(inputfilepath, palname, numcolours, selection_strategy);
    pal.savePalette(None);
    println!("GENERATED PALETTE SIZE: {}", &pal.colours.len())
}

fn px_pix_test(inputfilepath: &str, outputfilepath: &str, pxfactor: u32, interpolfilter: FilterType) {
    let mut img = Image::new(inputfilepath);
    img.pixelate(pxfactor, interpolfilter);
    img.saveImage(Some(outputfilepath));
}