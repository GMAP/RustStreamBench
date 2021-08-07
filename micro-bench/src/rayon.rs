use std::time::SystemTime;
use std::io::Write;
use std::fs::File;

use rayon::prelude::*;

struct TcontentIter {
    size: usize,
    line: i64,
    line_buffer: Vec<u8>,
    a_buffer: Vec<f64>,
    b_buffer: Vec<f64>,
    k_buffer: Vec<i32>,
}

impl TcontentIter {
    fn new(size: usize) -> TcontentIter {
        TcontentIter { size,
                line: -1,
                line_buffer: vec![0; size],
                a_buffer: vec![0.0; size],
                b_buffer: vec![0.0; size],
                k_buffer: vec![0; size], }
    }
}

impl Iterator for TcontentIter {
    type Item = TcontentIter;

    fn next(&mut self) -> Option<Self::Item> {
        self.line += 1;

        if (self.line as usize) < self.size {
            Some(TcontentIter { size: self.size,
                line: self.line,
                line_buffer: self.line_buffer.clone(),
                a_buffer: self.a_buffer.clone(),
                b_buffer: self.b_buffer.clone(),
                k_buffer: self.k_buffer.clone(), })
        } else {
            None
        }
    }
}

pub fn rayon_pipeline(size: usize, threads: usize, iter_size1: i32, iter_size2: i32) {

    let start = SystemTime::now();

    rayon::ThreadPoolBuilder::new().num_threads(2*threads).build_global().unwrap();

    let mut collection: Vec<TcontentIter>  = TcontentIter::new(size)
        .par_bridge()
        .filter_map(|mut content: TcontentIter| { 
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
        })
        .filter_map(|mut content: TcontentIter| { 
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
        })
        .collect();

        collection.sort_by_key(|content| content.line);

        let system_duration = start.elapsed().expect("Failed to get render time?");
        let in_sec = system_duration.as_secs() as f64 + system_duration.subsec_nanos() as f64 * 1e-9;
        println!("Execution time Rayon: {} sec", in_sec);

        let mut m = vec![];

        for line in collection {
            m.extend(line.line_buffer);
        }

        let mut buffer = File::create("result_rayon.txt").unwrap();
        buffer.write_all(&m).unwrap();
}
