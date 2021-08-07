use raster::filter;
use raster::Image;
use std::time::{SystemTime};

use{
    futures::future::lazy,
    futures::sync::*,
    futures::{stream, Future, Stream},
};

macro_rules! spawn_return {
    ($block:expr) => {{
        let (sender, receiver) = oneshot::channel::<_>();
        tokio::spawn(lazy(move || {
            let result = $block;
            sender.send(result).ok();
            Ok(())
        }));
        receiver
    }};
}

pub fn tokio(dir_name: &str, threads: usize) {
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

    let processing_pipeline = stream::iter_ok(all_images)
        .map(move |mut image: Image| {
            spawn_return!({
                filter::saturation(&mut image, 0.2).unwrap();
                image
            })
        })
        .buffer_unordered(threads)
        .map(move |mut image: Image| {
            spawn_return!({
                filter::emboss(&mut image).unwrap();
                image
            })
        })
        .buffer_unordered(threads)
        .map(move |mut image: Image| {
            spawn_return!({
                filter::gamma(&mut image, 2.0).unwrap();
                image
            })
        })
        .buffer_unordered(threads)
        .map(move |mut image: Image| {
            spawn_return!({
                filter::sharpen(&mut image).unwrap();
                image
            })
        })
        .buffer_unordered(threads)
        .map(move |mut image: Image| {
            spawn_return!({
                filter::grayscale(&mut image).unwrap();
                image
            })
        })
        .buffer_unordered(threads)
        .for_each(|_| Ok(()))
        .map_err(|e| println!("listener error = {:?}", e));

    tokio::run(processing_pipeline);

    let system_duration = start.elapsed().expect("Failed to get render time?");
    let in_sec = system_duration.as_secs() as f64 + system_duration.subsec_nanos() as f64 * 1e-9;
    println!("Execution time: {} sec", in_sec);
}