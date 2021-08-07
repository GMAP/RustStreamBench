use std::time::SystemTime;
use std::io::Write;
use std::fs::File;

pub fn sequential(size: usize, iter_size1: i32, iter_size2: i32) {

    let start = SystemTime::now();

    let init_a = -2.125 as f64;
    let init_b = -1.5 as f64;
    let range = 3.0 as f64;
    let step = range / (size as f64);

    let mut m = vec![0; size*size];
    let iter = iter_size1 + iter_size2;

    for i in 0 .. size{
        let im = init_b + (step * (i as f64));

        for j in 0 .. size {
            let mut a = init_a + step * j as f64;
            let cr = a;
            let mut b = im;
            let mut k = 0;

            for ii in 0 .. iter {
                let a2 = a * a;
                let b2 = b * b;
                if (a2 + b2) > 4.0 {break;}
                b = 2.0 * a * b + im;
                a = a2 - b2 + cr;
                k = ii;
            }
            
            m[i*size+j] = (255 as f64 - ((k as f64 * 255 as f64 / (iter as f64)))) as u8;
        
        }
    }

    let system_duration = start.elapsed().expect("Failed to get render time?");
    let in_sec = system_duration.as_secs() as f64 + system_duration.subsec_nanos() as f64 * 1e-9;
    println!("Execution time: {} sec", in_sec);

    let mut buffer = File::create("result_sequential.txt").unwrap();
    buffer.write_all(&m).unwrap();
}