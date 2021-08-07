use std::mem;
use std::fs::File;
use std::io::prelude::*;
use std::time::{SystemTime};

use {bzip2_sys};
use rust_spp::*;

struct Tcontent {
	buffer_input: Vec<u8>,
	buffer_output: Vec<u8>,
	output_size: u32,
}

struct WriteOutput{
    buf_write : std::fs::File,
}
impl WriteOutput {
    fn new(compressed_file_name : &str) -> WriteOutput {
        WriteOutput { buf_write: File::create(compressed_file_name).unwrap() } 
    }
}
impl In<Tcontent> for WriteOutput {
    fn process(&mut self, content: Tcontent, _order: u64){
        self.buf_write.write(&content.buffer_output[0..content.output_size as usize]).unwrap();
    }
}


pub fn rust_ssp(threads: usize, file_action: &str, file_name: &str,) {

	let mut file = File::open(file_name).expect("No file found.");

	if file_action == "compress" {
		let compressed_file_name = file_name.to_owned() + &".bz2";
		let mut buf_write = File::create(compressed_file_name).unwrap();
		let mut buffer_input = vec![];
		let mut buffer_output = vec![];

		// read data to memory
		file.read_to_end(&mut buffer_input).unwrap();

		// initialization
        let block_size = 900000;
        let mut pos_init: usize;
        let mut pos_end = 0;
        let mut bytes_left = buffer_input.len();

		let start = SystemTime::now();

        let pipeline = pipeline![
    		parallel!(move |mut content: Tcontent| {
    			unsafe{
	    			// computation
			        let mut bz_buffer: bzip2_sys::bz_stream = mem::zeroed();
			        bzip2_sys::BZ2_bzCompressInit(&mut bz_buffer as *mut _, 9, 0, 30);

			        bz_buffer.next_in = content.buffer_input.as_ptr() as *mut _;
			        bz_buffer.avail_in = content.buffer_input.len() as _;
			        bz_buffer.next_out = content.buffer_output.as_mut_ptr() as *mut _;
			        bz_buffer.avail_out = content.buffer_output.len() as _;
			        
	            	bzip2_sys::BZ2_bzCompress(&mut bz_buffer as *mut _, bzip2_sys::BZ_FINISH as _);
	            	bzip2_sys::BZ2_bzCompressEnd(&mut bz_buffer as *mut _);
	            
	            	content.output_size = bz_buffer.total_out_lo32;
	            }

            	Some(content)
    		}, threads as i32),
	        collect_ordered!()
		];

        while bytes_left > 0 {
    		pos_init = pos_end;
        	pos_end += if bytes_left < block_size {
	        		buffer_input.len()-pos_end
	        	} else {
	        		block_size
	        	};
	        bytes_left -= pos_end-pos_init;
        	
        	let buffer_slice = &buffer_input[pos_init..pos_end];

        	pipeline.post( Tcontent {
                buffer_input: buffer_slice.to_vec().clone(),
                buffer_output: vec![0; (buffer_slice.len() as f64 *1.01) as usize+600],
                output_size: 0,
            }).unwrap();

	    }

    	let collection = pipeline.collect();
        
        let system_duration = start.elapsed().expect("Failed to get render time?");
		let in_sec = system_duration.as_secs() as f64 + system_duration.subsec_nanos() as f64 * 1e-9;
        println!("Execution time: {} sec", in_sec);

        // write stage
    	for content in collection {
	        buffer_output.extend(&content.buffer_output[0..content.output_size as usize]);
	    }

		// write compressed data to file
		buf_write.write_all(&buffer_output).unwrap();
		std::fs::remove_file(file_name).unwrap();
	}
	else if file_action == "decompress" {
		// creating the decompressed file
		let decompressed_file_name = &file_name.to_owned()[..file_name.len()-4];
		let mut buf_write = File::create(decompressed_file_name).unwrap();
		let mut buffer_input = vec![];
		let mut buffer_output = vec![];

		// read data to memory
		file.read_to_end(&mut buffer_input).unwrap();

		// initialization
		let block_size = 900000;
        let mut pos_init: usize;
        let mut pos_end = 0;
        let mut bytes_left = buffer_input.len();
        let mut queue_blocks: Vec<(usize, usize)> = Vec::new();

        while bytes_left > 0 {
    		pos_init = pos_end;
            pos_end +=  {
                    // find the ending position by identifing the header of the next stream block
                    let buffer_slice;
                    if buffer_input.len() > block_size+10000 {
                        if (pos_init+block_size+10000) > buffer_input.len() {
                            buffer_slice = &buffer_input[pos_init+10..];
                        }else{
                            buffer_slice = &buffer_input[pos_init+10..pos_init+block_size+10000];
                        }
                    }else{
                        buffer_slice = &buffer_input[pos_init+10..];
                    }

                    let ret = buffer_slice.windows(10).position(|window| window == b"BZh91AY&SY");
                    let pos = match ret {
                        Some(i) => i+10,
                        None => buffer_input.len()-pos_init,
                    };
                    pos
                };
            bytes_left -= pos_end-pos_init;
            queue_blocks.push((pos_init, pos_end));

	    }

		let start = SystemTime::now();
        
		let pipeline = pipeline![
    		parallel!(move |mut content: Tcontent| {
    			unsafe{
	            	// computation
			        let mut bz_buffer: bzip2_sys::bz_stream = mem::zeroed();
			        bzip2_sys::BZ2_bzDecompressInit(&mut bz_buffer as *mut _, 0, 0);

			        bz_buffer.next_in = content.buffer_input.as_ptr() as *mut _;
			        bz_buffer.avail_in = content.buffer_input.len() as _;
			        bz_buffer.next_out = content.buffer_output.as_mut_ptr() as *mut _;
			        bz_buffer.avail_out = content.buffer_output.len() as _;
			        
		        	bzip2_sys::BZ2_bzDecompress(&mut bz_buffer as *mut _);	
		        	bzip2_sys::BZ2_bzDecompressEnd(&mut bz_buffer as *mut _);

		        	content.output_size = bz_buffer.total_out_lo32;
	            }

            	Some(content)
    		}, threads as i32),
	        collect_ordered!()
		];

        // Stream region
        for block in queue_blocks{
        	let buffer_slice = &buffer_input[block.0..block.1];

        	pipeline.post( Tcontent {
                buffer_input: buffer_slice.to_vec().clone(),
                buffer_output: vec![0; block_size],
                output_size: 0,
            }).unwrap();
        }

    	let collection = pipeline.collect();

		let system_duration = start.elapsed().expect("Failed to get render time?");
		let in_sec = system_duration.as_secs() as f64 + system_duration.subsec_nanos() as f64 * 1e-9;
        println!("Execution time: {} sec", in_sec);

        // write stage
    	for content in collection {
	        buffer_output.extend(&content.buffer_output[0..content.output_size as usize]);
	    }

		// write decompressed data to file
		buf_write.write_all(&buffer_output).unwrap();
		std::fs::remove_file(file_name).unwrap();
	}
}

