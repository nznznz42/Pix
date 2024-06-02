use image::Rgb;

pub fn euclideanDistance(color1: &Rgb<u8>, color2: &Rgb<u8>) -> f32 {
    let r = (color2[0] as f32 - color1[0] as f32).powf(2f32);
    let g = (color2[1] as f32 - color1[1] as f32).powf(2f32);
    let b = (color2[2] as f32 - color1[2] as f32).powf(2f32);

    let dist = (r + g + b).sqrt();
    return dist;
}