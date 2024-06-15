use image::{DynamicImage, GenericImage, GenericImageView, Rgb, Rgba};
use rand::Rng;
use crate::colour::calculate_avg_distance_in_palette;
use crate::utils::{calculate_error, clamp, distribute_error, find_closest_color, gen_blue_noise_threshold, generate_raw_palette};

#[derive(Copy, Clone)]
pub enum BlueNoiseThreshold {
    LOW, //0-85 (inclusive)
    MEDIUM, //85-170 (inclusive)
    HIGH, //170 -255 (inclusive)
}
#[derive(Copy, Clone)]
pub enum DitherMode {
    BAYER(u32),
    //BLUENOISE(BlueNoiseThreshold),
    //ATKINSON,
    FLOYDSTEINBERG,
    //JARVISJUDICENINKE,
    //SIERRA,
}

pub struct Ditherer {
    pub dither_mode: DitherMode,
    pub dither_fn: Box<dyn Fn(&mut DynamicImage)>
}

impl Ditherer {

    pub fn new(dither_mode: DitherMode) -> Ditherer {
        let mode = dither_mode.clone();
        let dither_fn = Self::get_dither_fn(mode);
        return Ditherer {
            dither_mode: dither_mode,
            dither_fn
        }
    }

    fn get_dither_fn(mode: DitherMode) -> Box<dyn Fn(&mut DynamicImage) + 'static> {
        match mode {
            DitherMode::BAYER(order) => Box::new(move |image: &mut DynamicImage| {bayer_dithering(image, order)}),
            DitherMode::FLOYDSTEINBERG => Box::new(move |image: &mut DynamicImage| {floyd_steinberg_dither(image)}),
            //DitherMode::JARVISJUDICENINKE => Box::new(move |image: &mut DynamicImage| {jarvis_judice_ninke_dither(image)}),
            //DitherMode::SIERRA => Box::new(move |image: &mut DynamicImage| {sierra_dither(image)}),
            //DitherMode::ATKINSON => Box::new(move |image: &mut DynamicImage| {atkinson_dither(image)}),
            //DitherMode::BLUENOISE(threshold) => Box::new(move |image: &mut DynamicImage| {blue_noise_dither(image, threshold)})
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

fn bayer_threshold(matrix: &Vec<Vec<f32>>, order:u32, x: u32, y: u32) -> f32 {
    let size = matrix.len() as u32;
    let value = matrix[(y % size) as usize][(x % size) as usize];
    let max_value = (1 << (2 * order)) as f32; // equivalent to 2^(2*order)
    (value / max_value) * 255.0
}

pub fn floyd_steinberg_dither(image: &mut DynamicImage) {
    let (width, height) = image.dimensions();
    let pal = generate_raw_palette(image);
    let mut pixels = image.to_rgb8();


    for y in 0..height {
        for x in 0..width {
            let old_color = pixels.get_pixel(x, y).clone();
            let new_color = find_closest_color(&old_color, &pal);
            pixels.put_pixel(x, y, new_color);
            let error = calculate_error(&old_color, &new_color);

            //dirty hack for now
            if x + 1 < width {
                let mut right = pixels.get_pixel(x + 1, y).clone();
                right[0] = right[0] + (error[0] as f32 * (7f32 / 16f32)) as u8;
                clamp(right[0] as i16, 0, 255);
                right[1] = right[1] + (error[1] as f32 * (7f32 / 16f32)) as u8;
                clamp(right[1] as i16, 0, 255);
                right[2] = right[2] + (error[2] as f32 * (7f32 / 16f32)) as u8;
                clamp(right[2] as i16, 0, 255);
                pixels.put_pixel(x + 1, y, right);
            }

            if x > 0 && y + 1 < height {
                let mut bottomleft = pixels.get_pixel(x - 1, y + 1).clone();
                bottomleft[0] = bottomleft[0] + (error[0] as f32 * (3f32 / 16f32)) as u8;
                clamp(bottomleft[0] as i16, 0, 255);
                bottomleft[1] = bottomleft[1] + (error[1] as f32 * (3f32 / 16f32)) as u8;
                clamp(bottomleft[1] as i16, 0, 255);
                bottomleft[2] = bottomleft[2] + (error[2] as f32 * (3f32 / 16f32)) as u8;
                clamp(bottomleft[2] as i16, 0, 255);
                pixels.put_pixel(x - 1, y + 1, bottomleft);
            }

            if y + 1 < height {
                let mut bottom = pixels.get_pixel(x, y + 1).clone();
                bottom[0] = bottom[0] + (error[0] as f32 * (5f32 / 16f32)) as u8;
                clamp(bottom[0] as i16, 0, 255);
                bottom[1] = bottom[1] + (error[1] as f32 * (5f32 / 16f32)) as u8;
                clamp(bottom[1] as i16, 0, 255);
                bottom[2] = bottom[2] + (error[2] as f32 * (5f32 / 16f32)) as u8;
                clamp(bottom[2] as i16, 0, 255);
                pixels.put_pixel(x, y + 1, bottom);
            }

            if x + 1 < width && y + 1 < height {
                let mut bottomright = pixels.get_pixel(x + 1, y + 1).clone();
                bottomright[0] = bottomright[0] + (error[0] as f32 * (1f32 / 16f32)) as u8;
                clamp(bottomright[0] as i16, 0, 255);
                bottomright[1] = bottomright[1] + (error[1] as f32 * (1f32 / 16f32)) as u8;
                clamp(bottomright[1] as i16, 0, 255);
                bottomright[2] = bottomright[2] + (error[2] as f32 * (1f32 / 16f32)) as u8;
                clamp(bottomright[2] as i16, 0, 255);
                pixels.put_pixel(x + 1, y + 1, bottomright);
            }
        }
    }

    *image = DynamicImage::ImageRgb8(pixels);
}

/*fn blue_noise_dither(image: &mut DynamicImage, threshold: BlueNoiseThreshold) {
    let pal = generate_raw_palette(image);
    let (width, height) = image.dimensions();
    let mut pixels = image.to_rgb8();
    let mut rng = rand::thread_rng();
    let noise_threshold = gen_blue_noise_threshold(threshold);  // Example threshold, can be adjusted

    for y in 0..height {
        for x in 0..width {
            let old_color = pixels.get_pixel(x, y);
            let new_color = find_closest_color(old_color, &pal);
            let error = calculate_error(old_color, &new_color);

            let noise_value: u8 = rng.gen();
            if noise_value > noise_threshold {
                let new_color = Rgb([
                    (new_color[0] as i16 + error[0] / 4) as u8,
                    (new_color[1] as i16 + error[1] / 4) as u8,
                    (new_color[2] as i16 + error[2] / 4) as u8,
                    //new_color[3],
                ]);
                pixels.put_pixel(x, y, new_color);
            } else {
                pixels.put_pixel(x, y, new_color);
            }

            distribute_error(&mut pixels, x, y, &error, &[
                (1, 0, 7.0 / 16.0),
                (-1, 1, 3.0 / 16.0),
                (0, 1, 5.0 / 16.0),
                (1, 1, 1.0 / 16.0),
            ]);
        }
    }
}*/

