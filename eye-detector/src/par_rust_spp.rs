use {
    opencv::{
        prelude::*,
        core,
        objdetect,
        videoio,
        types,
    },
    rust_spp::*,
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

struct DetectFaces{
    face_detector : objdetect::CascadeClassifier,
}
impl DetectFaces {
    fn new() -> DetectFaces {
        let face_xml = core::find_file("config/haarcascade_frontalface_alt.xml", true, false).unwrap();
        let face_detector = objdetect::CascadeClassifier::new(&face_xml).unwrap();
        DetectFaces { face_detector : face_detector } 
    }
}
impl InOut<MatData,EyesData> for DetectFaces {
    fn process(&mut self, in_data: MatData) -> Option<EyesData>{
        // Convert to gray and equalize frame
        let equalized = common::prepare_frame(&in_data.frame).unwrap();

        // Detect faces
        let faces = common::detect_faces(&equalized,&mut self.face_detector).unwrap();

        let out_data = EyesData{frame : in_data.frame, 
                                equalized : equalized, 
                                faces : faces};
        Some(out_data)
    }
}


struct DetectEyes{
    eye_detector : objdetect::CascadeClassifier,
}
impl DetectEyes {
    fn new() -> DetectEyes {
        let eye_xml = core::find_file("config/haarcascade_eye.xml", true, false).unwrap();
        let eye_detector = objdetect::CascadeClassifier::new(&eye_xml).unwrap();
        DetectEyes { eye_detector : eye_detector } 
    }
}
impl InOut<EyesData,MatData> for DetectEyes {
    fn process(&mut self, mut in_data : EyesData) -> Option<MatData>{
        for face in in_data.faces {

            let eyes =  common::detect_eyes(&core::Mat::roi(&in_data.equalized,face).unwrap(),
                                            &mut self.eye_detector).unwrap();

            common::draw_in_frame(&mut in_data.frame,&eyes,&face).unwrap();

        }
        let out_data = MatData{ frame : in_data.frame };
        Some(out_data)
    }
}

struct WriteOutput{
    video_out : videoio::VideoWriter,
}
impl WriteOutput {
    fn new(fps_out : f64, frame_size : core::Size) -> WriteOutput {
	    let fourcc = videoio::VideoWriter::fourcc(
	        'm' as i8,
	        'p' as i8,
	        'g' as i8,
	        '1' as i8
	    ).unwrap();
		let video_out = videoio::VideoWriter::new(
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

        WriteOutput { video_out : video_out } 
    }
}
impl In<MatData> for WriteOutput {
    fn process(&mut self, mut in_data: MatData, _order: u64){
        //Write output frame
        self.video_out.write(&mut in_data.frame).unwrap();
    }
}

pub fn rust_spp_eye_tracker(input_video: &String, nthreads: i32) -> opencv::Result<()> {
    let mut video_in = videoio::VideoCapture::from_file(input_video, videoio::CAP_FFMPEG)?;
    let in_opened = videoio::VideoCapture::is_opened(&video_in)?;
    if !in_opened {
        panic!("Unable to open input video {:?}!", input_video);
    }
    let frame_size = core::Size::new(video_in.get(videoio::VideoCaptureProperties::CAP_PROP_FRAME_WIDTH as i32)? as i32,
                video_in.get(videoio::VideoCaptureProperties::CAP_PROP_FRAME_HEIGHT as i32)? as i32,);
    let fps_out = video_in.get(videoio::VideoCaptureProperties::CAP_PROP_FPS as i32)?;

    let mut pipeline = pipeline![
        parallel!(DetectFaces::new(), nthreads),
        parallel!(DetectEyes::new(), nthreads),
        sequential_ordered!(WriteOutput::new(fps_out,frame_size))
    ];

    loop {
        // Read and post frames
        let mut frame = Mat::default()?;
        video_in.read(&mut frame)?;
        if frame.size()?.width == 0 {
            break;
        }
        pipeline.post(MatData{frame : frame}).unwrap();
    }

    pipeline.end_and_wait();

    Ok(())
}