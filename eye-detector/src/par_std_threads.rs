use {
    opencv::{
        prelude::*,
        core,
        objdetect,
        videoio,
        types,
    },
    std::thread,
    crossbeam_channel::{bounded, TryRecvError},
    std::collections::BTreeMap,
};

#[path = "common.rs"]
mod common;

struct StreamData {
	order: u64,
    frame: Mat,
    equalized: Option<Mat>,
    faces: Option<types::VectorOfRect>,
}
unsafe impl Sync for StreamData {}
unsafe impl Send for StreamData {}

pub struct Reorder {
    storage: BTreeMap<u64, StreamData>,
}

impl Reorder {
    fn new() -> Reorder {
        Reorder {
            storage: BTreeMap::<u64, StreamData>::new(),
        }
    }

    fn enqueue(&mut self, item: StreamData) {
        self.storage.insert(item.order, item);
    }

    fn remove(&mut self, order: u64) -> Option<StreamData> {
        if self.storage.contains_key(&order){
        	let removed_item = self.storage.remove(&order);
	        match removed_item {
	            Some(value) => return Some(value),
	            None => { panic!("Ordered removal failed") }
	        }
    	} else {
    		return None;
    	}
    }
}

pub fn std_threads_eye_tracker(input_video: &String, nthreads: i32) -> opencv::Result<()> {

	let (queue1_send, queue1_recv) = bounded(512);
    let (queue2_send, queue2_recv) = bounded(512);
    let (queue3_send, queue3_recv) = bounded(512);

	let mut video_in = videoio::VideoCapture::from_file(input_video, videoio::CAP_FFMPEG).unwrap();
    let in_opened = videoio::VideoCapture::is_opened(&video_in).unwrap();
    if !in_opened {
        panic!("Unable to open input video {:?}!", input_video);
    }
    let frame_size = core::Size::new(video_in.get(videoio::VideoCaptureProperties::CAP_PROP_FRAME_WIDTH as i32).unwrap() as i32,
                video_in.get(videoio::VideoCaptureProperties::CAP_PROP_FRAME_HEIGHT as i32).unwrap() as i32,);
    let fourcc = videoio::VideoWriter::fourcc(
        'm' as i8,
        'p' as i8,
        'g' as i8,
        '1' as i8
    ).unwrap();
    let fps_out = video_in.get(videoio::VideoCaptureProperties::CAP_PROP_FPS as i32).unwrap();
    let mut video_out : videoio::VideoWriter = videoio::VideoWriter::new(
        "output.avi",
        fourcc,
        fps_out,
        frame_size,
        true
    ).unwrap();
    let out_opened = videoio::VideoWriter::is_opened(&video_out).unwrap();
    if !out_opened {
        panic!("Unable to open output video output.avi!");
    }

    let stage1_thread = thread::spawn(move || {
    	let mut order_id : u64 = 0;
        loop {
        	let mut frame = Mat::default().unwrap();
	        video_in.read(&mut frame).unwrap();
	        if frame.size().unwrap().width == 0 {
	            break;
		    }
            queue1_send.send(StreamData {
                order: order_id ,
                frame: frame,
                equalized: None,
                faces: None, 
            }).unwrap();
            order_id += 1;
        }
        drop(queue1_send);
    });

    let mut stage2_threads = Vec::new();
    for _i in 0..nthreads {
        let (send, recv) = (queue2_send.clone(), queue1_recv.clone());
        
        let local_thread = thread::spawn(move || {
            loop{
                let content = recv.try_recv();
                let mut content = match content {
                    Ok(content) => content,
                    Err(e) if e == TryRecvError::Disconnected => break,
                    Err(e) if e == TryRecvError::Empty => continue,
                    Err(e) => panic!("Error during recv {}", e),
                };
				let face_xml = core::find_file("config/haarcascade_frontalface_alt.xml", true, false).unwrap();
                let mut face_detector = objdetect::CascadeClassifier::new(&face_xml).unwrap();

                let equalized = common::prepare_frame(&content.frame).unwrap();

                // Detect faces
                let faces = common::detect_faces(&equalized,&mut face_detector).unwrap();
               
                // Out data
                content.equalized = Some(equalized);
                content.faces = Some(faces);
                send.send(content).unwrap();

            }
        });
        stage2_threads.push(local_thread);
    } 
    drop(queue2_send);

    let mut stage3_threads = Vec::new();
    for _i in 0..nthreads {
        let (send, recv) = (queue3_send.clone(), queue2_recv.clone());
        
        let local_thread = thread::spawn(move || {
            loop{
                let content = recv.try_recv();
                let mut content = match content {
                    Ok(content) => content,
                    Err(e) if e == TryRecvError::Disconnected => break,
                    Err(e) if e == TryRecvError::Empty => continue,
                    Err(e) => panic!("Error during recv {}", e),
                };
                let equalized = match content.equalized {
				    Some(ref x) => x,
				    None    => panic!("Empty value inside stream!"),
				};
                let faces = match content.faces {
				    Some(ref x) => x,
				    None    => panic!("Empty value inside stream!"),
				};
                let eye_xml = core::find_file("config/haarcascade_eye.xml", true, false).unwrap();
                let mut eye_detector = objdetect::CascadeClassifier::new(&eye_xml).unwrap();

                for face in faces {

                    let eyes =  common::detect_eyes(&core::Mat::roi(&equalized,face).unwrap(),
                                                    &mut eye_detector).unwrap();

                    common::draw_in_frame(&mut content.frame,&eyes,&face).unwrap();

                }

                send.send(content).unwrap();
            }
        });
        stage3_threads.push(local_thread);
    }
    drop(queue3_send);

    let recv = queue3_recv.clone();
    // Sequential reordering thread
    let stage4_thread = thread::spawn(move || {
        let mut reorder_engine = Reorder::new();
        let mut expected_ordered: u64 = 0;
        loop {
            let content = recv.try_recv();
            let mut content = match content {
                Ok(content) => content,
                Err(e) if e == TryRecvError::Disconnected => break,
                Err(e) if e == TryRecvError::Empty => continue,
                Err(e) => panic!("Error during recv {}", e),
            };
            loop {
                if content.order != expected_ordered {
                    reorder_engine.enqueue(content);
                    break;
                }
                
                // Write
                video_out.write(&mut content.frame).unwrap();

                expected_ordered += 1;
                let removed_item = reorder_engine.remove(expected_ordered);
                match removed_item {
                    Some(value) => { content = value; continue; },
                    None => break,
                }
            }
        }
    });

    stage1_thread.join().unwrap();
    for thread in stage2_threads {
        thread.join().unwrap();  
    }
    for thread in stage3_threads {
        thread.join().unwrap();  
    }
    stage4_thread.join().unwrap();

	Ok(())
}
