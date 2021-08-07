use raster::filter;
use raster::Image;
use std::time::{SystemTime};

use pipeliner::Pipeline;

pub fn pipeliner(dir_name: &str, threads: usize) {
    let dir_entries = std::fs::read_dir(format!("{}", dir_name));
    let mut all_images: Vec<raster::Image> = Vec::new();

    for entry in dir_entries.unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().is_none() {
            continue;
        }
        all_images.push(raster::open(path.to_str().unwrap()).unwrap());
    }
    
    let start = SystemTime::now();

    let _collection: Vec<raster::Image> = all_images
	    .with_threads(threads)
	    .out_buffer(512)
	    .map(move |mut image: Image| { 
	        filter::saturation(&mut image, 0.2).unwrap();
	        image
        })
        .with_threads(threads)
	    .out_buffer(512)
	    .map(move |mut image: Image| { 
	        filter::emboss(&mut image).unwrap();
	        image
        })
        .with_threads(threads)
	    .out_buffer(512)
	    .map(move |mut image: Image| { 
	        filter::gamma(&mut image, 2.0).unwrap();
	        image
        })
        .with_threads(threads)
	    .out_buffer(512)
	    .map(move |mut image: Image| { 
	        filter::sharpen(&mut image).unwrap();
	        image
        })
        .with_threads(threads)
	    .out_buffer(512)
	    .map(move |mut image: Image| { 
	        filter::grayscale(&mut image).unwrap();
	        image
        })
        .into_iter().collect();


    let system_duration = start.elapsed().expect("Failed to get render time?");
    let in_sec = system_duration.as_secs() as f64 + system_duration.subsec_nanos() as f64 * 1e-9;
    println!("Execution time: {} sec", in_sec);
}