use std::mem;
use std::fs::File;
use std::io::prelude::*;
use std::time::{SystemTime};

use {bzip2_sys};
use rayon::prelude::*;

struct TcontentIter {
	order: usize,
	buffer_input: Vec<u8>,
	buffer_output: Vec<u8>,
	output_size: u32,
}

struct EmitterCompress {
    buffer_input: Vec<u8>,
    block_size: usize,
    pos_init: usize,
    pos_end: usize,
    bytes_left: usize,
    order: usize,
}

impl EmitterCompress {
    fn new(buffer_input: Vec<u8>) -> EmitterCompress {
        let bytes_left = buffer_input.len();
        EmitterCompress { buffer_input,
                block_size: 900000,
                pos_init: 0,
			    pos_end: 0,
			    bytes_left,
			    order: 0,
			}
    }
}

impl Iterator for EmitterCompress {
    type Item = TcontentIter;

    fn next(&mut self) -> Option<Self::Item> {
    	if self.bytes_left <= 0 {
    	 	return None;
    	}

        self.pos_init = self.pos_end;
    	self.pos_end += if self.bytes_left < self.block_size {
        		self.buffer_input.len()-self.pos_end
        	} else {
        		self.block_size
        	};
        self.bytes_left -= self.pos_end-self.pos_init;
    	
    	let buffer_slice = &self.buffer_input[self.pos_init..self.pos_end];

		let content = TcontentIter{
			order: self.order,
    		buffer_input: buffer_slice.to_vec().clone(),
            buffer_output: vec![0; (buffer_slice.len() as f64 *1.01) as usize+600],
            output_size: 0,
        };

  		self.order += 1;
  		Some(content)
    }
}

struct EmitterDecompress {
    buffer_input: Vec<u8>,
    block_size: usize,
    order: usize,
    queue_blocks: Vec<(usize, usize)>,
}

impl EmitterDecompress {
    fn new(buffer_input: Vec<u8>, queue_blocks: Vec<(usize, usize)>) -> EmitterDecompress {
        EmitterDecompress { buffer_input,
                block_size: 900000,
			    order: 0,
			    queue_blocks,
			}
    }
}

impl Iterator for EmitterDecompress {
    type Item = TcontentIter;

    fn next(&mut self) -> Option<Self::Item> {
    	if self.order >= self.queue_blocks.len() {
    	 	return None;
    	}
    	let block = self.queue_blocks[self.order];
		// Stream region
    	let buffer_slice = &self.buffer_input[block.0..block.1];

    	let content =  TcontentIter{
    			order: self.order,
                buffer_input: buffer_slice.to_vec().clone(),
                buffer_output: vec![0; self.block_size],
                output_size: 0,
            };

  		self.order += 1;
  		Some(content)
    }
}

pub fn rayon(threads: usize, file_action: &str, file_name: &str,) {

	let mut file = File::open(file_name).expect("No file found.");

	if file_action == "compress" {
		let compressed_file_name = file_name.to_owned() + &".bz2";
		let mut buf_write = File::create(compressed_file_name).unwrap();
		let mut buffer_input = vec![];
		let mut buffer_output = vec![];

		// read data to memory
		file.read_to_end(&mut buffer_input).unwrap();

		let start = SystemTime::now();

		rayon::ThreadPoolBuilder::new().num_threads(threads).build_global().unwrap();

	    let mut collection: Vec<TcontentIter>  = EmitterCompress::new(buffer_input)
	        .par_bridge()
	        .filter_map(|mut content: TcontentIter| { 
	    		// computation
	            unsafe{
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
	        })
	        .collect();

        
        collection.sort_by_key(|content| content.order);
        
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
        
		rayon::ThreadPoolBuilder::new().num_threads(threads).build_global().unwrap();

	    let mut collection: Vec<TcontentIter>  = EmitterDecompress::new(buffer_input, queue_blocks)
	        .par_bridge()
	        .filter_map(|mut content: TcontentIter| { 
	    		// computation
	            unsafe{
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
	        })
	        .collect();

        
        collection.sort_by_key(|content| content.order);

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