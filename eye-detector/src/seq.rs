use {
    opencv::{
        prelude::*,
        core,
        objdetect,
        videoio,
    },
};

#[path = "common.rs"]
mod common;

pub fn seq_eye_tracker(input_video: &String) -> opencv::Result<()> {

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
    let mut video_out = videoio::VideoWriter::new(
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

    //"haarcascade_frontalface_alt.xml".to_owned()
    let face_xml = core::find_file("config/haarcascade_frontalface_alt.xml", true, false)?;
    let eye_xml = core::find_file("config/haarcascade_eye.xml", true, false)?;
    let mut face_detector = objdetect::CascadeClassifier::new(&face_xml)?;
    let mut eyes_detector = objdetect::CascadeClassifier::new(&eye_xml)?;

    loop {
        
        // Read frame
        let mut frame = Mat::default()?;
        video_in.read(&mut frame)?;
        if frame.size()?.width == 0 {
            break;
        }

        // Convert to gray and equalize frame
        let equalized = common::prepare_frame(&frame)?;

        // Detect faces
        let faces = common::detect_faces(&equalized,&mut face_detector)?;

        for face in faces {

            let eyes =  common::detect_eyes(&core::Mat::roi(&equalized,face)?,
            								&mut eyes_detector)?;

            common::draw_in_frame(&mut frame,&eyes,&face)?;

        }
        //Write output frame
        video_out.write(&mut frame)?;
    }
    Ok(())
}