pub fn rust_ssp_io(threads: usize, file_action: &str, file_name: &str,) {

	let mut file = File::open(file_name).expect("No file found.");

	if file_action == "compress" {
		let compressed_file_name = file_name.to_owned() + &".bz2";
		

		// initialization
        let block_size = 900000;
        let mut pos_init: usize;
        let mut pos_end = 0;
        let mut bytes_left: usize = file.metadata().unwrap().len() as usize;

		let start = SystemTime::now();

        let mut pipeline = pipeline![
    		parallel!(move |mut content: Tcontent| {
    			unsafe{
	    			// computation
			        let mut bz_buffer: bzip2_sys::bz_stream = mem::zeroed();
			        bzip2_sys::BZ2_bzCompressInit(&mut bz_buffer as *mut _, 9, 0, 30);

			        bz_buffer.next_in = content.buffer_input.as_ptr() as *mut _;
			        bz_buffer.avail_in = content.buffer_input.len() as _;
			        bz_buffer.next_out = content.buffer_output.as_mut_ptr() as *mut _;
			        bz_buffer.avail_out = content.buffer_output.len() as _;
			        
	            	bzip2_sys::BZ2_bzCompress(&mut bz_buffer as *mut _, bzip2_sys::BZ_FINISH as _);
	            	bzip2_sys::BZ2_bzCompressEnd(&mut bz_buffer as *mut _);
	            
	            	content.output_size = bz_buffer.total_out_lo32;
	            }

            	Some(content)
    		}, threads as i32),
	        sequential_ordered!(WriteOutput::new(&compressed_file_name))
		];

        while bytes_left > 0 {
    		pos_init = pos_end;
        	pos_end += if bytes_left < block_size {
	        		file.metadata().unwrap().len() as usize-pos_end
	        	} else {
	        		block_size
	        	};
	        bytes_left -= pos_end-pos_init;
        	
        	let mut buffer_slice: Vec<u8> = vec![0; pos_end-pos_init];
        	file.read(&mut buffer_slice).unwrap();

        	pipeline.post( Tcontent {
                buffer_input: buffer_slice.to_vec().clone(),
                buffer_output: vec![0; (buffer_slice.len() as f64 *1.01) as usize+600],
                output_size: 0,
            }).unwrap();

	    }
        
	    pipeline.end_and_wait();

        let system_duration = start.elapsed().expect("Failed to get render time?");
		let in_sec = system_duration.as_secs() as f64 + system_duration.subsec_nanos() as f64 * 1e-9;
        println!("Execution time: {} sec", in_sec);

		std::fs::remove_file(file_name).unwrap();
	}
	else if file_action == "decompress" {
		// creating the decompressed file
		let decompressed_file_name = (&file_name.to_owned()[..file_name.len()-4]).to_owned();
		let mut buffer_input = vec![];

		// read data to memory
		file.read_to_end(&mut buffer_input).unwrap();

		// initialization
		let block_size = 900000;
        let mut pos_init: usize;
        let mut pos_end = 0;
        let mut bytes_left = buffer_input.len();
        let mut queue_blocks: Vec<(usize, usize)> = Vec::new();

        while bytes_left > 0 {
    		pos_init = pos_end;
            pos_end +=  {
                    // find the ending position by identifing the header of the next stream block
                    let buffer_slice;
                    if buffer_input.len() > block_size+10000 {
                        if (pos_init+block_size+10000) > buffer_input.len() {
                            buffer_slice = &buffer_input[pos_init+10..];
                        }else{
                            buffer_slice = &buffer_input[pos_init+10..pos_init+block_size+10000];
                        }
                    }else{
                        buffer_slice = &buffer_input[pos_init+10..];
                    }

                    let ret = buffer_slice.windows(10).position(|window| window == b"BZh91AY&SY");
                    let pos = match ret {
                        Some(i) => i+10,
                        None => buffer_input.len()-pos_init,
                    };
                    pos
                };
            bytes_left -= pos_end-pos_init;
            queue_blocks.push((pos_init, pos_end));

	    }

		let start = SystemTime::now();
        
		let mut pipeline = pipeline![
    		parallel!(move |mut content: Tcontent| {
    			unsafe{
	            	// computation
			        let mut bz_buffer: bzip2_sys::bz_stream = mem::zeroed();
			        bzip2_sys::BZ2_bzDecompressInit(&mut bz_buffer as *mut _, 0, 0);

			        bz_buffer.next_in = content.buffer_input.as_ptr() as *mut _;
			        bz_buffer.avail_in = content.buffer_input.len() as _;
			        bz_buffer.next_out = content.buffer_output.as_mut_ptr() as *mut _;
			        bz_buffer.avail_out = content.buffer_output.len() as _;
			        
		        	bzip2_sys::BZ2_bzDecompress(&mut bz_buffer as *mut _);	
		        	bzip2_sys::BZ2_bzDecompressEnd(&mut bz_buffer as *mut _);

		        	content.output_size = bz_buffer.total_out_lo32;
	            }

            	Some(content)
    		}, threads as i32),
	        sequential_ordered!(WriteOutput::new(&decompressed_file_name))
		];

        // Stream region
        for block in queue_blocks{
        	let buffer_slice = &buffer_input[block.0..block.1];

        	pipeline.post( Tcontent {
                buffer_input: buffer_slice.to_vec().clone(),
                buffer_output: vec![0; block_size],
                output_size: 0,
            }).unwrap();
        }

        pipeline.end_and_wait();

		let system_duration = start.elapsed().expect("Failed to get render time?");
		let in_sec = system_duration.as_secs() as f64 + system_duration.subsec_nanos() as f64 * 1e-9;
        println!("Execution time: {} sec", in_sec);

		std::fs::remove_file(file_name).unwrap();
	}
}