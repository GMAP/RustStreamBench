use std::time::SystemTime;
use std::io::Write;
use std::fs::File;

use rust_spp::*;

struct Tcontent {
    size: usize,
    line: i64,
    line_buffer: Vec<u8>,
    a_buffer: Vec<f64>,
    b_buffer: Vec<f64>,
    k_buffer: Vec<i32>,
}

pub fn rust_ssp_pipeline(size: usize, threads: usize, iter_size1: i32, iter_size2: i32){

    let start = SystemTime::now();

    let pipeline = pipeline![
        parallel!(move |mut content: Tcontent| {
            let init_a = -2.125 as f64;
            let init_b = -1.5 as f64;
            let range = 3.0 as f64;
            let step = range / (size as f64);

            let im = init_b + (step * (content.line as f64));

            for j in 0 .. size {

                let mut a = init_a + step * j as f64;
                let cr = a;

                let mut b = im;
                let mut k = 0;

                for ii in 0.. iter_size1 {
                    let a2 = a * a;
                    let b2 = b * b;
                    if (a2 + b2) > 4.0 {break;}
                    b = 2.0 * a * b + im;
                    a = a2 - b2 + cr;
                    k = ii;
                }
                content.a_buffer[j] = a;
                content.b_buffer[j] = b;
                content.k_buffer[j] = k;

            }
            Some(content)

        }, threads as i32),
        parallel!(move |mut content: Tcontent| {   
            let init_a = -2.125 as f64; 
            let init_b = -1.5 as f64;
            let range = 3.0 as f64;
            let step = range / (size as f64);

            let im = init_b + (step * (content.line as f64));

            for j in 0 .. size {
                let cr = init_a + step * j as f64;
                if content.k_buffer[j] == iter_size1-1 {

                    for ii in iter_size1 .. iter_size1+iter_size2 {
                        let a2 = content.a_buffer[j] * content.a_buffer[j];
                        let b2 = content.b_buffer[j] * content.b_buffer[j];
                        if (a2 + b2) > 4.0 {break;}
                        content.b_buffer[j] = 2.0 * content.a_buffer[j] * content.b_buffer[j] + im;
                        content.a_buffer[j] = a2 - b2 + cr;
                        content.k_buffer[j] = ii;
                    }
                }
                content.line_buffer[j] = (255 as f64 - (((content.k_buffer[j] as f64) * 255 as f64 / ((iter_size1+iter_size2) as f64)))) as u8;
            }
            Some(content)

        }, threads as i32),
        collect_ordered!()];
    
    for i in 0 .. size {
        pipeline.post( Tcontent {
                size,
                line: i as i64,
                line_buffer: vec![0; size],
                a_buffer: vec![0.0; size],
                b_buffer: vec![0.0; size],
                k_buffer: vec![0; size],
            }).unwrap();
    }
    let collection = pipeline.collect();

    let system_duration = start.elapsed().expect("Failed to get render time?");
    let in_sec = system_duration.as_secs() as f64 + system_duration.subsec_nanos() as f64 * 1e-9;
    println!("Execution time Rust-SSP: {} sec", in_sec);

    let mut m = vec![];

    for line in collection {
        m.extend(line.line_buffer);
    }

    let mut buffer = File::create("result_rust-ssp.txt").unwrap();
    buffer.write_all(&m).unwrap();
}