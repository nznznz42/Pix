use std::fmt::Error;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

use image::{DynamicImage, ExtendedColorType, GenericImage, GenericImageView, ImageFormat, Rgba, save_buffer_with_format};
use image::imageops::FilterType;

use crate::colour::{calculate_avg_distance_in_palette, euclidean_distance};
use crate::palette::Palette;
use crate::utils::available_threads;

#[derive(Eq, PartialEq)]
pub enum DitherMat {
    BAYER4x4,
    BAYER8x8,
    BAYER16x16,
}

pub const BAYER_MATRIX_4X4: [[f32; 4]; 4] = [
    [0.0, 8.0, 2.0, 10.0],
    [12.0, 4.0, 14.0, 6.0],
    [3.0, 11.0, 1.0, 9.0],
    [15.0, 7.0, 13.0, 5.0],
];
pub const BAYER_MATRIX_8X8: [[f32; 8]; 8] = [
    [0.0, 32.0, 8.0, 40.0, 2.0, 34.0, 10.0, 42.0],
    [48.0, 16.0, 56.0, 24.0, 50.0, 18.0, 58.0, 26.0],
    [12.0, 44.0, 4.0, 36.0, 14.0, 46.0, 6.0, 38.0],
    [60.0, 28.0, 52.0, 20.0, 62.0, 30.0, 54.0, 22.0],
    [3.0, 35.0, 11.0, 43.0, 1.0, 33.0, 9.0, 41.0],
    [51.0, 19.0, 59.0, 27.0, 49.0, 17.0, 57.0, 25.0],
    [15.0, 47.0, 7.0, 39.0, 13.0, 45.0, 5.0, 37.0],
    [63.0, 31.0, 55.0, 23.0, 61.0, 29.0, 53.0, 21.0],
];

pub const BAYER_MATRIX_16X16: [[f32; 16]; 16] = [
    [0.0, 192.0, 48.0, 240.0, 12.0, 204.0, 60.0, 252.0, 3.0, 195.0, 51.0, 243.0, 15.0, 207.0, 63.0, 255.0],
    [128.0, 64.0, 176.0, 112.0, 140.0, 76.0, 188.0, 124.0, 131.0, 67.0, 179.0, 115.0, 143.0, 79.0, 191.0, 127.0],
    [32.0, 224.0, 16.0, 208.0, 44.0, 236.0, 28.0, 220.0, 35.0, 227.0, 19.0, 211.0, 47.0, 239.0, 31.0, 223.0],
    [160.0, 96.0, 144.0, 80.0, 172.0, 108.0, 156.0, 92.0, 163.0, 99.0, 147.0, 83.0, 175.0, 111.0, 159.0, 95.0],
    [8.0, 200.0, 56.0, 248.0, 4.0, 196.0, 52.0, 244.0, 11.0, 203.0, 59.0, 251.0, 7.0, 199.0, 55.0, 247.0],
    [136.0, 72.0, 184.0, 120.0, 132.0, 68.0, 180.0, 116.0, 139.0, 75.0, 187.0, 123.0, 145.0, 81.0, 193.0, 129.0],
    [40.0, 232.0, 24.0, 216.0, 36.0, 228.0, 20.0, 212.0, 43.0, 235.0, 27.0, 219.0, 39.0, 231.0, 23.0, 215.0],
    [168.0, 104.0, 152.0, 88.0, 164.0, 100.0, 148.0, 84.0, 171.0, 107.0, 155.0, 91.0, 167.0, 103.0, 151.0, 87.0],
    [2.0, 194.0, 50.0, 242.0, 14.0, 206.0, 62.0, 254.0, 1.0, 193.0, 49.0, 241.0, 13.0, 205.0, 61.0, 253.0],
    [130.0, 66.0, 178.0, 114.0, 142.0, 78.0, 190.0, 126.0, 129.0, 65.0, 177.0, 113.0, 141.0, 77.0, 189.0, 125.0],
    [34.0, 226.0, 18.0, 210.0, 46.0, 238.0, 30.0, 222.0, 33.0, 225.0, 17.0, 209.0, 45.0, 237.0, 29.0, 221.0],
    [162.0, 98.0, 146.0, 82.0, 174.0, 110.0, 158.0, 94.0, 161.0, 97.0, 145.0, 81.0, 173.0, 109.0, 157.0, 93.0],
    [10.0, 202.0, 58.0, 250.0, 6.0, 198.0, 54.0, 246.0, 9.0, 201.0, 57.0, 249.0, 5.0, 197.0, 53.0, 245.0],
    [138.0, 74.0, 186.0, 122.0, 144.0, 80.0, 192.0, 128.0, 137.0, 73.0, 185.0, 121.0, 143.0, 79.0, 191.0, 127.0],
    [42.0, 234.0, 26.0, 218.0, 38.0, 230.0, 22.0, 214.0, 41.0, 233.0, 25.0, 217.0, 37.0, 229.0, 21.0, 213.0],
    [170.0, 106.0, 154.0, 90.0, 166.0, 102.0, 150.0, 86.0, 169.0, 105.0, 153.0, 89.0, 165.0, 101.0, 149.0, 85.0],
];


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

    pub fn dither(&mut self, mat: DitherMat, palette: &Palette) {
        apply_ordered_dithering(&mut self.data, mat, &palette)
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

fn apply_ordered_dithering(image: &mut DynamicImage, mat: DitherMat, palette: &&Palette) {
    let (width, height) = image.dimensions();
    let avg = calculate_avg_distance_in_palette(&palette);

    let threshold_fn: fn(u32, u32) -> f32 = match mat {
        DitherMat::BAYER4x4 => bayer4x4_threshold,
        DitherMat::BAYER8x8 => bayer8x8_threshold,
        DitherMat::BAYER16x16 => bayer16x16_threshold,
    };
    for y in 0..height {
        for x in 0..width {
            let pixel = image.get_pixel(x, y);
            let threshold = threshold_fn(x, y);

            let new_pixel = [
                if pixel[0] as f32 > threshold { pixel[0] + (avg * threshold) as u8 } else { 0 },
                if pixel[1] as f32 > threshold { pixel[1] + (avg * threshold) as u8 } else { 0 },
                if pixel[2] as f32 > threshold { pixel[2] + (avg * threshold) as u8 } else { 0 },
                pixel[3],
            ];

            image.put_pixel(x, y, Rgba(new_pixel));
        }
    }
}

fn bayer4x4_threshold(x: u32, y: u32) -> f32 {
    return BAYER_MATRIX_4X4[(y % 4) as usize][(x % 4) as usize] / 16.0 * 255.0;
}

fn bayer8x8_threshold(x: u32, y: u32) -> f32 {
    return BAYER_MATRIX_8X8[(y % 8) as usize][(x % 8) as usize] / 64.0 * 255.0;
}

fn bayer16x16_threshold(x: u32, y: u32) -> f32 {
    return BAYER_MATRIX_16X16[(y % 16) as usize][(x % 16) as usize] / 256.0 * 255.0;
}