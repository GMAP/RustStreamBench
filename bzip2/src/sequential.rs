use std::mem;
use std::fs::File;
use std::io::prelude::*;
use std::time::{SystemTime};

use {bzip2_sys};

pub fn sequential(file_action: &str, file_name: &str,) {

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

        while bytes_left > 0 {
    		pos_init = pos_end;
        	pos_end += if bytes_left < block_size {
	        		buffer_input.len()-pos_end
	        	} else {
	        		block_size
	        	};
	        bytes_left -= pos_end-pos_init;
        	
        	let buffer_slice = &buffer_input[pos_init..pos_end];

	        // computation
        	unsafe{
		        let mut bz_buffer: bzip2_sys::bz_stream = mem::zeroed();
		        bzip2_sys::BZ2_bzCompressInit(&mut bz_buffer as *mut _, 9, 0, 30);

	        	let mut output: Vec<u8> = vec![0; (buffer_slice.len() as f64 *1.01) as usize+600];

		        bz_buffer.next_in = buffer_slice.as_ptr() as *mut _;
		        bz_buffer.avail_in = buffer_slice.len() as _;
		        bz_buffer.next_out = output.as_mut_ptr() as *mut _;
		        bz_buffer.avail_out = output.len() as _;
		        
            	bzip2_sys::BZ2_bzCompress(&mut bz_buffer as *mut _, bzip2_sys::BZ_FINISH as _);
            	bzip2_sys::BZ2_bzCompressEnd(&mut bz_buffer as *mut _);
		        
		        // write stage
		        buffer_output.extend(&output[0..bz_buffer.total_out_lo32 as usize]);
        	}
        }

        let system_duration = start.elapsed().expect("Failed to get render time?");
		let in_sec = system_duration.as_secs() as f64 + system_duration.subsec_nanos() as f64 * 1e-9;
        println!("Execution time: {} sec", in_sec);

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
        
        // Stream region
        for block in queue_blocks{
        	let buffer_slice = &buffer_input[block.0..block.1];

        	// computation
        	unsafe{
		        let mut bz_buffer: bzip2_sys::bz_stream = mem::zeroed();
		        bzip2_sys::BZ2_bzDecompressInit(&mut bz_buffer as *mut _, 0, 0);

	        	let mut output: Vec<u8> = vec![0; block_size];

		        bz_buffer.next_in = buffer_slice.as_ptr() as *mut _;
		        bz_buffer.avail_in = buffer_slice.len() as _;
		        bz_buffer.next_out = output.as_mut_ptr() as *mut _;
		        bz_buffer.avail_out = output.len() as _;
		        
	        	bzip2_sys::BZ2_bzDecompress(&mut bz_buffer as *mut _);	
            	bzip2_sys::BZ2_bzDecompressEnd(&mut bz_buffer as *mut _);
		        
		        // write stage
		        buffer_output.extend(&output[0..bz_buffer.total_out_lo32 as usize]);
	        }
	    }

		let system_duration = start.elapsed().expect("Failed to get render time?");
		let in_sec = system_duration.as_secs() as f64 + system_duration.subsec_nanos() as f64 * 1e-9;
        println!("Execution time: {} sec", in_sec);

		// write decompressed data to file
		buf_write.write_all(&buffer_output).unwrap();
		std::fs::remove_file(file_name).unwrap();
	}
}

