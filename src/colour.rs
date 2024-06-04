use image::Rgb;
use rand::prelude::IteratorRandom;

pub fn euclideanDistance(color1: &Rgb<u8>, color2: &Rgb<u8>) -> f32 {
    let r = (color2[0] as f32 - color1[0] as f32).powf(2f32);
    let g = (color2[1] as f32 - color1[1] as f32).powf(2f32);
    let b = (color2[2] as f32 - color1[2] as f32).powf(2f32);

    let dist = (r + g + b).sqrt();
    return dist;
}

pub enum SelectionStrategy {
    Random,
    Average,
    KMeans,
    Median
}
pub fn selectRandomly(colors: &Vec<Rgb<u8>>, num_colours: usize) -> Vec<Rgb<u8>> {
    let mut rng = rand::thread_rng();
    colors.iter().choose_multiple(&mut rng, num_colours)
        .into_iter()
        .cloned()
        .collect()
}

pub fn selectAverage(colors: &Vec<Rgb<u8>>, num_colours: usize) -> Vec<Rgb<u8>> {
    let chunk_size = (colors.len() / num_colours).max(1);
    colors.chunks(chunk_size)
        .map(|chunk| {
            let chunk_vec: Vec<Rgb<u8>> = chunk.to_vec();
            let (sum_r, sum_g, sum_b, count) = sumFoldAndCount(&chunk_vec);
            Rgb([
                (sum_r / count) as u8,
                (sum_g / count) as u8,
                (sum_b / count) as u8
            ])
        })
        .collect()
}

pub fn selectKMeans(colors: &Vec<Rgb<u8>>, num_colours: usize) -> Vec<Rgb<u8>> {
    let mut rng = rand::thread_rng();

    let mut centroids: Vec<Rgb<u8>> = colors.iter()
        .choose_multiple(&mut rng, num_colours)
        .into_iter()
        .cloned()
        .collect();

    let mut clusters = vec![vec![]; num_colours];
    let mut assignments = vec![0; colors.len()];

    loop {
        for cluster in clusters.iter_mut() {
            cluster.clear();
        }

        for (i, color) in colors.iter().enumerate() {
            let mut min_distance = f32::MAX;
            let mut min_index = 0;

            for (j, centroid) in centroids.iter().enumerate() {
                let distance = euclideanDistance(color, centroid);
                if distance < min_distance {
                    min_distance = distance;
                    min_index = j;
                }
            }

            clusters[min_index].push(*color);
            assignments[i] = min_index;
        }

        let mut new_centroids = Vec::with_capacity(num_colours);
        for cluster in clusters.iter() {
            if cluster.is_empty() {
                new_centroids.push(Rgb([0, 0, 0]));
            } else {
                let (sum_r, sum_g, sum_b, count) = sumFoldAndCount(cluster);
                new_centroids.push(Rgb([
                    (sum_r / count) as u8,
                    (sum_g / count) as u8,
                    (sum_b / count) as u8
                ]));
            }
        }

        if centroids == new_centroids {
            break;
        } else {
            centroids = new_centroids;
        }
    }

    centroids
}

pub fn selectMedian(colors: &Vec<Rgb<u8>>, num_colours: usize) -> Vec<Rgb<u8>> {
    let mut boxes = vec![colors.to_vec()];
    while boxes.len() < num_colours {
        let mut new_boxes = vec![];

        for b in boxes {
            if b.len() <= 1 {
                new_boxes.push(b);
                continue;
            }

            let (min_r, min_g, min_b, max_r, max_g, max_b) = b.iter().fold(
                (255, 255, 255, 0, 0, 0),
                |(min_r, min_g, min_b, max_r, max_g, max_b), color| {
                    (
                        min_r.min(color[0]),
                        min_g.min(color[1]),
                        min_b.min(color[2]),
                        max_r.max(color[0]),
                        max_g.max(color[1]),
                        max_b.max(color[2]),
                    )
                },
            );

            let r_range = max_r - min_r;
            let g_range = max_g - min_g;
            let b_range = max_b - min_b;

            let sort_channel = if r_range >= g_range && r_range >= b_range {
                0
            } else if g_range >= r_range && g_range >= b_range {
                1
            } else {
                2
            };

            let mut b = b;
            b.sort_by_key(|color| color[sort_channel]);
            let mid = b.len() / 2;
            let (box1, box2) = b.split_at(mid);

            new_boxes.push(box1.to_vec());
            new_boxes.push(box2.to_vec());
        }

        boxes = new_boxes;
    }

    boxes.iter().map(|b| {
        let (sum_r, sum_g, sum_b, count) = sumFoldAndCount(b);
        Rgb([
            (sum_r / count) as u8,
            (sum_g / count) as u8,
            (sum_b / count) as u8
        ])
    }).collect()
}

fn sumFoldAndCount(cluster: &Vec<Rgb<u8>>) -> (u32, u32, u32, u32) {
    cluster.iter().fold((0u32, 0u32, 0u32, 0u32), |acc, color| {
        (
            acc.0 + color[0] as u32,
            acc.1 + color[1] as u32,
            acc.2 + color[2] as u32,
            acc.3 + 1,
        )
    })
}