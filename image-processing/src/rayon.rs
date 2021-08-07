use raster::filter;
use raster::Image;
use std::time::{SystemTime};

use rayon::prelude::*;

pub fn rayon(dir_name: &str, threads: usize) {
    rayon::ThreadPoolBuilder::new().num_threads(threads*5).build_global().unwrap();

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

        let _collection: Vec<Image>  = all_images.into_iter()
            .par_bridge()
            .filter_map(|mut image: Image| { 
                filter::saturation(&mut image, 0.2).unwrap();
                Some(image)
            })
            .filter_map(|mut image: Image| { 
                filter::emboss(&mut image).unwrap();
                Some(image)
            })
            .filter_map(|mut image: Image| { 
                filter::gamma(&mut image, 2.0).unwrap();
                Some(image)
            })
            .filter_map(|mut image: Image| { 
                filter::sharpen(&mut image).unwrap();
                Some(image)
            })
            .filter_map(|mut image: Image| { 
                filter::grayscale(&mut image).unwrap();
                Some(image)
            })
            .collect();

    let system_duration = start.elapsed().expect("Failed to get render time?");
    let in_sec = system_duration.as_secs() as f64 + system_duration.subsec_nanos() as f64 * 1e-9;
    println!("Execution time: {} sec", in_sec);
}