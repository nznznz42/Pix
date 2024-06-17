use image::{DynamicImage, GenericImage, GenericImageView, Rgb, Rgba};
use rand::Rng;

use crate::colour::calculate_avg_distance_in_palette;
use crate::consts::{DIFF_MAT_ATKINSON, DIFF_MAT_BURKES, DIFF_MAT_FAN, DIFF_MAT_FLOYD_STEINBERG, DIFF_MAT_IMPROVED_STUCKI, DIFF_MAT_JARVIS_JUDICE_NINKE, DIFF_MAT_K3M, DIFF_MAT_LI_WAN, DIFF_MAT_PJARRI, DIFF_MAT_SHIAU_FAN, DIFF_MAT_SIERRA, DIFF_MAT_SIERRA_LITE, DIFF_MAT_STEVENSON_ARCE, DIFF_MAT_STUCKI, DIFF_MAT_TWO_ROW_SIERRA};
use crate::palette::Palette;
use crate::utils::{calculate_error, diffuse_error, find_closest_color, gen_blue_noise_threshold, generate_raw_palette};

#[derive(Copy, Clone)]
pub enum BlueNoiseThreshold {
    LOW, //0-85 (inclusive)
    MEDIUM, //85-170 (inclusive)
    HIGH, //170 -255 (inclusive)
}