pub fn sequential_io(file_action: &str, file_name: &str,) {

	let mut file = File::open(file_name).expect("No file found.");

	if file_action == "compress" {
		let compressed_file_name = file_name.to_owned() + &".bz2";
		let mut buf_write = File::create(compressed_file_name).unwrap();
		//let mut buffer_input = vec![];
		//let mut buffer_output = vec![];

		// read data to memory
		//file.read_to_end(&mut buffer_input).unwrap();

		// initialization
        let block_size = 900000;
        let mut pos_init: usize;
        let mut pos_end = 0;
        let mut bytes_left: usize = file.metadata().unwrap().len() as usize;

		let start = SystemTime::now();

        while bytes_left > 0 {
    		pos_init = pos_end;
        	pos_end += if bytes_left < block_size {
	        		file.metadata().unwrap().len() as usize-pos_end
	        	} else {
	        		block_size
	        	};
	        bytes_left -= pos_end-pos_init;
        	
        	//let buffer_slice = &buffer_input[pos_init..pos_end];
        	let mut buffer_slice: Vec<u8> = vec![0; pos_end-pos_init];
        	file.read(&mut buffer_slice).unwrap();

	        // computation
        	unsafe{
		        let mut bz_buffer: bzip2_sys::bz_stream = mem::zeroed();
		        bzip2_sys::BZ2_bzCompressInit(&mut bz_buffer as *mut _, 9, 0, 30);

	        	let mut output: Vec<u8> = vec![0; (buffer_slice.len() as f64 *1.01) as usize+600];

		        bz_buffer.next_in = buffer_slice.as_ptr() as *mut _;
		        bz_buffer.avail_in = buffer_slice.len() as _;
		        bz_buffer.next_out = output.as_mut_ptr() as *mut _;
		        bz_buffer.avail_out = output.len() as _;
		        
            	bzip2_sys::BZ2_bzCompress(&mut bz_buffer as *mut _, bzip2_sys::BZ_FINISH as _);
            	bzip2_sys::BZ2_bzCompressEnd(&mut bz_buffer as *mut _);
		        
		        // write stage
		        //buffer_output.extend(&output[0..bz_buffer.total_out_lo32 as usize]);
        		buf_write.write(&output[0..bz_buffer.total_out_lo32 as usize]).unwrap();
        	}
        }

        let system_duration = start.elapsed().expect("Failed to get render time?");
		let in_sec = system_duration.as_secs() as f64 + system_duration.subsec_nanos() as f64 * 1e-9;
        println!("Execution time: {} sec", in_sec);

		// write compressed data to file
		//buf_write.write_all(&buffer_output).unwrap();
		std::fs::remove_file(file_name).unwrap();
	}
	else if file_action == "decompress" {
		// creating the decompressed file
		let decompressed_file_name = &file_name.to_owned()[..file_name.len()-4];
		let mut buf_write = File::create(decompressed_file_name).unwrap();
		let mut buffer_input = vec![];
		//let mut buffer_output = vec![];

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
        
        // Stream region
        for block in queue_blocks{
        	let buffer_slice = &buffer_input[block.0..block.1];

        	// computation
        	unsafe{
		        let mut bz_buffer: bzip2_sys::bz_stream = mem::zeroed();
		        bzip2_sys::BZ2_bzDecompressInit(&mut bz_buffer as *mut _, 0, 0);

	        	let mut output: Vec<u8> = vec![0; block_size];

		        bz_buffer.next_in = buffer_slice.as_ptr() as *mut _;
		        bz_buffer.avail_in = buffer_slice.len() as _;
		        bz_buffer.next_out = output.as_mut_ptr() as *mut _;
		        bz_buffer.avail_out = output.len() as _;
		        
	        	bzip2_sys::BZ2_bzDecompress(&mut bz_buffer as *mut _);	
            	bzip2_sys::BZ2_bzDecompressEnd(&mut bz_buffer as *mut _);
		        
		        // write stage
		        //buffer_output.extend(&output[0..bz_buffer.total_out_lo32 as usize]);
        		buf_write.write(&output[0..bz_buffer.total_out_lo32 as usize]).unwrap();
	        }
	    }

		let system_duration = start.elapsed().expect("Failed to get render time?");
		let in_sec = system_duration.as_secs() as f64 + system_duration.subsec_nanos() as f64 * 1e-9;
        println!("Execution time: {} sec", in_sec);

		// write decompressed data to file
		//buf_write.write_all(&buffer_output).unwrap();
		std::fs::remove_file(file_name).unwrap();
	}
}