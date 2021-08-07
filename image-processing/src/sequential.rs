use raster::filter;
use raster::Image;
use std::time::{SystemTime};

pub fn sequential(dir_name: &str) {
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

    for mut image in all_images.into_iter() {
        filter::saturation(&mut image, 0.2).unwrap();
        filter::emboss(&mut image).unwrap();
        filter::gamma(&mut image, 2.0).unwrap();
        filter::sharpen(&mut image).unwrap();
        filter::grayscale(&mut image).unwrap();
    }

    let system_duration = start.elapsed().expect("Failed to get render time?");
    let in_sec = system_duration.as_secs() as f64 + system_duration.subsec_nanos() as f64 * 1e-9;
    println!("Execution time: {} sec", in_sec);
}