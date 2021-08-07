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

use std::thread::Thread;
use std::time::Duration;
use std::sync::{Arc, RwLock};

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

#[derive(Clone)]
struct ThreadsState{
    t_handle: Thread,
    is_parked: bool
}

pub struct BetterCrossbeam {
    blocked_by_empty: RwLock<Vec<ThreadsState>>,
    blocked_by_full: RwLock<Vec<ThreadsState>>,
}

impl BetterCrossbeam {
    pub fn new(threads: usize) -> Arc<BetterCrossbeam> {
        Arc::new(BetterCrossbeam {
            blocked_by_empty: RwLock::new(vec![ThreadsState{t_handle: thread::current(), is_parked:false};threads]),
            blocked_by_full: RwLock::new(vec![ThreadsState{t_handle: thread::current(), is_parked:false};threads]),
        })
    }
    //Sender threads may only be blocked by full state
    pub fn set_send_handle(&self, t_id: usize, thread: Thread){
        let mut t_handle = self.blocked_by_full.write().unwrap();
        t_handle.insert(t_id, ThreadsState{
                        t_handle: thread,
                        is_parked: false
                    });
    }
    //Receiver threads may only be blocked by empty state
    pub fn set_rcv_handle(&self, t_id: usize, thread: Thread){
        let mut t_handle = self.blocked_by_empty.write().unwrap();
        t_handle.insert(t_id, ThreadsState{
                        t_handle: thread,
                        is_parked: false
                    });
    }
    pub fn send<T>(&self, sender: &crossbeam_channel::Sender<T>, t_id: usize, send_data: T) {
        if let Err(err) = sender.try_send(send_data) {
            if err.is_disconnected() {
                panic!("Error: Tried to insert in a disconnected channel!");
            } else if err.is_full(){

                let mut t_handle = self.blocked_by_full.write().unwrap();
                t_handle[t_id].is_parked = true;
                drop(t_handle);

                thread::park_timeout(Duration::from_millis(100));

                let mut t_handle = self.blocked_by_full.write().unwrap();
                t_handle[t_id].is_parked = false;
                drop(t_handle);

                self.send(sender,t_id,err.into_inner());
            } else {
                panic!("Error: {:?}!", err);
            }
        }

        let t_handle = self.blocked_by_empty.read().unwrap();
        if !t_handle.is_empty() {
            // Unpark the first parked thread. This could lead to unbalanced thread distribution
            for thread in &*t_handle {
                if thread.is_parked {
                    thread.t_handle.unpark();
                    break;
                }
            }
        }

    }
    pub fn recv<T>(&self, receiver: &crossbeam_channel::Receiver<T>, t_id: usize) -> Option<T> {
        loop {
            match receiver.try_recv() {
                Ok(content) => {
                    let t_handle = self.blocked_by_full.read().unwrap();
                    if !t_handle.is_empty() {
                        // Unpark the first parked thread. This could lead to unbalanced thread distribution
                        for thread in &*t_handle {
                            if thread.is_parked {
                                thread.t_handle.unpark();
                                break;
                            }
                        }
                    }
                    drop(t_handle);
                    return Some(content);
                }

                Err(e) if e == TryRecvError::Empty => { 
                    let mut t_handle = self.blocked_by_empty.write().unwrap();
                    t_handle[t_id].is_parked = true;
                    drop(t_handle);

                    thread::park_timeout(Duration::from_millis(100));

                    let mut t_handle = self.blocked_by_empty.write().unwrap();
                    t_handle[t_id].is_parked = false;
                    drop(t_handle);

                    continue;
                },

                Err(e) if e == TryRecvError::Disconnected => return None,
                Err(e) => panic!("Error during recv {}", e),
            }
        }
    }
}

pub fn better_eye_tracker(input_video: &String, nthreads: i32) -> opencv::Result<()> {

    let nthreads = nthreads as usize;

    let queue1: Arc<BetterCrossbeam> = BetterCrossbeam::new(nthreads);
    let queue2: Arc<BetterCrossbeam> = BetterCrossbeam::new(nthreads);
    let queue3: Arc<BetterCrossbeam> = BetterCrossbeam::new(nthreads);

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

    let generator_send = queue1.clone();
    let stage1_thread = thread::spawn(move || {
    	let mut order_id : u64 = 0;
        generator_send.set_send_handle(0,thread::current());
        loop {
        	let mut frame = Mat::default().unwrap();
	        video_in.read(&mut frame).unwrap();
	        if frame.size().unwrap().width == 0 {
	            break;
		    }
            generator_send.send(&queue1_send,0,StreamData {
                order: order_id ,
                frame: frame,
                equalized: None,
                faces: None, 
            });
            order_id += 1;
        }
        drop(queue1_send);
    });

    let mut stage2_threads = Vec::new();
    for i in 0..nthreads {        
        let in_queue = queue1.clone();
        let out_queue = queue2.clone();
        let local_send = queue2_send.clone();
        let local_recv = queue1_recv.clone();

        let local_thread = thread::spawn(move || {
            in_queue.set_rcv_handle(i,thread::current());
            out_queue.set_send_handle(i,thread::current());
            loop{
                let mut content = match in_queue.recv(&local_recv,i) {
                    Some(content) => content,
                    None => {break},
                };
				let face_xml = core::find_file("config/haarcascade_frontalface_alt.xml", true, false).unwrap();
                let mut face_detector = objdetect::CascadeClassifier::new(&face_xml).unwrap();

                let equalized = common::prepare_frame(&content.frame).unwrap();

                // Detect faces
                let faces = common::detect_faces(&equalized,&mut face_detector).unwrap();
               
                // Out data
                content.equalized = Some(equalized);
                content.faces = Some(faces);
                out_queue.send(&local_send,i,content);
            }
            drop(local_send);
        });
        stage2_threads.push(local_thread);
    } 
    drop(queue2_send);

    let mut stage3_threads = Vec::new();
    for i in 0..nthreads {
        let in_queue = queue2.clone();
        let out_queue = queue3.clone();
        let local_send = queue3_send.clone();
        let local_recv = queue2_recv.clone();    

        let local_thread = thread::spawn(move || {
            in_queue.set_rcv_handle(i,thread::current());
            out_queue.set_send_handle(i,thread::current());
            loop{
                let mut content = match in_queue.recv(&local_recv,i) {
                    Some(content) => content,
                    None => {break},
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

                out_queue.send(&local_send,i,content);
            }
            drop(local_send);
        });
        stage3_threads.push(local_thread);
    }
    drop(queue3_send);

    let in_queue = queue3.clone();
    let local_recv = queue3_recv.clone();
    // Sequential reordering thread
    let stage4_thread = thread::spawn(move || {
        in_queue.set_rcv_handle(0,thread::current());
        let mut reorder_engine = Reorder::new();
        let mut expected_ordered: u64 = 0;
        loop {
            let mut content = match in_queue.recv(&local_recv,0) {
                Some(content) => content,
                None => {break},
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
