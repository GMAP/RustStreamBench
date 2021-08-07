use raster::filter;
use raster::Image;
use std::time::{SystemTime};

use rust_spp::*;

pub fn rust_ssp(dir_name: &str, threads: usize) {
    let dir_entries = std::fs::read_dir(format!("{}", dir_name));
    let mut all_images: Vec<Image> = Vec::new();

    for entry in dir_entries.unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().is_none() {
            continue;
        }
        all_images.push(raster::open(path.to_str().unwrap()).unwrap());
    }

    let start = SystemTime::now();

    let pipeline = pipeline![
            parallel!(move |mut image: Image| {
                filter::saturation(&mut image, 0.2).unwrap();
                Some(image)
            }, threads as i32),
            parallel!(move |mut image: Image| {
                filter::emboss(&mut image).unwrap();
                Some(image)
            }, threads as i32),
            parallel!(move |mut image: Image| {
                filter::gamma(&mut image, 2.0).unwrap();
                Some(image)
            }, threads as i32),
            parallel!(move |mut image: Image| {
                filter::sharpen(&mut image).unwrap();
                Some(image)
            }, threads as i32),
            parallel!(move |mut image: Image| {
                filter::grayscale(&mut image).unwrap();
                Some(image)
            }, threads as i32),
            collect!()
        ];


    for image in all_images.into_iter() {
        pipeline.post(image).unwrap();
    }

    let _collection = pipeline.collect();

    let system_duration = start.elapsed().expect("Failed to get render time?");
    let in_sec = system_duration.as_secs() as f64 + system_duration.subsec_nanos() as f64 * 1e-9;
    println!("Execution time: {} sec", in_sec);
}