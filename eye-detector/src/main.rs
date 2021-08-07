use {
	std::env,	
	opencv::core,
    std::time::SystemTime,
};
pub mod common;
mod seq;
mod par_rust_spp;
mod par_tokio;
mod par_std_threads;
mod par_better;

fn main() {
	
	let args: Vec<String> = env::args().collect();
	if args.len() < 4 {
		println!();
        panic!("Correct usage: $ ./{:?} <run_mode> <nthreads> <input_video>", args[0]);
	}

	// For our analysis, we don't want OpenCV's parallelism
	core::set_num_threads(1).unwrap();

    // Arguments
    let run_mode = &args[1];
    let nthreads = args[2].parse::<i32>().unwrap();
    let input_video = &args[3];

    let start = SystemTime::now();

    match run_mode.as_str() {
        "seq" => seq::seq_eye_tracker(input_video).unwrap(),
        "rust-spp" => par_rust_spp::rust_spp_eye_tracker(input_video,nthreads).unwrap(),
        "tokio" => par_tokio::tokio_eye_tracker(input_video,nthreads).unwrap(),
        "std-threads" => par_std_threads::std_threads_eye_tracker(input_video,nthreads).unwrap(),
        "better" => par_better::better_eye_tracker(input_video,nthreads).unwrap(),
        _ => println!("Invalid run_mode, use (seq | rust-spp | tokio)"),
    }

    let system_duration = start.elapsed().expect("Failed to get render time?");
    let in_sec = system_duration.as_secs() as f64 + system_duration.subsec_nanos() as f64 * 1e-9;
    println!("Execution time: {} sec", in_sec);

}
