use std::time::SystemTime;
use std::io::Write;
use std::fs::File;

use{
    futures::future::lazy,
    futures::sync::*,
    futures::{stream, Future, Stream},
    tokio::prelude::*,
};

struct Tcontent {
    size: usize,
    line: i64,
    line_buffer: Vec<u8>,
    a_buffer: Vec<f64>,
    b_buffer: Vec<f64>,
    k_buffer: Vec<i32>,
}

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

pub fn tokio_pipeline(size: usize, threads: usize, iter_size1: i32, iter_size2: i32) {
    let mut i = 0;
    let mut jumper = true;
    let mut counter = 0;

    let mut m = vec![];

    let start = SystemTime::now();

    let processing_stream = stream::poll_fn(move || -> Poll<Option<Tcontent>,futures::sync::oneshot::Canceled> {
        if !jumper {
            i += 1;
        }else{
            jumper = false; 
        }
        if i >= size {
            return Ok(Async::Ready(None));
        }
        Ok(Async::Ready(Some(Tcontent {
                    size,
                    line: i as i64,
                    line_buffer: vec![0; size],
                    a_buffer: vec![0.0; size],
                    b_buffer: vec![0.0; size],
                    k_buffer: vec![0; size],
                })))
    });


    let collection = processing_stream
        .map(move |mut content: Tcontent| {
            spawn_return!({
                let init_a = -2.125 as f64;
                let init_b = -1.5 as f64;
                let range = 3.0 as f64;
                let step = range / (content.size as f64);

                let im = init_b + (step * (content.line as f64));

                for j in 0 .. content.size {

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

                content
            })
        }).buffered(threads)
        .map(move |mut content: Tcontent| {
                    spawn_return!({
                        let init_a = -2.125 as f64; 
                        let init_b = -1.5 as f64;
                        let range = 3.0 as f64;
                        let step = range / (content.size as f64);

                        let im = init_b + (step * (content.line as f64));

                        for j in 0 .. content.size {
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
                        content
                    })
        }).buffered(threads)
        .for_each(move |content: Tcontent| {
            m.extend(content.line_buffer);
            counter += 1;
            if counter == size {
                let system_duration = start.elapsed().expect("Failed to get render time?");
                let in_sec = system_duration.as_secs() as f64 + system_duration.subsec_nanos() as f64 * 1e-9;
                println!("Execution time Tokio: {} sec", in_sec);

                let mut buffer = File::create("result_tokio.txt").unwrap();
                buffer.write_all(&m).unwrap();
            }
            Ok(())
        })
        .map_err(|e| println!("Error = {:?}", e));

        tokio::run(collection);
}