use {
    opencv::{
        prelude::*,
        core,
        objdetect,
        videoio,
        types,
    },
    futures::future::lazy,
	futures::sync::*,
	futures::{stream, Future, Stream},
	tokio::prelude::*,
};
#[path = "common.rs"]
mod common;

struct MatData {
    frame: Mat,
}
unsafe impl Sync for MatData {}
unsafe impl Send for MatData {}

struct EyesData {
    frame: Mat,
    equalized: Mat,
    faces: types::VectorOfRect,
}
unsafe impl Sync for EyesData {}
unsafe impl Send for EyesData {}

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

pub fn tokio_eye_tracker(input_video: &String, nthreads: i32) -> opencv::Result<()> {
    let mut video_in = videoio::VideoCapture::from_file(input_video, videoio::CAP_FFMPEG)?;
    let in_opened = videoio::VideoCapture::is_opened(&video_in)?;
    if !in_opened {
        panic!("Unable to open input video {:?}!", input_video);
    }
    let frame_size = core::Size::new(video_in.get(videoio::VideoCaptureProperties::CAP_PROP_FRAME_WIDTH as i32)? as i32,
                video_in.get(videoio::VideoCaptureProperties::CAP_PROP_FRAME_HEIGHT as i32)? as i32,);
    let fourcc = videoio::VideoWriter::fourcc(
        'm' as i8,
        'p' as i8,
        'g' as i8,
        '1' as i8
    )?;
    let fps_out = video_in.get(videoio::VideoCaptureProperties::CAP_PROP_FPS as i32)?;
    let mut video_out : videoio::VideoWriter = videoio::VideoWriter::new(
        "output.avi",
        fourcc,
        fps_out,
        frame_size,
        true
    )?;
    let out_opened = videoio::VideoWriter::is_opened(&video_out)?;
    if !out_opened {
        panic!("Unable to open output video output.avi!");
    }

	let processing_stream = stream::poll_fn(move || -> Poll<Option<MatData>,futures::sync::oneshot::Canceled> {
        // Read frame
        let mut frame = Mat::default().unwrap();
        video_in.read(&mut frame).unwrap();
        if frame.size().unwrap().width == 0 {
            return Ok(Async::Ready(None));
	    }
        Ok(Async::Ready(Some(MatData{frame : frame})))
    } );

    let threads = nthreads as usize;

    let pipeline = processing_stream
        .map(move |in_data: MatData| {
            spawn_return!({
                let face_xml = core::find_file("config/haarcascade_frontalface_alt.xml", true, false).unwrap();
                let mut face_detector = objdetect::CascadeClassifier::new(&face_xml).unwrap();

                let equalized = common::prepare_frame(&in_data.frame).unwrap();

                // Detect faces
                let faces = common::detect_faces(&equalized,&mut face_detector).unwrap();
                // Out data
                EyesData{frame : in_data.frame, 
                        equalized : equalized, 
                        faces : faces}
            })
        })
        .buffered(threads)
        .map(move |mut in_data: EyesData| {
            spawn_return!({
                let eye_xml = core::find_file("config/haarcascade_eye.xml", true, false).unwrap();
                let mut eye_detector = objdetect::CascadeClassifier::new(&eye_xml).unwrap();

                for face in in_data.faces {

                    let eyes =  common::detect_eyes(&core::Mat::roi(&in_data.equalized,face).unwrap(),
                                                    &mut eye_detector).unwrap();

                    common::draw_in_frame(&mut in_data.frame,&eyes,&face).unwrap();

                }
                MatData{ frame : in_data.frame }
            })
        })
        .buffered(threads)
        .for_each(move |mut in_data: MatData| {
            video_out.write(&mut in_data.frame).unwrap();
            Ok(())
        })
        .map_err(|e| println!("Error = {:?}", e));

    tokio::run(pipeline);

	Ok(())
}