#[derive(Copy, Clone)]
pub enum DitherMode {
    BAYER(u32),
    BLUENOISE(BlueNoiseThreshold, &'static str),
    FLOYDSTEINBERG(&'static str),
    ATKINSON(&'static str),
    JARVISJUDICENINKE(&'static str),
    SIERRA(&'static str),
    STUCKI(&'static str),
    BURKES(&'static str),
    STEVENSONARCE(&'static str),
    SIERRA2(&'static str),
    SIERRALITE(&'static str),
    FAN(&'static str),
    K3M(&'static str),
    LIWAN(&'static str),
    PJARRI(&'static str),
    SHIAUFAN(&'static str),
    IMPROVEDSTUCKI(&'static str),
}

pub struct Ditherer {
    pub dither_mode: DitherMode,
    pub dither_fn: Box<dyn Fn(&mut DynamicImage)>,
}

impl Ditherer {
    pub fn new(dither_mode: DitherMode) -> Ditherer {
        let mode = dither_mode.clone();
        let dither_fn = Self::get_dither_fn(mode);
        return Ditherer {
            dither_mode: dither_mode,
            dither_fn,
        };
    }

    fn get_dither_fn(mode: DitherMode) -> Box<dyn Fn(&mut DynamicImage) + 'static> {
        match mode {
            DitherMode::BAYER(order) => Box::new(move |image: &mut DynamicImage| { bayer_dithering(image, order) }),
            DitherMode::BLUENOISE(threshold, palette) => Box::new(move |image: &mut DynamicImage| { blue_noise_dither(image, threshold, palette) }),
            DitherMode::FLOYDSTEINBERG(palette) => Box::new(move |image: &mut DynamicImage| { generic_error_diffusion_dither(image, palette, &DIFF_MAT_FLOYD_STEINBERG) }),
            DitherMode::ATKINSON(palette) => Box::new(move |image: &mut DynamicImage| { generic_error_diffusion_dither(image, palette, &DIFF_MAT_ATKINSON) }),
            DitherMode::JARVISJUDICENINKE(palette) => Box::new(move |image: &mut DynamicImage| { generic_error_diffusion_dither(image, palette, &DIFF_MAT_JARVIS_JUDICE_NINKE) }),
            DitherMode::SIERRA(palette) => Box::new(move |image: &mut DynamicImage| { generic_error_diffusion_dither(image, palette, &DIFF_MAT_SIERRA) }),
            DitherMode::STUCKI(palette) => Box::new(move |image: &mut DynamicImage| { generic_error_diffusion_dither(image, palette, &DIFF_MAT_STUCKI) }),
            DitherMode::BURKES(palette) => Box::new(move |image: &mut DynamicImage| { generic_error_diffusion_dither(image, palette, &DIFF_MAT_BURKES) }),
            DitherMode::STEVENSONARCE(palette) => Box::new(move |image: &mut DynamicImage| { generic_error_diffusion_dither(image, palette, &DIFF_MAT_STEVENSON_ARCE) }),
            DitherMode::SIERRA2(palette) => Box::new(move |image: &mut DynamicImage| { generic_error_diffusion_dither(image, palette, &DIFF_MAT_TWO_ROW_SIERRA) }),
            DitherMode::SIERRALITE(palette) => Box::new(move |image: &mut DynamicImage| { generic_error_diffusion_dither(image, palette, &DIFF_MAT_SIERRA_LITE) }),
            DitherMode::FAN(palette) => Box::new(move |image: &mut DynamicImage| { generic_error_diffusion_dither(image, palette, &DIFF_MAT_FAN) }),
            DitherMode::K3M(palette) => Box::new(move |image: &mut DynamicImage| { generic_error_diffusion_dither(image, palette, &DIFF_MAT_K3M) }),
            DitherMode::LIWAN(palette) => Box::new(move |image: &mut DynamicImage| { generic_error_diffusion_dither(image, palette, &DIFF_MAT_LI_WAN) }),
            DitherMode::PJARRI(palette) => Box::new(move |image: &mut DynamicImage| { generic_error_diffusion_dither(image, palette, &DIFF_MAT_PJARRI) }),
            DitherMode::SHIAUFAN(palette) => Box::new(move |image: &mut DynamicImage| { generic_error_diffusion_dither(image, palette, &DIFF_MAT_SHIAU_FAN) }),
            DitherMode::IMPROVEDSTUCKI(palette) => Box::new(move |image: &mut DynamicImage| { generic_error_diffusion_dither(image, palette, &DIFF_MAT_IMPROVED_STUCKI) }),
        }
    }
}

fn bayer_dithering(image: &mut DynamicImage, order: u32) {
    let (width, height) = image.dimensions();
    let pal = generate_raw_palette(image);
    let avg = calculate_avg_distance_in_palette(&pal);
    let mat = generate_bayer_matrix(order);

    for y in 0..height {
        for x in 0..width {
            let pixel = image.get_pixel(x, y);
            let threshold = bayer_threshold(&mat, order, x, y);

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

pub fn generate_bayer_matrix(order: u32) -> Vec<Vec<f32>> {
    if order == 0 {
        return vec![vec![0f32]];
    }

    let previous_matrix = generate_bayer_matrix(order - 1);
    let size = previous_matrix.len();
    let new_size = size * 2;
    let mut matrix = vec![vec![0f32; new_size]; new_size];

    for i in 0..size {
        for j in 0..size {
            let value = previous_matrix[i][j];
            matrix[i][j] = 4.0 * value;
            matrix[i][j + size] = 4.0 * value + 2.0;
            matrix[i + size][j] = 4.0 * value + 3.0;
            matrix[i + size][j + size] = 4.0 * value + 1.0;
        }
    }

    matrix
}

fn bayer_threshold(matrix: &Vec<Vec<f32>>, order: u32, x: u32, y: u32) -> f32 {
    let size = matrix.len() as u32;
    let value = matrix[(y % size) as usize][(x % size) as usize];
    let max_value = (1 << (2 * order)) as f32; // equivalent to 2^(2*order)
    (value / max_value) * 255.0
}

pub fn generic_error_diffusion_dither(image: &mut DynamicImage, palette: &str, diff_mat: &[((i32, i32), f32)]) {
    let (width, height) = image.dimensions();
    let pal = Palette::new(palette).colours;
    let mut pixels = image.to_rgb8();

    for y in 0..height {
        for x in 0..width {
            let old_color = pixels.get_pixel(x, y).clone();
            let new_color = find_closest_color(&old_color, &pal);
            pixels.put_pixel(x, y, new_color);
            let error = calculate_error(&old_color, &new_color);

            diffuse_error(x, y, diff_mat, error, &mut pixels);
        }
    }
    *image = DynamicImage::ImageRgb8(pixels);
}

fn blue_noise_dither(image: &mut DynamicImage, threshold: BlueNoiseThreshold, palette: &str) {
    let pal = Palette::new(palette).colours;
    let (width, height) = image.dimensions();
    let mut pixels = image.to_rgb8();
    let mut rng = rand::thread_rng();
    let noise_threshold = gen_blue_noise_threshold(threshold);  // Example threshold, can be adjusted

    for y in 0..height {
        for x in 0..width {
            let old_color = pixels.get_pixel(x, y);
            let new_color = find_closest_color(old_color, &pal);
            let error = calculate_error(&old_color, &new_color);

            let noise_value: u8 = rng.gen();
            if noise_value > noise_threshold {
                let new_color = Rgb([
                    (new_color[0] as i16 + error[0] / 4) as u8,
                    (new_color[1] as i16 + error[1] / 4) as u8,
                    (new_color[2] as i16 + error[2] / 4) as u8,
                ]);
                pixels.put_pixel(x, y, new_color);
            } else {
                pixels.put_pixel(x, y, new_color);
            }

            diffuse_error(x, y, &DIFF_MAT_FLOYD_STEINBERG, error, &mut pixels);
        }
    }
    *image = DynamicImage::ImageRgb8(pixels);
}


