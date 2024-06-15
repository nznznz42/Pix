use std::fmt::Error;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

use image::{DynamicImage, ExtendedColorType, GenericImage, GenericImageView, ImageFormat, save_buffer_with_format};
use image::imageops::FilterType;

use crate::colour::{euclidean_distance};
use crate::ditherer::{Ditherer, DitherMode, floyd_steinberg_dither};
use crate::palette::Palette;
use crate::utils::available_threads;

pub enum Extension {
    PNG,
    JPG,
    SVG,
    AVIF,
    BMP,
    GIF,
    QOI,
    TIFF,
    WEBP,
}

impl Extension {
    pub fn new(ext: &str) -> Result<Extension, Error> {
        let extension = match ext {
            "png" => Extension::PNG,
            "jpg" => Extension::JPG,
            "jpeg" => Extension::JPG,
            "qoi" => Extension::QOI,
            "gif" => Extension::GIF,
            "bmp" => Extension::BMP,
            "svg" => Extension::SVG,
            "webp" => Extension::WEBP,
            "avif" => Extension::AVIF,
            "tiff" => Extension::TIFF,

            _ => return Err(Error)
        };

        return Ok(extension);
    }

    pub fn to_string(extension: &Extension) -> String {
        let ext = match extension {
            Extension::PNG => "png".to_string(),
            Extension::JPG => "jpg".to_string(),
            Extension::QOI => "qoi".to_string(),
            Extension::GIF => "gif".to_string(),
            Extension::BMP => "bmp".to_string(),
            Extension::SVG => "svg".to_string(),
            Extension::WEBP => "webp".to_string(),
            Extension::AVIF => "avif".to_string(),
            Extension::TIFF => "tiff".to_string(),
        };

        return ext;
    }

    pub fn change_file_extension(file_path: &str, new_extension: Extension) -> String {
        let path = Path::new(file_path);
        let mut new_path = PathBuf::from(path);

        if new_path.set_extension(Extension::to_string(&new_extension)) {
            new_path.to_str().unwrap().to_string()
        } else {
            file_path.to_string()
        }
    }
}

pub struct Image {
    pub filename: String,
    pub extension: Extension,
    pub data: DynamicImage,
}

impl Image {
    pub fn new(filepath: &str) -> Image {
        let path = Path::new(filepath);
        let name = path.file_stem().unwrap().to_str().unwrap().to_string();
        let extstr = path.extension().unwrap().to_str().unwrap();
        let ext = Extension::new(extstr).expect("ERROR: UNSUPPORTED EXTENSION");
        let data = load_image(filepath);

        return Image {
            filename: name,
            extension: ext,
            data,
        };
    }

    pub fn pixelate(&mut self, scale: u32, filter: FilterType) {
        pixelate_image(&mut self.data, scale, filter)
    }

    pub fn apply_palette(&mut self, palette: Palette) {
        self.data = apply_palette(self.data.clone(), palette)
    }

    // Currently only jpeg, png, ico, pnm, bmp, exr and tiff files are supported.
    pub fn save_image(&self, file_path: Option<&str>) {
        let ext = Extension::to_string(&self.extension);
        let extstr = ext.as_str();
        let path = file_path.unwrap_or_else(|| "./output");
        if path.eq("./output") {
            let mut fullpath = Path::new(path).join(&self.filename);
            fullpath.set_extension(extstr);
            let new = fullpath.to_str().unwrap();
            save_image(&self.data, extstr, &new)
        } else {
            save_image(&self.data, extstr, path)
        }
    }

    pub fn resize(infilepath: &str, outfilepath: Option<&str>, n_width: u32, n_height: u32, filter: FilterType) {
        let img = load_image(infilepath);
        let path = outfilepath.unwrap_or_else(|| "./output");
        img.resize(n_width, n_height, filter);
    }

    pub fn dither(&mut self, mode: DitherMode) {
        let ditherer = Ditherer::new(mode);
        (ditherer.dither_fn)(&mut self.data)
    }
}

fn load_image(file_path: &str) -> DynamicImage {
    let img = image::open(file_path).expect("ERROR: Unable to open image");
    return img;
}

fn save_image(img: &DynamicImage, extension: &str, file_path: &str) {
    let imgbuf = img.as_bytes();
    let width = img.width();
    let height = img.height();
    let colour = ExtendedColorType::Rgb8;
    let format = ImageFormat::from_extension(extension);
    save_buffer_with_format(file_path, imgbuf, width, height, colour, format.unwrap()).expect("ERROR: Unable to save image");
}

fn pixelate_image(img: &mut DynamicImage, scale: u32, filter: FilterType) {
    let (width, height) = img.dimensions();

    let new_width = width / scale;
    let new_height = height / scale;

    let resized_down = img.resize_exact(new_width, new_height, filter);

    let pixelated = resized_down.resize_exact(width, height, filter);

    *img = pixelated;
}

fn apply_palette_partial(image: &mut DynamicImage, palette: &Palette, start_row: u32, end_row: u32) {
    let (width, height) = image.dimensions();
    let mut rgb_image = image.to_rgb8();

    for y in start_row..end_row.min(height) {
        for x in 0..width {
            let mut pixel = rgb_image.get_pixel(x, y);
            let mut min_distance = f64::MAX;
            let mut best_match = *pixel;

            for color in &palette.colours {
                let distance = euclidean_distance(pixel, &color);
                if distance < min_distance as f32 {
                    min_distance = distance as f64;
                    best_match = *color;
                }
            }

            rgb_image.put_pixel(x, y, best_match);
        }
    }

    *image = DynamicImage::ImageRgb8(rgb_image);
}

fn apply_palette(image: DynamicImage, palette: Palette) -> DynamicImage {
    let num_threads = available_threads();
    let (width, height) = image.dimensions();
    let rows_per_thread = (height as usize + num_threads - 1) / num_threads;

    let image = Arc::new(Mutex::new(image));
    let palette = Arc::new(palette);

    let mut handles = vec![];

    for i in 0..num_threads {
        let image = Arc::clone(&image);
        let palette = Arc::clone(&palette);
        let start_row = (i * rows_per_thread) as u32;
        let end_row = ((i + 1) * rows_per_thread).min(height as usize) as u32;

        let handle = thread::spawn(move || {
            let mut image = image.lock().unwrap();
            apply_palette_partial(&mut image, &palette, start_row, end_row);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    return Arc::try_unwrap(image).unwrap().into_inner().unwrap();
}



